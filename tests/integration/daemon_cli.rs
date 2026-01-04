//! Daemon-CLI間IPC統合テスト
//!
//! TC-I-001 to TC-I-004: IPCを介したDaemon-CLI連携をテストする。

use std::sync::Arc;
use std::time::Duration;

use pomodoro::{
    daemon::{handle_request, IpcServer, TimerEngine},
    types::{IpcRequest, IpcResponse, PomodoroConfig, StartParams, TimerPhase},
};
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, Mutex};

/// テスト用エンジンを作成
fn create_test_engine() -> (
    Arc<Mutex<TimerEngine>>,
    mpsc::UnboundedReceiver<pomodoro::daemon::TimerEvent>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    let engine = Arc::new(Mutex::new(TimerEngine::new(config, tx)));
    (engine, rx)
}

// ============================================================================
// TC-I-001: タイマー開始（IPC経由）
// ============================================================================

#[tokio::test]
async fn test_ipc_start_timer() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");
    let server = IpcServer::new(&socket_path).unwrap();
    let (engine, _rx) = create_test_engine();

    // クライアントタスク: startリクエスト送信
    let client_path = socket_path.clone();
    let client_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut stream = UnixStream::connect(&client_path).await.unwrap();
        let request = r#"{"command":"start","taskName":"API実装"}"#;
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await.unwrap();
        String::from_utf8(buffer[..n].to_vec()).unwrap()
    });

    // サーバー処理
    let mut stream = server.accept().await.unwrap();
    let request = IpcServer::receive_request(&mut stream).await.unwrap();
    let response = handle_request(request, engine.clone()).await;
    IpcServer::send_response(&mut stream, &response)
        .await
        .unwrap();

    // レスポンス検証
    let client_response = client_handle.await.unwrap();
    assert!(client_response.contains("\"status\":\"success\""));
    assert!(client_response.contains("\"working\""));

    // エンジン状態検証
    let state = engine.lock().await;
    assert_eq!(state.get_state().phase, TimerPhase::Working);
    assert_eq!(state.get_state().task_name, Some("API実装".to_string()));
}

// ============================================================================
// TC-I-002: タイマー一時停止（IPC経由）
// ============================================================================

#[tokio::test]
async fn test_ipc_pause_timer() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");
    let server = IpcServer::new(&socket_path).unwrap();
    let (engine, _rx) = create_test_engine();

    // 事前にタイマーを開始
    {
        let mut eng = engine.lock().await;
        eng.start(Some(StartParams { task_name: Some("タスク".to_string()), ..Default::default() })).unwrap();
    }

    // クライアント: pauseリクエスト
    let client_path = socket_path.clone();
    let client_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut stream = UnixStream::connect(&client_path).await.unwrap();
        stream
            .write_all(r#"{"command":"pause"}"#.as_bytes())
            .await
            .unwrap();

        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await.unwrap();
        String::from_utf8(buffer[..n].to_vec()).unwrap()
    });

    // サーバー処理
    let mut stream = server.accept().await.unwrap();
    let request = IpcServer::receive_request(&mut stream).await.unwrap();
    let response = handle_request(request, engine.clone()).await;
    IpcServer::send_response(&mut stream, &response)
        .await
        .unwrap();

    // レスポンス検証
    let client_response = client_handle.await.unwrap();
    assert!(client_response.contains("\"status\":\"success\""));
    assert!(client_response.contains("一時停止"));

    // 状態検証
    let state = engine.lock().await;
    assert_eq!(state.get_state().phase, TimerPhase::Paused);
}

// ============================================================================
// TC-I-003: ステータス確認（IPC経由）
// ============================================================================

#[tokio::test]
async fn test_ipc_status() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");
    let server = IpcServer::new(&socket_path).unwrap();
    let (engine, _rx) = create_test_engine();

    // タイマー開始
    {
        let mut eng = engine.lock().await;
        eng.start(Some(StartParams { task_name: Some("ステータス確認タスク".to_string()), ..Default::default() })).unwrap();
    }

    // クライアント: statusリクエスト
    let client_path = socket_path.clone();
    let client_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut stream = UnixStream::connect(&client_path).await.unwrap();
        stream
            .write_all(r#"{"command":"status"}"#.as_bytes())
            .await
            .unwrap();

        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer).await.unwrap();
        String::from_utf8(buffer[..n].to_vec()).unwrap()
    });

    // サーバー処理
    let mut stream = server.accept().await.unwrap();
    let request = IpcServer::receive_request(&mut stream).await.unwrap();
    let response = handle_request(request, engine.clone()).await;
    IpcServer::send_response(&mut stream, &response)
        .await
        .unwrap();

    // レスポンス検証
    let client_response = client_handle.await.unwrap();
    let response: IpcResponse = serde_json::from_str(&client_response).unwrap();

    assert_eq!(response.status, "success");
    let data = response.data.unwrap();
    assert_eq!(data.state, Some("working".to_string()));
    assert_eq!(data.task_name, Some("ステータス確認タスク".to_string()));
    assert!(data.remaining_seconds.is_some());
}

// ============================================================================
// TC-I-004: IPC接続エラー
// ============================================================================

#[tokio::test]
async fn test_ipc_connection_error() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("nonexistent.sock");

    // 存在しないソケットへの接続はエラーになる
    let result = UnixStream::connect(&socket_path).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

// ============================================================================
// 追加: 複数コマンドのシーケンステスト
// ============================================================================

#[tokio::test]
async fn test_ipc_command_sequence() {
    let (engine, _rx) = create_test_engine();

    // Start
    let start_response = handle_request(
        IpcRequest::Start {
            params: StartParams {
                task_name: Some("シーケンステスト".to_string()),
                ..Default::default()
            },
        },
        engine.clone(),
    )
    .await;
    assert_eq!(start_response.status, "success");

    // Status
    let status_response = handle_request(IpcRequest::Status, engine.clone()).await;
    assert_eq!(status_response.status, "success");
    let data = status_response.data.unwrap();
    assert_eq!(data.state, Some("working".to_string()));

    // Pause
    let pause_response = handle_request(IpcRequest::Pause, engine.clone()).await;
    assert_eq!(pause_response.status, "success");

    // Resume
    let resume_response = handle_request(IpcRequest::Resume, engine.clone()).await;
    assert_eq!(resume_response.status, "success");

    // Stop
    let stop_response = handle_request(IpcRequest::Stop, engine.clone()).await;
    assert_eq!(stop_response.status, "success");

    // 最終状態確認
    let final_status = handle_request(IpcRequest::Status, engine.clone()).await;
    let final_data = final_status.data.unwrap();
    assert_eq!(final_data.state, Some("stopped".to_string()));
}
