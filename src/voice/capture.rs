//! 音频捕获模块
//! 
//! 实现跨平台音频捕获功能

use std::sync::Arc;
use tokio::sync::RwLock;

const SAMPLE_RATE: u32 = 16000;
const CHANNELS: u16 = 1;
const BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            buffer_size: BUFFER_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState {
    Idle,
    Capturing,
    Error,
}

pub struct AudioCapture {
    config: AudioConfig,
    state: Arc<RwLock<CaptureState>>,
    buffer: Arc<RwLock<Vec<u8>>>,
}

impl AudioCapture {
    pub fn new(config: Option<AudioConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state: Arc::new(RwLock::new(CaptureState::Idle)),
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn is_available() -> bool {
        #[cfg(all(feature = "audio", any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            use cpal::traits::{DeviceTrait, HostTrait};
            
            let host = cpal::default_host();
            if let Ok(input_device) = host.default_input_device() {
                if let Ok(_supported) = input_device.supported_input_formats() {
                    return true;
                }
            }
            false
        }
        #[cfg(not(all(feature = "audio", any(target_os = "macos", target_os = "linux", target_os = "windows"))))]
        false
    }

    pub async fn start_capture(&self) -> crate::error::Result<()> {
        let mut state = self.state.write().await;
        if *state == CaptureState::Capturing {
            return Err(crate::error::ClaudeError::Other("Already capturing".to_string()));
        }

        *state = CaptureState::Capturing;
        self.buffer.write().await.clear();

        Ok(())
    }

    pub async fn stop_capture(&self) -> crate::error::Result<Vec<u8>> {
        let mut state = self.state.write().await;
        *state = CaptureState::Idle;

        let buffer = self.buffer.read().await;
        Ok(buffer.clone())
    }

    pub async fn get_state(&self) -> CaptureState {
        *self.state.read().await
    }

    pub async fn read_audio(&self) -> Vec<u8> {
        self.buffer.read().await.clone()
    }

    pub fn config(&self) -> &AudioConfig {
        &self.config
    }
}

impl std::fmt::Debug for AudioCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioCapture")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_capture_creation() {
        let capture = AudioCapture::new(None);
        assert_eq!(capture.get_state().await, CaptureState::Idle);
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }
}
