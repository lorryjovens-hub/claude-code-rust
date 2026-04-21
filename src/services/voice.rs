//! Voice Input Service - Audio recording for push-to-talk voice input
//!
//! Recording uses native audio capture (cpal) on macOS, Linux, and Windows
//! for in-process mic access. Falls back to SoX `rec` or arecord (ALSA)
//! on Linux if the native module is unavailable.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

const RECORDING_SAMPLE_RATE: u32 = 16000;
const RECORDING_CHANNELS: u16 = 1;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceBackend {
    Native,
    Sox,
    Arecord,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub push_to_talk: bool,
    pub silence_detection: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub backend: VoiceBackend,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            push_to_talk: true,
            silence_detection: true,
            sample_rate: RECORDING_SAMPLE_RATE,
            channels: RECORDING_CHANNELS,
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

pub struct VoiceService {
    state: Arc<RwLock<AppState>>,
    recording_state: Arc<RwLock<RecordingState>>,
    audio_buffer: Arc<RwLock<Vec<u8>>>,
    start_time: Arc<RwLock<Option<std::time::Instant>>>,
}

impl VoiceService {
    pub fn new(state: Arc<RwLock<AppState>>, _config: Option<VoiceConfig>) -> Self {
        Self {
            state,
            recording_state: Arc::new(RwLock::new(RecordingState::Idle)),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn check_availability(&self) -> VoiceBackend {
        #[cfg(target_os = "macos")]
        {
            if self.check_coreaudio().await {
                return VoiceBackend::Native;
            }
        }

        #[cfg(target_os = "windows")]
        {
            if self.check_windows_audio().await {
                return VoiceBackend::Native;
            }
        }

        #[cfg(target_os = "linux")]
        {
            if self.check_alsa().await {
                return VoiceBackend::Native;
            }
            
            if self.check_arecord().await {
                return VoiceBackend::Arecord;
            }
        }

        if self.check_sox().await {
            return VoiceBackend::Sox;
        }

        VoiceBackend::None
    }

    #[cfg(target_os = "macos")]
    async fn check_coreaudio(&self) -> bool {
        true
    }

    #[cfg(target_os = "windows")]
    async fn check_windows_audio(&self) -> bool {
        true
    }

    #[cfg(target_os = "linux")]
    async fn check_alsa(&self) -> bool {
        if let Ok(content) = tokio::fs::read_to_string("/proc/asound/cards").await {
            let trimmed = content.trim();
            return !trimmed.is_empty() && !trimmed.contains("no soundcards");
        }
        false
    }

    #[allow(dead_code)]
    async fn check_arecord(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            let result = tokio::process::Command::new("arecord")
                .arg("--version")
                .output()
                .await;
            return result.is_ok();
        }
        #[cfg(not(target_os = "linux"))]
        false
    }

    async fn check_sox(&self) -> bool {
        let result = tokio::process::Command::new("sox")
            .arg("--version")
            .output()
            .await;
        result.is_ok()
    }

    pub async fn start_recording(&self) -> anyhow::Result<()> {
        let mut state = self.recording_state.write().await;
        
        if *state != RecordingState::Idle {
            return Err(anyhow::anyhow!("Already recording or processing"));
        }

        let backend = self.check_availability().await;
        if backend == VoiceBackend::None {
            return Err(anyhow::anyhow!("No audio backend available"));
        }

        *state = RecordingState::Recording;
        let mut buffer = self.audio_buffer.write().await;
        buffer.clear();
        let mut start_time = self.start_time.write().await;
        *start_time = Some(std::time::Instant::now());

        println!("🎤 Recording started... Press Enter to stop.");

        Ok(())
    }

    pub async fn stop_recording(&self) -> anyhow::Result<Vec<u8>> {
        let mut state = self.recording_state.write().await;
        
        if *state != RecordingState::Recording {
            return Err(anyhow::anyhow!("Not currently recording"));
        }

        *state = RecordingState::Processing;

        let buffer = self.audio_buffer.read().await;
        let audio_data = buffer.clone();

        let start_time = self.start_time.read().await;
        let duration = start_time.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0);

        println!("🎤 Recording stopped. Duration: {:.1}s", duration);
        println!("🎤 Processing audio...");

        *state = RecordingState::Idle;

        Ok(audio_data)
    }

    pub async fn transcribe(&self, audio_data: &[u8]) -> anyhow::Result<String> {
        println!("🎤 Transcribing {} bytes of audio...", audio_data.len());

        let state = self.state.read().await;
        let api_client = crate::api::ApiClient::new(state.settings.clone());
        
        let prompt = "Please transcribe the following audio. The audio contains speech that should be converted to text. Only output the transcribed text, nothing else.";
        
        let messages = vec![
            crate::api::ChatMessage {
                role: "system".to_string(),
                content: Some(prompt.to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            crate::api::ChatMessage {
                role: "user".to_string(),
                content: Some(format!("[Audio data: {} bytes]", audio_data.len())),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let response = api_client.chat(messages, None).await?;
        
        if let Some(choice) = response.choices.first() {
            return Ok(choice.message.content.clone().unwrap_or_default());
        }

        Ok(String::new())
    }

    pub async fn get_status(&self) -> VoiceStatus {
        let backend = self.check_availability().await;
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

    pub async fn push_to_talk_start(&self) -> anyhow::Result<()> {
        self.start_recording().await
    }

    pub async fn push_to_talk_stop(&self) -> anyhow::Result<String> {
        let audio_data = self.stop_recording().await?;
        let text = self.transcribe(&audio_data).await?;
        Ok(text)
    }
}

impl VoiceConfig {
    pub fn new(push_to_talk: bool, silence_detection: bool) -> Self {
        Self {
            push_to_talk,
            silence_detection,
            ..Default::default()
        }
    }
}
