//! 静音检测模块
//! 
//! 实现2秒静音自动停止的智能检测算法

use std::sync::Arc;
use tokio::sync::RwLock;

const DEFAULT_SILENCE_DURATION_SECS: f32 = 2.0;
const DEFAULT_SILENCE_THRESHOLD: f32 = 0.03;
const SAMPLE_RATE: u32 = 16000;

#[derive(Debug, Clone)]
pub struct SilenceConfig {
    pub duration_secs: f32,
    pub threshold: f32,
    pub sample_rate: u32,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            duration_secs: DEFAULT_SILENCE_DURATION_SECS,
            threshold: DEFAULT_SILENCE_THRESHOLD,
            sample_rate: SAMPLE_RATE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilenceState {
    Listening,
    SilenceDetected,
    SpeechDetected,
}

pub struct SilenceDetector {
    config: SilenceConfig,
    state: Arc<RwLock<SilenceState>>,
    silence_start: Arc<RwLock<Option<std::time::Instant>>>,
    rms_buffer: Arc<RwLock<Vec<f32>>>,
}

impl SilenceDetector {
    pub fn new(config: Option<SilenceConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            state: Arc::new(RwLock::new(SilenceState::Listening)),
            silence_start: Arc::new(RwLock::new(None)),
            rms_buffer: Arc::new(RwLock::new(Vec::with_capacity(1024))),
        }
    }

    pub fn config(&self) -> &SilenceConfig {
        &self.config
    }

    pub async fn process_audio(&self, audio_data: &[u8]) -> bool {
        let rms = self.calculate_rms(audio_data);
        
        let threshold = (self.config.threshold * 32767.0) as f32;
        let is_silent = rms < threshold;

        let mut state = self.state.write().await;
        let mut silence_start = self.silence_start.write().await;

        if is_silent {
            if silence_start.is_none() {
                *silence_start = Some(std::time::Instant::now());
            }
            
            let silence_duration = silence_start.unwrap().elapsed().as_secs_f32();
            if silence_duration >= self.config.duration_secs {
                *state = SilenceState::SilenceDetected;
                return true;
            }
        } else {
            *silence_start = None;
            *state = SilenceState::SpeechDetected;
        }

        false
    }

    fn calculate_rms(&self, audio_data: &[u8]) -> f32 {
        if audio_data.is_empty() {
            return 0.0;
        }

        let samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f64 = samples.iter()
            .map(|&s| (s as f64).powi(2))
            .sum();

        (sum_squares / samples.len() as f64).sqrt() as f32
    }

    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = SilenceState::Listening;
        let mut silence_start = self.silence_start.write().await;
        *silence_start = None;
        let mut buffer = self.rms_buffer.write().await;
        buffer.clear();
    }

    pub async fn get_state(&self) -> SilenceState {
        *self.state.read().await
    }

    pub async fn get_current_rms(&self) -> f32 {
        let buffer = self.rms_buffer.read().await;
        buffer.last().copied().unwrap_or(0.0)
    }
}

impl std::fmt::Debug for SilenceDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SilenceDetector")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_silence_detector_creation() {
        let detector = SilenceDetector::new(None);
        assert_eq!(detector.get_state().await, SilenceState::Listening);
    }

    #[test]
    fn test_silence_config_default() {
        let config = SilenceConfig::default();
        assert_eq!(config.duration_secs, 2.0);
        assert_eq!(config.threshold, 0.03);
    }

    #[tokio::test]
    async fn test_silence_detection() {
        let detector = SilenceDetector::new(None);
        
        let silence_data = vec![0u8; 32000];
        let detected = detector.process_audio(&silence_data).await;
        
        assert!(!detected);
    }
}
