use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder};
use std::io::Cursor;
use async_trait::async_trait;
use super::{SoundPlayer, SoundSource, SoundError};
use std::sync::mpsc;
use std::thread;

pub struct RodioSoundPlayer {
    stream_handle: Option<OutputStreamHandle>,
    // Keeps the background thread alive. When dropped, channel closes, thread exits.
    _keep_alive: Option<mpsc::Sender<()>>,
    disabled: bool,
}

impl RodioSoundPlayer {
    pub fn new(disabled: bool) -> Result<Self, SoundError> {
        if disabled {
             return Ok(Self { stream_handle: None, _keep_alive: None, disabled: true });
        }

        let (tx_handle, rx_handle) = mpsc::channel();
        let (tx_keep_alive, rx_keep_alive) = mpsc::channel::<()>();

        thread::spawn(move || {
            // Attempt to create OutputStream
            // On Linux/ALSA this might fail if no device, or return a stream that isn't Send
            // But we keep it in this thread.
            match OutputStream::try_default() {
                Ok((_stream, handle)) => {
                    if tx_handle.send(Ok(handle)).is_err() {
                        return; // Main thread gone
                    }
                    // Wait until main thread drops tx_keep_alive
                    let _ = rx_keep_alive.recv();
                    // _stream is dropped here, stopping audio
                }
                Err(e) => {
                    let _ = tx_handle.send(Err(SoundError::DeviceNotAvailable(e.to_string())));
                }
            }
        });

        match rx_handle.recv() {
            Ok(Ok(handle)) => Ok(Self {
                stream_handle: Some(handle),
                _keep_alive: Some(tx_keep_alive),
                disabled: false,
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(SoundError::StreamError("Audio thread failed to initialize".to_string())),
        }
    }
}

#[async_trait]
impl SoundPlayer for RodioSoundPlayer {
    async fn play(&self, source: &SoundSource) -> Result<(), SoundError> {
        if self.disabled {
            return Ok(());
        }

        let handle = self.stream_handle.as_ref().ok_or_else(|| 
            SoundError::DeviceNotAvailable("Output stream not initialized".to_string())
        )?;

        let sink = Sink::try_new(handle).map_err(|e| 
            SoundError::StreamError(e.to_string())
        )?;

        match source {
            SoundSource::System { path, .. } => {
                let file = std::fs::File::open(path).map_err(|e| 
                    SoundError::FileNotFound(e.to_string())
                )?;
                let decoder = Decoder::new(std::io::BufReader::new(file)).map_err(|e| 
                    SoundError::DecodeError(e.to_string())
                )?;
                sink.append(decoder);
            },
            SoundSource::Embedded { .. } => {
                // Using embedded::DEFAULT_SOUND_DATA
                // Note: If data is invalid (e.g. empty), Decoder::new will fail.
                // We handle this gracefully.
                let cursor = Cursor::new(super::DEFAULT_SOUND_DATA);
                let decoder = Decoder::new(cursor).map_err(|e| 
                    SoundError::DecodeError(e.to_string())
                )?;
                sink.append(decoder);
            }
        }

        sink.detach();
        Ok(())
    }

    fn is_available(&self) -> bool {
        !self.disabled && self.stream_handle.is_some()
    }
}
