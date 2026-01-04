mod error;
mod embedded;

// インラインモジュール定義（ファイル数削減のため）
pub mod source {
    #[derive(Debug, Clone)]
    pub enum SoundSource {
        Embedded,
    }
}

pub mod player {
    // 将来的な実装用
}

pub use error::SoundError;
pub use embedded::DEFAULT_SOUND_DATA;
pub use source::SoundSource;

use async_trait::async_trait;

#[async_trait]
pub trait SoundPlayer: Send + Sync {
    async fn play(&self, source: &SoundSource) -> Result<(), SoundError>;
    fn is_available(&self) -> bool;
}

/// サウンドプレイヤーを作成するファクトリ関数（プレースホルダー）
pub fn create_sound_player(disabled: bool) -> Box<dyn SoundPlayer> {
    if disabled {
        Box::new(DummySoundPlayer)
    } else {
        // まだ実装がないのでとりあえずDummyを返す
        Box::new(DummySoundPlayer)
    }
}

struct DummySoundPlayer;

#[async_trait]
impl SoundPlayer for DummySoundPlayer {
    async fn play(&self, _source: &SoundSource) -> Result<(), SoundError> {
        Ok(())
    }
    fn is_available(&self) -> bool {
        false
    }
}
