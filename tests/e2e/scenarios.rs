//! E2Eシナリオテスト
//!
//! TC-E-001 to TC-E-006: エンドツーエンドのユーザーフローをテストする。
//!
//! 注意: E2Eテストは実際のタイマー時間を使用せず、高速化されたシミュレーションで検証する。

use std::sync::Arc;
use std::time::Duration;

use pomodoro::{
    daemon::{handle_request, IpcServer, TimerEngine, TimerEvent},
    types::{PomodoroConfig, StartParams, TimerPhase},
};
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, Mutex};

fn create_fast_test_engine() -> (Arc<Mutex<TimerEngine>>, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: false,
        focus_mode: false,
    };
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));
    (engine, rx)
}

async fn send_ipc_request(socket_path: &std::path::Path, request_json: &str) -> String {
    let mut stream = UnixStream::connect(socket_path).await.unwrap();
    stream.write_all(request_json.as_bytes()).await.unwrap();

    let mut buffer = vec![0u8; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    String::from_utf8(buffer[..n].to_vec()).unwrap()
}

// TC-E-001: 完全なポモドーロサイクル
#[tokio::test]
async fn test_e2e_complete_pomodoro_cycle() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");
    let (engine, mut _event_rx) = create_fast_test_engine();

    let server_engine = engine.clone();
    let server_path = socket_path.clone();
    let server_handle = tokio::spawn(async move {
        let server = IpcServer::new(&server_path).unwrap();
        for _ in 0..2 {
            if let Ok(mut stream) = server.accept().await {
                if let Ok(request) = IpcServer::receive_request(&mut stream).await {
                    let response = handle_request(request, server_engine.clone()).await;
                    let _ = IpcServer::send_response(&mut stream, &response).await;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    let response = send_ipc_request(
        &socket_path,
        r#"{"command":"start","taskName":"E2Eテスト"}"#,
    )
    .await;
    assert!(response.contains("\"status\":\"success\""));
    assert!(response.contains("\"working\""));

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Working);
        assert_eq!(state.get_state().task_name, Some("E2Eテスト".to_string()));
    }

    // Work phase: tick until phase changes to Breaking
    loop {
        let mut eng = engine.lock().await;
        let processed = eng.process_tick().unwrap();
        if !processed || eng.get_state().phase != TimerPhase::Working {
            break;
        }
    }

    // Verify transitioned to Breaking phase
    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Breaking);
        // remaining_seconds is reset to break duration
        assert_eq!(state.get_state().remaining_seconds, 60);
    }

    // Break phase: tick until phase changes (auto_cycle=false -> Stopped)
    loop {
        let mut eng = engine.lock().await;
        let processed = eng.process_tick().unwrap();
        if !processed || eng.get_state().phase != TimerPhase::Breaking {
            break;
        }
    }

    // Verify stopped (auto_cycle=false)
    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Stopped);
    }

    server_handle.abort();
}

// TC-E-002: 一時停止・再開フロー
#[tokio::test]
async fn test_e2e_pause_resume_flow() {
    let (engine, _rx) = create_fast_test_engine();

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("一時停止テスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    for _ in 0..10 {
        let mut eng = engine.lock().await;
        let _ = eng.process_tick();
    }

    let remaining_before_pause = {
        let state = engine.lock().await;
        state.get_state().remaining_seconds
    };

    {
        let mut eng = engine.lock().await;
        eng.pause().unwrap();
    }

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Paused);
        assert_eq!(state.get_state().remaining_seconds, remaining_before_pause);
    }

    for _ in 0..5 {
        let eng = engine.lock().await;
        assert_eq!(eng.get_state().remaining_seconds, remaining_before_pause);
    }

    {
        let mut eng = engine.lock().await;
        eng.resume().unwrap();
    }

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Working);
        assert_eq!(state.get_state().remaining_seconds, remaining_before_pause);
    }

    {
        let mut eng = engine.lock().await;
        let _ = eng.process_tick();
        assert_eq!(
            eng.get_state().remaining_seconds,
            remaining_before_pause - 1
        );
    }
}

// TC-E-003: 停止フロー
#[tokio::test]
async fn test_e2e_stop_flow() {
    let (engine, _rx) = create_fast_test_engine();

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("停止テスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    for _ in 0..10 {
        let mut eng = engine.lock().await;
        let _ = eng.process_tick();
    }

    {
        let mut eng = engine.lock().await;
        eng.stop().unwrap();
    }

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Stopped);
        assert_eq!(state.get_state().remaining_seconds, 0);
        assert_eq!(state.get_state().task_name, None);
    }
}

// TC-E-004: 自動サイクル有効時の動作
#[tokio::test]
async fn test_e2e_auto_cycle() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: true,
        focus_mode: false,
    };
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("自動サイクルテスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    loop {
        let mut eng = engine.lock().await;
        let processed = eng.process_tick().unwrap();
        if eng.get_state().phase == TimerPhase::Breaking {
            break;
        }
        if !processed {
            break;
        }
    }

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Breaking);
    }

    loop {
        let mut eng = engine.lock().await;
        let processed = eng.process_tick().unwrap();
        if eng.get_state().phase == TimerPhase::Working && eng.get_state().pomodoro_count == 1 {
            break;
        }
        if !processed {
            break;
        }
    }

    {
        let state = engine.lock().await;
        assert_eq!(state.get_state().phase, TimerPhase::Working);
        assert_eq!(state.get_state().pomodoro_count, 1);
    }
}

// TC-E-005: 4ポモドーロ後の長い休憩
#[tokio::test]
async fn test_e2e_long_break_after_four_pomodoros() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 3,
        auto_cycle: true,
        focus_mode: false,
    };
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("4ポモドーロテスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    for i in 1..=4 {
        loop {
            let mut eng = engine.lock().await;
            let _ = eng.process_tick();
            if eng.get_state().phase != TimerPhase::Working
                || eng.get_state().remaining_seconds == 0
            {
                break;
            }
        }

        {
            let state = engine.lock().await;
            assert_eq!(state.get_state().pomodoro_count, i as u32);
        }

        {
            let state = engine.lock().await;
            if i == 4 {
                assert_eq!(
                    state.get_state().phase,
                    TimerPhase::LongBreaking,
                    "4ポモドーロ後は長い休憩になるべき"
                );
            } else {
                assert_eq!(state.get_state().phase, TimerPhase::Breaking);
            }
        }

        if i < 4 {
            loop {
                let mut eng = engine.lock().await;
                let _ = eng.process_tick();
                if eng.get_state().phase == TimerPhase::Working {
                    break;
                }
            }
        }
    }
}

// TC-E-006: フォーカスモード連携
#[tokio::test]
async fn test_e2e_focus_mode_integration() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let focus_enabled = Arc::new(AtomicBool::new(false));
    let focus_enabled_clone = focus_enabled.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig {
        work_minutes: 1,
        break_minutes: 1,
        long_break_minutes: 2,
        auto_cycle: false,
        focus_mode: true,
    };
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    let event_handle = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                TimerEvent::WorkStarted { .. } => {
                    focus_enabled_clone.store(true, Ordering::SeqCst);
                }
                TimerEvent::BreakStarted { .. } => {
                    focus_enabled_clone.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    });

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("フォーカスモードテスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(
        focus_enabled.load(Ordering::SeqCst),
        "作業開始時にフォーカスモードが有効になるべき"
    );

    loop {
        let mut eng = engine.lock().await;
        let _ = eng.process_tick();
        if eng.get_state().phase == TimerPhase::Breaking {
            break;
        }
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(
        !focus_enabled.load(Ordering::SeqCst),
        "休憩開始時にフォーカスモードが無効になるべき"
    );

    event_handle.abort();
}
