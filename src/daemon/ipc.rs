//! IPCサーバー
//!
//! Unix Domain Socketを使用したプロセス間通信サーバーを提供する。
//! CLIクライアントからのリクエストを受け付け、タイマーエンジンを操作する。

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use crate::daemon::TimerEngine;
use crate::types::{IpcRequest, IpcResponse, ResponseData, StartParams};

/// 接続タイムアウト（秒）
const CONNECTION_TIMEOUT_SECS: u64 = 5;

/// リクエストバッファサイズ
const REQUEST_BUFFER_SIZE: usize = 4096;

/// IPCサーバー
///
/// Unix Domain Socketでクライアントからのリクエストを受け付け、
/// タイマーエンジンを操作するサーバー。
pub struct IpcServer {
    /// Unixソケットリスナー
    listener: UnixListener,
    /// ソケットパス
    socket_path: std::path::PathBuf,
}

impl IpcServer {
    /// 新しいIPCサーバーを作成
    ///
    /// 指定されたパスにUnix Domain Socketを作成し、リッスンを開始する。
    /// 既存のソケットファイルがある場合は削除する。
    ///
    /// # Arguments
    ///
    /// * `socket_path` - ソケットファイルのパス
    ///
    /// # Errors
    ///
    /// - ソケットファイルの削除に失敗した場合
    /// - ソケットのバインドに失敗した場合
    pub fn new(socket_path: &Path) -> Result<Self> {
        // 親ディレクトリが存在することを確認
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
            }
        }

        // 既存のソケットファイルを削除
        if socket_path.exists() {
            std::fs::remove_file(socket_path).context("Failed to remove existing socket file")?;
        }

        let listener = UnixListener::bind(socket_path).context("Failed to bind Unix socket")?;

        Ok(Self {
            listener,
            socket_path: socket_path.to_path_buf(),
        })
    }

    /// クライアント接続を受け付ける
    ///
    /// 新しいクライアント接続があるまでブロックする。
    ///
    /// # Returns
    ///
    /// 接続されたUnixStream
    pub async fn accept(&self) -> Result<UnixStream> {
        let (stream, _) = self
            .listener
            .accept()
            .await
            .context("Failed to accept connection")?;
        Ok(stream)
    }

    /// リクエストを受信
    ///
    /// ストリームからJSONリクエストを読み取り、デシリアライズする。
    /// タイムアウトは5秒。
    ///
    /// # Arguments
    ///
    /// * `stream` - クライアントストリーム
    ///
    /// # Returns
    ///
    /// パースされたIpcRequest
    pub async fn receive_request(stream: &mut UnixStream) -> Result<IpcRequest> {
        let mut buffer = vec![0u8; REQUEST_BUFFER_SIZE];

        let n = timeout(
            Duration::from_secs(CONNECTION_TIMEOUT_SECS),
            stream.read(&mut buffer),
        )
        .await
        .context("Request read timed out")?
        .context("Failed to read from socket")?;

        if n == 0 {
            anyhow::bail!("Connection closed by client");
        }

        let request: IpcRequest =
            serde_json::from_slice(&buffer[..n]).context("Failed to parse request JSON")?;

        Ok(request)
    }

    /// レスポンスを送信
    ///
    /// IpcResponseをJSON形式でストリームに書き込む。
    ///
    /// # Arguments
    ///
    /// * `stream` - クライアントストリーム
    /// * `response` - 送信するレスポンス
    pub async fn send_response(stream: &mut UnixStream, response: &IpcResponse) -> Result<()> {
        let json = serde_json::to_vec(response).context("Failed to serialize response")?;

        stream
            .write_all(&json)
            .await
            .context("Failed to write to socket")?;

        stream.flush().await.context("Failed to flush socket")?;

        Ok(())
    }

    /// ソケットパスを取得
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // サーバー終了時にソケットファイルを削除
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// リクエストを処理
///
/// IpcRequestを解析し、適切なTimerEngine操作を実行してレスポンスを返す。
///
/// # Arguments
///
/// * `request` - 処理するリクエスト
/// * `engine` - タイマーエンジン
///
/// # Returns
///
/// 処理結果を含むIpcResponse
pub async fn handle_request(request: IpcRequest, engine: Arc<Mutex<TimerEngine>>) -> IpcResponse {
    let mut engine = engine.lock().await;

    match request {
        IpcRequest::Start { params } => handle_start(&mut engine, params),
        IpcRequest::Pause => handle_pause(&mut engine),
        IpcRequest::Resume => handle_resume(&mut engine),
        IpcRequest::Stop => handle_stop(&mut engine),
        IpcRequest::Status => handle_status(&engine),
    }
}

/// startコマンドを処理
fn handle_start(engine: &mut TimerEngine, params: StartParams) -> IpcResponse {
    match engine.start(params.task_name.clone()) {
        Ok(()) => {
            let state = engine.get_state();
            IpcResponse::success(
                "タイマーを開始しました",
                Some(ResponseData {
                    state: Some(state.phase.as_str().to_string()),
                    remaining_seconds: Some(state.remaining_seconds),
                    pomodoro_count: Some(state.pomodoro_count),
                    task_name: state.task_name.clone(),
                }),
            )
        }
        Err(e) => IpcResponse::error(e.to_string()),
    }
}

/// pauseコマンドを処理
fn handle_pause(engine: &mut TimerEngine) -> IpcResponse {
    match engine.pause() {
        Ok(()) => IpcResponse::success("タイマーを一時停止しました", None),
        Err(e) => IpcResponse::error(e.to_string()),
    }
}

/// resumeコマンドを処理
fn handle_resume(engine: &mut TimerEngine) -> IpcResponse {
    match engine.resume() {
        Ok(()) => IpcResponse::success("タイマーを再開しました", None),
        Err(e) => IpcResponse::error(e.to_string()),
    }
}

/// stopコマンドを処理
fn handle_stop(engine: &mut TimerEngine) -> IpcResponse {
    match engine.stop() {
        Ok(()) => IpcResponse::success("タイマーを停止しました", None),
        Err(e) => IpcResponse::error(e.to_string()),
    }
}

/// statusコマンドを処理
fn handle_status(engine: &TimerEngine) -> IpcResponse {
    let state = engine.get_state();
    IpcResponse::success(
        "",
        Some(ResponseData {
            state: Some(state.phase.as_str().to_string()),
            remaining_seconds: Some(state.remaining_seconds),
            pomodoro_count: Some(state.pomodoro_count),
            task_name: state.task_name.clone(),
        }),
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PomodoroConfig;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::sync::mpsc;

    // ------------------------------------------------------------------------
    // Helper Functions
    // ------------------------------------------------------------------------

    fn create_test_socket_path() -> PathBuf {
        let dir = tempdir().unwrap();
        dir.into_path().join("test.sock")
    }

    fn create_test_engine() -> Arc<Mutex<TimerEngine>> {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = PomodoroConfig::default();
        Arc::new(Mutex::new(TimerEngine::new(config, tx)))
    }

    // ------------------------------------------------------------------------
    // IpcServer Creation Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_ipc_server_new() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path);

        assert!(server.is_ok());
        assert!(socket_path.exists());

        // Cleanup
        drop(server);
    }

    #[tokio::test]
    async fn test_ipc_server_new_removes_existing_socket() {
        let socket_path = create_test_socket_path();

        // Create first server
        let server1 = IpcServer::new(&socket_path).unwrap();
        drop(server1);

        // Create file at socket path to simulate leftover
        std::fs::write(&socket_path, "dummy").unwrap();

        // Create second server should succeed
        let server2 = IpcServer::new(&socket_path);
        assert!(server2.is_ok());
    }

    #[tokio::test]
    async fn test_ipc_server_new_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("nested").join("test.sock");

        let server = IpcServer::new(&socket_path);
        assert!(server.is_ok());
        assert!(socket_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_ipc_server_socket_path() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        assert_eq!(server.socket_path(), socket_path);
    }

    #[tokio::test]
    async fn test_ipc_server_drop_removes_socket() {
        let socket_path = create_test_socket_path();

        {
            let _server = IpcServer::new(&socket_path).unwrap();
            assert!(socket_path.exists());
        }

        // After drop, socket should be removed
        assert!(!socket_path.exists());
    }

    // ------------------------------------------------------------------------
    // Client Connection Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_ipc_server_accept() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        // Spawn client connection
        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            UnixStream::connect(&client_path).await
        });

        // Accept connection
        let stream = server.accept().await;
        assert!(stream.is_ok());

        let client_result = client_handle.await.unwrap();
        assert!(client_result.is_ok());
    }

    // ------------------------------------------------------------------------
    // Request/Response Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_receive_request_status() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = UnixStream::connect(&client_path).await.unwrap();
            let request = r#"{"command":"status"}"#;
            stream.write_all(request.as_bytes()).await.unwrap();
            stream
        });

        let mut stream = server.accept().await.unwrap();
        let request = IpcServer::receive_request(&mut stream).await;

        assert!(request.is_ok());
        assert!(matches!(request.unwrap(), IpcRequest::Status));

        client_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_receive_request_start() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = UnixStream::connect(&client_path).await.unwrap();
            let request = r#"{"command":"start","taskName":"テスト"}"#;
            stream.write_all(request.as_bytes()).await.unwrap();
            stream
        });

        let mut stream = server.accept().await.unwrap();
        let request = IpcServer::receive_request(&mut stream).await.unwrap();

        if let IpcRequest::Start { params } = request {
            assert_eq!(params.task_name, Some("テスト".to_string()));
        } else {
            panic!("Expected Start request");
        }

        client_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_receive_request_empty_connection() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let stream = UnixStream::connect(&client_path).await.unwrap();
            // Close immediately without sending
            drop(stream);
        });

        let mut stream = server.accept().await.unwrap();
        let result = IpcServer::receive_request(&mut stream).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("closed"));

        client_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_send_response() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();

        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = UnixStream::connect(&client_path).await.unwrap();

            let mut buffer = vec![0u8; 4096];
            let n = stream.read(&mut buffer).await.unwrap();
            String::from_utf8(buffer[..n].to_vec()).unwrap()
        });

        let mut stream = server.accept().await.unwrap();
        let response = IpcResponse::success("OK", None);
        let result = IpcServer::send_response(&mut stream, &response).await;

        assert!(result.is_ok());

        let client_response = client_handle.await.unwrap();
        assert!(client_response.contains("\"status\":\"success\""));
        assert!(client_response.contains("\"message\":\"OK\""));
    }

    // ------------------------------------------------------------------------
    // Handle Request Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_handle_request_status() {
        let engine = create_test_engine();

        let response = handle_request(IpcRequest::Status, engine).await;

        assert_eq!(response.status, "success");
        assert!(response.data.is_some());
        let data = response.data.unwrap();
        assert_eq!(data.state, Some("stopped".to_string()));
    }

    #[tokio::test]
    async fn test_handle_request_start() {
        let engine = create_test_engine();

        let request = IpcRequest::Start {
            params: StartParams {
                task_name: Some("開発".to_string()),
                ..Default::default()
            },
        };

        let response = handle_request(request, engine).await;

        assert_eq!(response.status, "success");
        assert_eq!(response.message, "タイマーを開始しました");
        assert!(response.data.is_some());
        let data = response.data.unwrap();
        assert_eq!(data.state, Some("working".to_string()));
        assert_eq!(data.task_name, Some("開発".to_string()));
    }

    #[tokio::test]
    async fn test_handle_request_start_already_running() {
        let engine = create_test_engine();

        // Start first
        let request = IpcRequest::Start {
            params: StartParams::default(),
        };
        handle_request(request, engine.clone()).await;

        // Try to start again
        let request = IpcRequest::Start {
            params: StartParams::default(),
        };
        let response = handle_request(request, engine).await;

        assert_eq!(response.status, "error");
        assert!(response.message.contains("既に実行中"));
    }

    #[tokio::test]
    async fn test_handle_request_pause() {
        let engine = create_test_engine();

        // Start first
        let start_request = IpcRequest::Start {
            params: StartParams::default(),
        };
        handle_request(start_request, engine.clone()).await;

        // Pause
        let response = handle_request(IpcRequest::Pause, engine).await;

        assert_eq!(response.status, "success");
        assert_eq!(response.message, "タイマーを一時停止しました");
    }

    #[tokio::test]
    async fn test_handle_request_pause_not_running() {
        let engine = create_test_engine();

        let response = handle_request(IpcRequest::Pause, engine).await;

        assert_eq!(response.status, "error");
        assert!(response.message.contains("実行されていません"));
    }

    #[tokio::test]
    async fn test_handle_request_resume() {
        let engine = create_test_engine();

        // Start then pause
        let start_request = IpcRequest::Start {
            params: StartParams::default(),
        };
        handle_request(start_request, engine.clone()).await;
        handle_request(IpcRequest::Pause, engine.clone()).await;

        // Resume
        let response = handle_request(IpcRequest::Resume, engine).await;

        assert_eq!(response.status, "success");
        assert_eq!(response.message, "タイマーを再開しました");
    }

    #[tokio::test]
    async fn test_handle_request_resume_not_paused() {
        let engine = create_test_engine();

        // Start but don't pause
        let start_request = IpcRequest::Start {
            params: StartParams::default(),
        };
        handle_request(start_request, engine.clone()).await;

        let response = handle_request(IpcRequest::Resume, engine).await;

        assert_eq!(response.status, "error");
        assert!(response.message.contains("一時停止していません"));
    }

    #[tokio::test]
    async fn test_handle_request_stop() {
        let engine = create_test_engine();

        // Start first
        let start_request = IpcRequest::Start {
            params: StartParams::default(),
        };
        handle_request(start_request, engine.clone()).await;

        // Stop
        let response = handle_request(IpcRequest::Stop, engine).await;

        assert_eq!(response.status, "success");
        assert_eq!(response.message, "タイマーを停止しました");
    }

    #[tokio::test]
    async fn test_handle_request_stop_not_running() {
        let engine = create_test_engine();

        let response = handle_request(IpcRequest::Stop, engine).await;

        assert_eq!(response.status, "error");
        assert!(response.message.contains("実行されていません"));
    }

    // ------------------------------------------------------------------------
    // Integration Tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_full_ipc_flow() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();
        let engine = create_test_engine();

        // Client task
        let client_path = socket_path.clone();
        let client_handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;

            let mut stream = UnixStream::connect(&client_path).await.unwrap();

            // Send status request
            let request = r#"{"command":"status"}"#;
            stream.write_all(request.as_bytes()).await.unwrap();

            // Read response
            let mut buffer = vec![0u8; 4096];
            let n = stream.read(&mut buffer).await.unwrap();
            String::from_utf8(buffer[..n].to_vec()).unwrap()
        });

        // Server handling
        let mut stream = server.accept().await.unwrap();
        let request = IpcServer::receive_request(&mut stream).await.unwrap();
        let response = handle_request(request, engine).await;
        IpcServer::send_response(&mut stream, &response)
            .await
            .unwrap();

        // Verify client received response
        let client_response = client_handle.await.unwrap();
        assert!(client_response.contains("\"status\":\"success\""));
        assert!(client_response.contains("\"stopped\""));
    }

    #[tokio::test]
    async fn test_multiple_sequential_requests() {
        let socket_path = create_test_socket_path();
        let server = IpcServer::new(&socket_path).unwrap();
        let engine = create_test_engine();

        // First request: start
        let client_path = socket_path.clone();
        let client1 = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = UnixStream::connect(&client_path).await.unwrap();
            stream
                .write_all(r#"{"command":"start","taskName":"タスク1"}"#.as_bytes())
                .await
                .unwrap();
            let mut buffer = vec![0u8; 4096];
            let n = stream.read(&mut buffer).await.unwrap();
            String::from_utf8(buffer[..n].to_vec()).unwrap()
        });

        let mut stream1 = server.accept().await.unwrap();
        let request1 = IpcServer::receive_request(&mut stream1).await.unwrap();
        let response1 = handle_request(request1, engine.clone()).await;
        IpcServer::send_response(&mut stream1, &response1)
            .await
            .unwrap();

        let result1 = client1.await.unwrap();
        assert!(result1.contains("\"working\""));

        // Second request: status
        let client_path = socket_path.clone();
        let client2 = tokio::spawn(async move {
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

        let mut stream2 = server.accept().await.unwrap();
        let request2 = IpcServer::receive_request(&mut stream2).await.unwrap();
        let response2 = handle_request(request2, engine).await;
        IpcServer::send_response(&mut stream2, &response2)
            .await
            .unwrap();

        let result2 = client2.await.unwrap();
        assert!(result2.contains("\"working\""));
        assert!(result2.contains("タスク1"));
    }
}
