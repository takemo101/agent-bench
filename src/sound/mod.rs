mod embedded;
mod error;
mod player;
mod source;

pub mod config;

pub use config::SoundConfig;
pub use embedded::DEFAULT_SOUND_DATA;
pub use error::SoundError;
pub use player::RodioSoundPlayer;
pub use source::SoundSource;

use async_trait::async_trait;

#[async_trait]
pub trait SoundPlayer: Send + Sync {
    async fn play(&self, source: &SoundSource) -> Result<(), SoundError>;
    fn is_available(&self) -> bool;
}

/// サウンドプレイヤーを作成するファクトリ関数
pub fn create_sound_player(disabled: bool) -> Box<dyn SoundPlayer> {
    match RodioSoundPlayer::new(disabled) {
        Ok(player) => Box::new(player),
        Err(_) => {
            // 初期化失敗時は無効化状態で作成を試みる、あるいはDummyを返す
            // ここでは簡易的にログを出して（できないが）、再度disabled=trueで呼ぶか、
            // RodioSoundPlayerがdisabled=trueなら必ず成功する前提で作る。
            // ひとまずnew(true)は必ずOkを返すように実装する想定で。
            if let Ok(p) = RodioSoundPlayer::new(true) {
                Box::new(p)
            } else {
                // ここには来ないはずだが、万が一の場合はpanicか...
                // エラー型を変えるわけにいかないので、Box<dyn SoundPlayer>を返す必要がある。
                // struct RodioSoundPlayerがOption<_stream>を持てばnew(true)は成功する。
                panic!("Failed to create disabled sound player");
            }
        }
    }
}
