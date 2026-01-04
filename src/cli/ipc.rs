//! IPCクライアント
//!
//! Unix Domain Socketを使用してデーモンサーバーと通信するクライアント。

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{sleep, timeout, Duration};

use crate::cli::commands::StartArgs;
use crate::types::{IpcRequest, IpcResponse, StartParams};

/// 接続タイムアウト（秒）
const CONNECTION_TIMEOUT_SECS: u64 = 5;

/// リクエストバッファサイズ
const REQUEST_BUFFER_SIZE: usize = 4096;

/// リトライ初期待機時間（ミリ秒）
const INITIAL_RETRY_DELAY_MS: u64 = 100;

/// リトライ最大待機時間（ミリ秒）
const MAX_RETRY_DELAY_MS: u64 = 2000;

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

    /// リクエストを送信
    ///
    /// デーモンサーバーにリクエストを送信し、レスポンスを受信する。
    /// タイムアウトは5秒。
    ///
    /// # Arguments
    ///
    /// * `req` - 送信するリクエスト
    ///
    /// # Returns
    ///
    /// サーバーからのレスポンス
    ///
    /// # Errors
    ///
    /// - ソケット接続に失敗した場合
    /// - リクエスト送信に失敗した場合
    /// - レスポンス受信に失敗した場合
    /// - タイムアウトした場合
    pub async fn send_request(&self, req: IpcRequest) -> Result<IpcResponse> {
        // タイムアウト付きで実行
        timeout(
            Duration::from_secs(CONNECTION_TIMEOUT_SECS),
            self.send_request_internal(req),
        )
        .await
        .context("Request timed out")?
    }

    /// リクエストを送信（内部実装）
    async fn send_request_internal(&self, req: IpcRequest) -> Result<IpcResponse> {
        // ソケットに接続
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to daemon")?;

        // リクエストをJSON形式でシリアライズ
        let request_json = serde_json::to_string(&req).context("Failed to serialize request")?;

        // リクエストを送信
        stream
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send request")?;

        // レスポンスを受信
        let mut buffer = vec![0u8; REQUEST_BUFFER_SIZE];
        let n = stream
            .read(&mut buffer)
            .await
            .context("Failed to read response")?;

        // レスポンスをデシリアライズ
        let response: IpcResponse = serde_json::from_slice(&buffer[..n])
            .context("Failed to deserialize response")?;

        Ok(response)
    }

    /// リトライ付きリクエスト送信
    ///
    /// 指定された回数までリトライを行う。
    /// リトライ間隔は指数バックオフ（100ms → 200ms → 400ms → ... 最大2秒）。
    ///
    /// # Arguments
    ///
    /// * `req` - 送信するリクエスト
    /// * `max_retries` - 最大リトライ回数
    ///
    /// # Returns
    ///
    /// サーバーからのレスポンス
    pub async fn send_request_with_retry(
        &self,
        req: IpcRequest,
        max_retries: u32,
    ) -> Result<IpcResponse> {
        let mut last_error = None;
        let mut delay_ms = INITIAL_RETRY_DELAY_MS;

        for attempt in 0..=max_retries {
            match self.send_request(req.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    
                    // 最後の試行でなければ待機
                    if attempt < max_retries {
                        sleep(Duration::from_millis(delay_ms)).await;
                        
                        // 指数バックオフ（最大2秒）
                        delay_ms = (delay_ms * 2).min(MAX_RETRY_DELAY_MS);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// タイマーを開始
    pub async fn start(&self, args: StartArgs) -> Result<IpcResponse> {
        let params = StartParams {
            work_minutes: Some(args.work),
            break_minutes: Some(args.break_time),
            long_break_minutes: Some(args.long_break),
            task_name: args.task,
            auto_cycle: Some(args.auto_cycle),
            focus_mode: Some(args.focus_mode),
        };

        self.send_request(IpcRequest::Start { params }).await
    }

    /// タイマーを一時停止
    pub async fn pause(&self) -> Result<IpcResponse> {
        self.send_request(IpcRequest::Pause).await
    }

    /// タイマーを再開
    pub async fn resume(&self) -> Result<IpcResponse> {
        self.send_request(IpcRequest::Resume).await
    }

    /// タイマーを停止
    pub async fn stop(&self) -> Result<IpcResponse> {
        self.send_request(IpcRequest::Stop).await
    }

    /// ステータスを取得
    pub async fn status(&self) -> Result<IpcResponse> {
        self.send_request(IpcRequest::Status).await
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
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    PathBuf::from(home)
        .join(".pomodoro")
        .join("pomodoro.sock")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseData;
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
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("timeout") || err_msg.contains("timed out"));
        
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
