use thiserror::Error;

#[derive(Debug, Error)]
pub enum SoundError {
    #[error("オーディオデバイスが利用できません: {0}")]
    DeviceNotAvailable(String),
    #[error("サウンドファイルが見つかりません: {0}")]
    FileNotFound(String),
    #[error("サウンドファイルのデコードに失敗しました: {0}")]
    DecodeError(String),
    #[error("オーディオストリームの作成に失敗しました: {0}")]
    StreamError(String),
    #[error("サウンド再生エラー: {0}")]
    Other(String),
}
