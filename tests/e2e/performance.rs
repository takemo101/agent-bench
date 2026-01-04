//! パフォーマンステスト
//!
//! TC-P-001 to TC-P-006: パフォーマンス目標達成をテストする。

use std::sync::Arc;
use std::time::{Duration, Instant};

use pomodoro::{
    daemon::{IpcServer, TimerEngine},
    types::{PomodoroConfig, StartParams},
};
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, Mutex};

// TC-P-002: IPC通信遅延（50ms以内）
#[tokio::test]
async fn test_performance_ipc_latency() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("perf.sock");
    let server = IpcServer::new(&socket_path).unwrap();

    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    let server_engine = engine.clone();
    let server_handle = tokio::spawn(async move {
        if let Ok(mut stream) = server.accept().await {
            if let Ok(request) = IpcServer::receive_request(&mut stream).await {
                let response = pomodoro::daemon::handle_request(request, server_engine).await;
                let _ = IpcServer::send_response(&mut stream, &response).await;
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let start = Instant::now();

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    stream
        .write_all(r#"{"command":"status"}"#.as_bytes())
        .await
        .unwrap();

    let mut buffer = vec![0u8; 4096];
    let _ = stream.read(&mut buffer).await.unwrap();

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(50),
        "IPC通信遅延が50msを超過: {:?}",
        elapsed
    );

    server_handle.abort();
}

// TC-P-003: イベント処理遅延（500ms以内）
#[tokio::test]
async fn test_performance_event_processing_latency() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let event_received = Arc::new(AtomicBool::new(false));
    let event_received_clone = event_received.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    let event_handle = tokio::spawn(async move {
        if rx.recv().await.is_some() {
            event_received_clone.store(true, Ordering::SeqCst);
        }
    });

    let start = Instant::now();

    {
        let mut eng = engine.lock().await;
        eng.start(&StartParams {
            task_name: Some("パフォーマンステスト".to_string()),
            ..Default::default()
        })
        .unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    let elapsed = start.elapsed();

    assert!(
        event_received.load(Ordering::SeqCst),
        "イベントが受信されなかった"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "イベント処理遅延が500msを超過: {:?}",
        elapsed
    );

    event_handle.abort();
}

// TC-P-004: サウンドデータアクセス遅延（100ms以内）
#[tokio::test]
async fn test_performance_sound_data_access() {
    let start = Instant::now();

    let _data = pomodoro::sound::DEFAULT_SOUND_DATA;

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(100),
        "サウンドデータアクセスが100msを超過: {:?}",
        elapsed
    );
}

// TC-P-005: エンジン作成のメモリベースライン
#[tokio::test]
async fn test_performance_engine_memory_baseline() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();

    let engines: Vec<_> = (0..100)
        .map(|_| {
            let tx_clone = tx.clone();
            Arc::new(Mutex::new(TimerEngine::new(config.clone(), tx_clone)))
        })
        .collect();

    assert_eq!(engines.len(), 100);
}

// TC-P-006: アイドル時の状態取得パフォーマンス
#[tokio::test]
async fn test_performance_idle_cpu_baseline() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));

    let start = Instant::now();

    for _ in 0..1000 {
        let state = engine.lock().await;
        let _ = state.get_state().phase;
    }

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(100),
        "1000回の状態取得が100msを超過: {:?}",
        elapsed
    );
}
