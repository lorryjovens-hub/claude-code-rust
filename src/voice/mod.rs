//! 语音交互系统
//! 
//! 这个模块实现了跨平台音频捕获功能，支持：
//! - 原生音频捕获 基于 cpal 库
//! - SoX/arecord 备选方案
//! - 智能静音检测
//! - 语音编程助手功能

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod capture;
pub mod silence;
pub mod transcription;

pub use capture::AudioCapture;
pub use silence::{SilenceDetector, SilenceConfig};
pub use transcription::TranscriptionService;

const RECORDING_SAMPLE_RATE: u32 = 16000;
const RECORDING_CHANNELS: u16 = 1;
const SILENCE_DURATION_SECS: f32 = 2.0;
const SILENCE_THRESHOLD_PERCENT: f32 = 0.03;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceBackend {
    Native,
    Sox,
    Arecord,
    None,
}

impl std::fmt::Display for VoiceBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceBackend::Native => write!(f, "Native (cpal)"),
            VoiceBackend::Sox => write!(f, "SoX"),
            VoiceBackend::Arecord => write!(f, "ALSA (arecord)"),
            VoiceBackend::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub push_to_talk: bool,
    pub silence_detection: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub silence_duration_secs: f32,
    pub silence_threshold: f32,
    pub backend: VoiceBackend,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            push_to_talk: true,
            silence_detection: true,
            sample_rate: RECORDING_SAMPLE_RATE,
            channels: RECORDING_CHANNELS,
            silence_duration_secs: SILENCE_DURATION_SECS,
            silence_threshold: SILENCE_THRESHOLD_PERCENT,
            backend: VoiceBackend::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
    Processing,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceStatus {
    pub available: bool,
    pub backend: VoiceBackend,
    pub state: RecordingState,
    pub duration_secs: f32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordingAvailability {
    pub available: bool,
    pub reason: Option<String>,
}

pub struct VoiceService {
    config: VoiceConfig,
    recording_state: Arc<RwLock<RecordingState>>,
    audio_buffer: Arc<RwLock<Vec<u8>>>,
    start_time: Arc<RwLock<Option<std::time::Instant>>>,
    silence_detector: SilenceDetector,
    audio_capture: Option<AudioCapture>,
}

impl VoiceService {
    pub fn new(config: Option<VoiceConfig>) -> Self {
        let config = config.unwrap_or_default();
        let silence_config = SilenceConfig {
            duration_secs: config.silence_duration_secs,
            threshold: config.silence_threshold,
            sample_rate: config.sample_rate,
        };
        let silence_detector = SilenceDetector::new(Some(silence_config));
        
        Self {
            config,
            recording_state: Arc::new(RwLock::new(RecordingState::Idle)),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            start_time: Arc::new(RwLock::new(None)),
            silence_detector,
            audio_capture: None,
        }
    }

    pub async fn check_dependencies() -> (bool, Vec<String>, Option<String>) {
        let mut missing = Vec::new();
        let mut install_command = None;

        if Self::check_native_audio().await {
            return (true, missing, None);
        }

        #[cfg(target_os = "windows")]
        {
            missing.push("Voice mode requires the native audio module (not loaded)".to_string());
            return (false, missing, None);
        }

        #[cfg(target_os = "linux")]
        {
            if Self::check_arecord_available().await {
                return (true, missing, None);
            }
        }

        if !Self::check_sox_available().await {
            missing.push("sox (rec command)".to_string());
            install_command = Self::get_install_command();
        }

        (missing.is_empty(), missing, install_command)
    }

    async fn check_native_audio() -> bool {
        #[cfg(all(feature = "audio", any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            return AudioCapture::is_available().await;
        }
        #[cfg(not(all(feature = "audio", any(target_os = "macos", target_os = "linux", target_os = "windows"))))]
        false
    }

    #[cfg(target_os = "linux")]
    async fn check_arecord_available() -> bool {
        if !Self::has_command("arecord").await {
            return false;
        }
        Self::probe_arecord().await
    }

    #[cfg(target_os = "linux")]
    async fn probe_arecord() -> bool {
        let result = tokio::process::Command::new("arecord")
            .args([
                "-f", "S16_LE",
                "-r", &RECORDING_SAMPLE_RATE.to_string(),
                "-c", &RECORDING_CHANNELS.to_string(),
                "-t", "raw",
                "/dev/null",
            ])
            .output()
            .await;
        
        result.is_ok()
    }

    async fn check_sox_available() -> bool {
        Self::has_command("rec").await
    }

    async fn has_command(cmd: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            let result = tokio::process::Command::new("where")
                .arg(cmd)
                .output()
                .await;
            result.is_ok()
        }
        #[cfg(not(target_os = "windows"))]
        {
            let result = tokio::process::Command::new("which")
                .arg(cmd)
                .output()
                .await;
            result.is_ok()
        }
    }

    fn get_install_command() -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            if Self::has_command_sync("brew") {
                return Some("brew install sox".to_string());
            }
        }

        #[cfg(target_os = "linux")]
        {
            if Self::has_command_sync("apt-get") {
                return Some("sudo apt-get install sox".to_string());
            }
            if Self::has_command_sync("dnf") {
                return Some("sudo dnf install sox".to_string());
            }
            if Self::has_command_sync("pacman") {
                return Some("sudo pacman -S sox".to_string());
            }
        }

        None
    }

    fn has_command_sync(cmd: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("where")
                .arg(cmd)
                .output()
                .is_ok()
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .is_ok()
        }
    }

    pub async fn check_recording_availability() -> RecordingAvailability {
        if Self::check_native_audio().await {
            return RecordingAvailability {
                available: true,
                reason: None,
            };
        }

        #[cfg(target_os = "windows")]
        {
            return RecordingAvailability {
                available: false,
                reason: Some("Voice recording requires the native audio module, which could not be loaded.".to_string()),
            };
        }

        #[cfg(target_os = "linux")]
        {
            if Self::check_arecord_available().await {
                return RecordingAvailability {
                    available: true,
                    reason: None,
                };
            }
        }

        if !Self::check_sox_available().await {
            let reason = Self::get_install_command()
                .map(|cmd| format!("Voice mode requires SoX for audio recording. Install it with: {}", cmd))
                .unwrap_or_else(|| {
                    "Voice mode requires SoX for audio recording. Install SoX manually:\n  macOS: brew install sox\n  Ubuntu/Debian: sudo apt-get install sox\n  Fedora: sudo dnf install sox".to_string()
                });
            
            return RecordingAvailability {
                available: false,
                reason: Some(reason),
            };
        }

        RecordingAvailability {
            available: true,
            reason: None,
        }
    }

    pub async fn request_microphone_permission(&self) -> bool {
        if !Self::check_native_audio().await {
            return true;
        }

        let started = self.start_recording_internal(false).await;
        if started {
            self.stop_recording_internal().await;
        }
        started
    }

    pub async fn start_recording(&self) -> crate::error::Result<()> {
        self.start_recording_internal(self.config.silence_detection).await;
        Ok(())
    }

    async fn start_recording_internal(&self, silence_detection: bool) -> bool {
        let mut state = self.recording_state.write().await;
        
        if *state != RecordingState::Idle {
            return false;
        }

        let backend = self.detect_backend().await;
        if backend == VoiceBackend::None {
            return false;
        }

        *state = RecordingState::Recording;
        let mut buffer = self.audio_buffer.write().await;
        buffer.clear();
        let mut start_time = self.start_time.write().await;
        *start_time = Some(std::time::Instant::now());

        true
    }

    pub async fn stop_recording(&self) -> crate::error::Result<Vec<u8>> {
        self.stop_recording_internal().await;
        let buffer = self.audio_buffer.read().await;
        Ok(buffer.clone())
    }

    async fn stop_recording_internal(&self) {
        let mut state = self.recording_state.write().await;
        *state = RecordingState::Idle;
    }

    async fn detect_backend(&self) -> VoiceBackend {
        if Self::check_native_audio().await {
            return VoiceBackend::Native;
        }

        #[cfg(target_os = "linux")]
        {
            if Self::check_arecord_available().await {
                return VoiceBackend::Arecord;
            }
        }

        if Self::check_sox_available().await {
            return VoiceBackend::Sox;
        }

        VoiceBackend::None
    }

    pub async fn capture_voice_input(&self) -> crate::error::Result<Vec<u8>> {
        self.start_recording().await?;
        
        if self.config.silence_detection {
            self.wait_for_silence().await;
        }
        
        self.stop_recording().await
    }

    async fn wait_for_silence(&self) {
        let silence_duration = std::time::Duration::from_secs_f32(self.config.silence_duration_secs);
        tokio::time::sleep(silence_duration).await;
    }

    pub async fn get_status(&self) -> VoiceStatus {
        let backend = self.detect_backend().await;
        let state = self.recording_state.read().await;
        let start_time = self.start_time.read().await;
        
        let duration = if *state == RecordingState::Recording {
            start_time.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0)
        } else {
            0.0
        };

        VoiceStatus {
            available: backend != VoiceBackend::None,
            backend,
            state: state.clone(),
            duration_secs: duration,
            error: None,
        }
    }

    pub async fn push_to_talk_start(&self) -> crate::error::Result<()> {
        self.start_recording().await
    }

    pub async fn push_to_talk_stop(&self) -> crate::error::Result<Vec<u8>> {
        self.stop_recording().await
    }
}

impl std::fmt::Debug for VoiceService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert!(config.push_to_talk);
        assert!(config.silence_detection);
    }

    #[tokio::test]
    async fn test_voice_service_creation() {
        let service = VoiceService::new(None);
        let status = service.get_status().await;
        assert_eq!(status.state, RecordingState::Idle);
    }

    #[tokio::test]
    async fn test_check_dependencies() {
        let (available, missing, _install_cmd) = VoiceService::check_dependencies().await;
        println!("Available: {}, Missing: {:?}", available, missing);
    }
}
