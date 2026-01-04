//! IPCクライアント
//!
//! Unix Domain Socketを使用してデーモンサーバーと通信するクライアント。

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

use crate::cli::commands::StartArgs;
use crate::types::{IpcRequest, IpcResponse, StartParams};

/// 接続タイムアウト（秒）
const CONNECTION_TIMEOUT_SECS: u64 = 5;

/// リクエストバッファサイズ
const REQUEST_BUFFER_SIZE: usize = 4096;

/// IPCクライアント
///
/// Unix Domain Socketを使用してデーモンサーバーと通信する。
pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    /// 新しいIPCクライアントを作成
    pub fn new() -> Self {
        Self {
            socket_path: get_socket_path(),
        }
    }

    /// ソケットパスを指定してIPCクライアントを作成（テスト用）
    #[cfg(test)]
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// ソケットパスを取得
///
/// `~/.pomodoro/pomodoro.sock` を返す。
fn get_socket_path() -> PathBuf {
    todo!("Not implemented yet")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ResponseData, StartParams};
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::net::UnixListener;

    /// モックサーバーを起動
    async fn start_mock_server(socket_path: &Path) -> UnixListener {
        // 既存のソケットファイルを削除
        if socket_path.exists() {
            std::fs::remove_file(socket_path).unwrap();
        }

        UnixListener::bind(socket_path).unwrap()
    }

    /// モックサーバーがリクエストを受信してレスポンスを返す
    async fn mock_server_respond(listener: UnixListener, response: IpcResponse) {
        let (mut stream, _) = listener.accept().await.unwrap();
        
        // リクエストを読み取る（検証はしない）
        let mut buffer = vec![0u8; REQUEST_BUFFER_SIZE];
        let _ = stream.read(&mut buffer).await.unwrap();
        
        // レスポンスを返す
        let response_json = serde_json::to_string(&response).unwrap();
        stream.write_all(response_json.as_bytes()).await.unwrap();
    }

    #[test]
    fn test_get_socket_path() {
        let path = get_socket_path();
        assert!(path.to_str().unwrap().contains(".pomodoro"));
        assert!(path.to_str().unwrap().ends_with("pomodoro.sock"));
    }

    #[test]
    fn test_ipc_client_new() {
        let client = IpcClient::new();
        assert!(client.socket_path.to_str().unwrap().contains(".pomodoro"));
    }

    #[test]
    fn test_ipc_client_default() {
        let client = IpcClient::default();
        assert!(client.socket_path.to_str().unwrap().contains(".pomodoro"));
    }

    #[tokio::test]
    async fn test_send_request_success() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        // モックサーバーを起動
        let response = IpcResponse::success("OK", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        // リクエストを送信
        let result = client.send_request(IpcRequest::Status).await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "success");
        assert_eq!(resp.message, "OK");
    }

    #[tokio::test]
    async fn test_send_request_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_timeout.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        // サーバーは応答しない（タイムアウトをテスト）
        let server_handle = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            // 応答せずに待機
            tokio::time::sleep(Duration::from_secs(10)).await;
        });

        // リクエストを送信（タイムアウトするはず）
        let result = client.send_request(IpcRequest::Status).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout") || 
                result.unwrap_err().to_string().contains("timed out"));
        
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_send_request_connection_refused() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("nonexistent.sock");
        
        let client = IpcClient::with_socket_path(socket_path);

        // 存在しないソケットに接続を試みる
        let result = client.send_request(IpcRequest::Status).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_request_with_retry_success_first_try() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_retry.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success("OK", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        // リトライ付きリクエストを送信（1回目で成功するはず）
        let result = client.send_request_with_retry(IpcRequest::Status, 3).await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "success");
    }

    #[tokio::test]
    async fn test_start_command() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_start.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success("Timer started", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        let args = StartArgs {
            work: 25,
            break_time: 5,
            long_break: 15,
            task: Some("Test task".to_string()),
            auto_cycle: false,
            focus_mode: false,
            no_sound: false,
        };

        let result = client.start(args).await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pause_command() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_pause.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success("Timer paused", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        let result = client.pause().await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resume_command() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_resume.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success("Timer resumed", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        let result = client.resume().await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_command() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_stop.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success("Timer stopped", None);
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        let result = client.stop().await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_status_command() {
        let temp_dir = TempDir::new().unwrap();
        let socket_path = temp_dir.path().join("test_status.sock");
        
        let listener = start_mock_server(&socket_path).await;
        let client = IpcClient::with_socket_path(socket_path.clone());

        let response = IpcResponse::success(
            "Status retrieved",
            Some(ResponseData {
                state: Some("working".to_string()),
                remaining_seconds: Some(1500),
                pomodoro_count: Some(2),
                task_name: Some("Test".to_string()),
            }),
        );
        let server_handle = tokio::spawn(mock_server_respond(listener, response.clone()));

        let result = client.status().await;
        
        server_handle.await.unwrap();
        
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.data.is_some());
    }
}
