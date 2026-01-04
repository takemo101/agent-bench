use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder};
use std::io::Cursor;
use async_trait::async_trait;
use super::{SoundPlayer, SoundSource, SoundError};

pub struct RodioSoundPlayer {
    _stream: Option<OutputStream>, // Option to allow dummy creation if needed, or stick to struct def
    stream_handle: Option<OutputStreamHandle>,
    disabled: bool,
}

impl RodioSoundPlayer {
    pub fn new(disabled: bool) -> Result<Self, SoundError> {
        if disabled {
             return Ok(Self { _stream: None, stream_handle: None, disabled: true });
        }
        // Stub
        Err(SoundError::DeviceNotAvailable("Not implemented".to_string()))
    }
}

#[async_trait]
impl SoundPlayer for RodioSoundPlayer {
    async fn play(&self, source: &SoundSource) -> Result<(), SoundError> {
        if self.disabled {
            return Ok(());
        }
        Err(SoundError::PlaybackFailed("Not implemented".to_string()))
    }

    fn is_available(&self) -> bool {
        !self.disabled
    }
}
