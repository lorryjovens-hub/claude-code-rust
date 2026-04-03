//! 语音交互模式 (VOICE_MODE)
//! 
//! 这个模块实现了语音输入和语音输出功能，支持用户通过语音与 Claude Code 交互。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

/// 语音输入状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceInputState {
    /// 空闲状态
    Idle,
    
    /// 正在监听
    Listening,
    
    /// 正在处理
    Processing,
    
    /// 已识别
    Recognized,
    
    /// 错误状态
    Error,
}

/// 语音输出状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceOutputState {
    /// 空闲
    Idle,
    
    /// 正在合成
    Synthesizing,
    
    /// 正在播放
    Playing,
    
    /// 播放完成
    Completed,
    
    /// 错误状态
    Error,
}

/// 语音配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// 是否启用语音输入
    pub voice_input_enabled: bool,
    
    /// 是否启用语音输出
    pub voice_output_enabled: bool,
    
    /// 语音输入语言
    pub input_language: String,
    
    /// 语音输出语言
    pub output_language: String,
    
    /// 语音速度 (0.5-2.0)
    pub speech_rate: f32,
    
    /// 语音音量 (0.0-1.0)
    pub volume: f32,
    
    /// 所选语音
    pub selected_voice: Option<String>,
    
    /// 静音检测阈值 (秒)
    pub silence_threshold: f32,
    
    /// 是否使用唤醒词
    pub use_wake_word: bool,
    
    /// 唤醒词
    pub wake_word: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            voice_input_enabled: false,
            voice_output_enabled: false,
            input_language: "zh-CN".to_string(),
            output_language: "zh-CN".to_string(),
            speech_rate: 1.0,
            volume: 1.0,
            selected_voice: None,
            silence_threshold: 2.0,
            use_wake_word: false,
            wake_word: "Hey Claude".to_string(),
        }
    }
}

/// 语音识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRecognitionResult {
    /// 识别的文本
    pub text: String,
    
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    
    /// 是否为最终结果
    pub is_final: bool,
    
    /// 识别时间戳
    pub timestamp: String,
    
    /// 备选文本
    pub alternatives: Vec<String>,
}

/// 语音合成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSynthesisRequest {
    /// 要合成的文本
    pub text: String,
    
    /// 语言代码
    pub language: String,
    
    /// 语音名称
    pub voice: Option<String>,
    
    /// 语速
    pub rate: Option<f32>,
    
    /// 音量
    pub volume: Option<f32>,
    
    /// 音调
    pub pitch: Option<f32>,
}

/// 语音管理器
#[derive(Debug)]
pub struct VoiceManager {
    /// 应用状态
    state: AppState,
    
    /// 语音配置
    config: VoiceConfig,
    
    /// 输入状态
    input_state: VoiceInputState,
    
    /// 输出状态
    output_state: VoiceOutputState,
    
    /// 最近的识别结果
    recent_results: Vec<SpeechRecognitionResult>,
    
    /// 语音历史记录
    voice_history: Vec<VoiceInteraction>,
    
    /// 可用的语音列表
    available_voices: Vec<VoiceInfo>,
}

/// 语音信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// 语音名称
    pub name: String,
    
    /// 语言代码
    pub language: String,
    
    /// 性别
    pub gender: Option<String>,
    
    /// 描述
    pub description: Option<String>,
}

/// 语音交互记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInteraction {
    /// 交互类型
    pub interaction_type: VoiceInteractionType,
    
    /// 文本内容
    pub text: String,
    
    /// 时间戳
    pub timestamp: String,
    
    /// 持续时间（毫秒）
    pub duration_ms: u64,
}

/// 语音交互类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceInteractionType {
    /// 用户语音输入
    UserInput,
    
    /// AI 语音输出
    AiOutput,
}

impl VoiceManager {
    /// 创建新的语音管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            config: VoiceConfig::default(),
            input_state: VoiceInputState::Idle,
            output_state: VoiceOutputState::Idle,
            recent_results: Vec::new(),
            voice_history: Vec::new(),
            available_voices: Vec::new(),
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &VoiceConfig {
        &self.config
    }
    
    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut VoiceConfig {
        &mut self.config
    }
    
    /// 获取输入状态
    pub fn input_state(&self) -> VoiceInputState {
        self.input_state
    }
    
    /// 获取输出状态
    pub fn output_state(&self) -> VoiceOutputState {
        self.output_state
    }
    
    /// 开始监听
    pub async fn start_listening(&mut self) -> Result<()> {
        if !self.config.voice_input_enabled {
            return Err("Voice input is not enabled".into());
        }
        
        self.input_state = VoiceInputState::Listening;
        
        // TODO: 实现实际的语音监听
        // 这应该使用系统音频 API 或第三方语音识别服务
        
        Ok(())
    }
    
    /// 停止监听
    pub async fn stop_listening(&mut self) -> Result<Option<SpeechRecognitionResult>> {
        self.input_state = VoiceInputState::Processing;
        
        // TODO: 实现停止监听并获取结果
        
        self.input_state = VoiceInputState::Idle;
        Ok(None)
    }
    
    /// 合成语音
    pub async fn synthesize_speech(&mut self, _request: SpeechSynthesisRequest) -> Result<()> {
        if !self.config.voice_output_enabled {
            return Err("Voice output is not enabled".into());
        }
        
        self.output_state = VoiceOutputState::Synthesizing;
        
        // TODO: 实现实际的语音合成
        
        Ok(())
    }
    
    /// 播放合成的语音
    pub async fn play_speech(&mut self) -> Result<()> {
        self.output_state = VoiceOutputState::Playing;
        
        // TODO: 实现语音播放
        
        self.output_state = VoiceOutputState::Completed;
        Ok(())
    }
    
    /// 停止播放
    pub async fn stop_speech(&mut self) -> Result<()> {
        self.output_state = VoiceOutputState::Idle;
        
        // TODO: 实现停止播放
        
        Ok(())
    }
    
    /// 添加识别结果
    pub fn add_recognition_result(&mut self, result: SpeechRecognitionResult) {
        self.recent_results.push(result.clone());
        
        // 只保留最近的 10 个结果
        if self.recent_results.len() > 10 {
            self.recent_results.remove(0);
        }
        
        if result.is_final {
            self.voice_history.push(VoiceInteraction {
                interaction_type: VoiceInteractionType::UserInput,
                text: result.text.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms: 0,
            });
        }
    }
    
    /// 记录语音输出
    pub fn record_voice_output(&mut self, text: String, duration_ms: u64) {
        self.voice_history.push(VoiceInteraction {
            interaction_type: VoiceInteractionType::AiOutput,
            text,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
        });
    }
    
    /// 获取最近的识别结果
    pub fn recent_results(&self) -> &[SpeechRecognitionResult] {
        &self.recent_results
    }
    
    /// 获取语音历史
    pub fn voice_history(&self) -> &[VoiceInteraction] {
        &self.voice_history
    }
    
    /// 设置可用语音
    pub fn set_available_voices(&mut self, voices: Vec<VoiceInfo>) {
        self.available_voices = voices;
    }
    
    /// 获取可用语音
    pub fn available_voices(&self) -> &[VoiceInfo] {
        &self.available_voices
    }
    
    /// 选择语音
    pub fn select_voice(&mut self, voice_name: &str) -> bool {
        if self.available_voices.iter().any(|v| v.name == voice_name) {
            self.config.selected_voice = Some(voice_name.to_string());
            true
        } else {
            false
        }
    }
    
    /// 设置输入语言
    pub fn set_input_language(&mut self, language: String) {
        self.config.input_language = language;
    }
    
    /// 设置输出语言
    pub fn set_output_language(&mut self, language: String) {
        self.config.output_language = language;
    }
    
    /// 设置语速
    pub fn set_speech_rate(&mut self, rate: f32) {
        self.config.speech_rate = rate.clamp(0.5, 2.0);
    }
    
    /// 设置音量
    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 1.0);
    }
    
    /// 启用/禁用语音输入
    pub fn set_voice_input(&mut self, enabled: bool) {
        self.config.voice_input_enabled = enabled;
    }
    
    /// 启用/禁用语音输出
    pub fn set_voice_output(&mut self, enabled: bool) {
        self.config.voice_output_enabled = enabled;
    }
}
