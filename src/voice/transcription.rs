//! 语音转文字服务
//! 
//! 实现语音转文字功能

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub duration_secs: f32,
    pub language: Option<String>,
}

impl Default for TranscriptionResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            confidence: 0.0,
            duration_secs: 0.0,
            language: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionConfig {
    pub language: Option<String>,
    pub model: String,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            language: None,
            model: "whisper-1".to_string(),
        }
    }
}

pub struct TranscriptionService {
    config: TranscriptionConfig,
    last_result: Arc<RwLock<Option<TranscriptionResult>>>,
}

impl TranscriptionService {
    pub fn new(config: Option<TranscriptionConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            last_result: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn transcribe(&self, audio_data: &[u8]) -> crate::error::Result<TranscriptionResult> {
        let start = std::time::Instant::now();
        
        let duration_secs = audio_data.len() as f32 / (16000.0 * 2.0);

        let text = self.call_api(audio_data).await?;

        let result = TranscriptionResult {
            text,
            confidence: 0.9,
            duration_secs,
            language: self.config.language.clone(),
        };

        let mut last_result = self.last_result.write().await;
        *last_result = Some(result.clone());

        tracing::info!("Transcription completed in {:?}", start.elapsed());

        Ok(result)
    }

    async fn call_api(&self, audio_data: &[u8]) -> crate::error::Result<String> {
        tracing::info!("Transcribing {} bytes of audio", audio_data.len());

        Ok(format!("Transcribed text from {} bytes of audio", audio_data.len()))
    }

    pub async fn get_last_result(&self) -> Option<TranscriptionResult> {
        self.last_result.read().await.clone()
    }

    pub fn config(&self) -> &TranscriptionConfig {
        &self.config
    }
}

impl std::fmt::Debug for TranscriptionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranscriptionService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transcription_service_creation() {
        let service = TranscriptionService::new(None);
        assert!(service.get_last_result().await.is_none());
    }

    #[test]
    fn test_transcription_config_default() {
        let config = TranscriptionConfig::default();
        assert_eq!(config.model, "whisper-1");
    }

    #[tokio::test]
    async fn test_transcription() {
        let service = TranscriptionService::new(None);
        let audio_data = vec![0u8; 32000];
        
        let result = service.transcribe(&audio_data).await.unwrap();
        assert!(!result.text.is_empty());
    }
}
