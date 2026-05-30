import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Mic, X, Loader2, Volume2 } from 'lucide-react';

export type VoiceInputStatus = 'idle' | 'listening' | 'processing' | 'done' | 'error';

interface VoiceInputProps {
  onResult: (text: string) => void;
  onClose: () => void;
  isOpen: boolean;
}

const VoiceInput: React.FC<VoiceInputProps> = ({ onResult, onClose, isOpen }) => {
  const [status, setStatus] = useState<VoiceInputStatus>('idle');
  const [transcript, setTranscript] = useState('');
  const [interimTranscript, setInterimTranscript] = useState('');
  const [duration, setDuration] = useState(0);
  const [isSupported, setIsSupported] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);

  const recognitionRef = useRef<any>(null);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const durationTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const animationFrameRef = useRef<number | null>(null);
  const transcriptRef = useRef(transcript);
  transcriptRef.current = transcript;

  // 检查浏览器支持
  useEffect(() => {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    setIsSupported(!!SpeechRecognition);
  }, []);

  // 初始化 SpeechRecognition
  useEffect(() => {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    const recognition = new SpeechRecognition();
    recognition.continuous = true;
    recognition.interimResults = true;
    // 自动检测语言（先尝试中文）
    recognition.lang = 'zh-CN';

    recognition.onresult = (event: any) => {
      let finalTranscript = '';
      let interim = '';

      for (let i = 0; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          finalTranscript += result[0].transcript;
        } else {
          interim += result[0].transcript;
        }
      }

      if (finalTranscript) {
        setTranscript(prev => prev + finalTranscript);
      }
      setInterimTranscript(interim);
    };

    recognition.onerror = (event: any) => {
      console.error('Speech recognition error:', event.error);
      if (event.error === 'no-speech') {
        setStatus('idle');
      } else if (event.error === 'not-allowed') {
        setStatus('error');
      } else {
        setStatus('error');
      }
    };

    recognition.onend = () => {
      // 如果还在录音状态，自动重启（实现连续录音）
      if (status === 'listening') {
        try {
          recognition.start();
        } catch (_) {}
      } else {
        setStatus('done');
        stopDurationTimer();
        cancelAnimation();
        setAudioLevel(0);
      }
    };

    recognitionRef.current = recognition;

    return () => {
      if (recognitionRef.current) {
        try {
          recognitionRef.current.abort();
        } catch (_) {}
      }
    };
  }, []);

  // 音频级别监控
  const startAudioMonitoring = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const source = audioContext.createMediaStreamSource(stream);
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);

      audioContextRef.current = audioContext;
      analyserRef.current = analyser;
      mediaRecorderRef.current = new MediaRecorder(stream);
      mediaRecorderRef.current.start();

      const updateLevel = () => {
        if (analyserRef.current) {
          const dataArray = new Uint8Array(analyserRef.current.frequencyBinCount);
          analyserRef.current.getByteFrequencyData(dataArray);
          const average = dataArray.reduce((a, b) => a + b, 0) / dataArray.length;
          setAudioLevel(Math.min(100, (average / 128) * 100));
        }
        animationFrameRef.current = requestAnimationFrame(updateLevel);
      };
      updateLevel();
    } catch (err) {
      console.error('Audio monitoring failed:', err);
    }
  }, []);

  const stopAudioMonitoring = useCallback(() => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    if (audioContextRef.current) {
      audioContextRef.current.close();
      audioContextRef.current = null;
    }
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
      mediaRecorderRef.current.stop();
    }
    setAudioLevel(0);
  }, []);

  const startDurationTimer = useCallback(() => {
    setDuration(0);
    durationTimerRef.current = setInterval(() => {
      setDuration(prev => prev + 1);
    }, 1000);
  }, []);

  const stopDurationTimer = useCallback(() => {
    if (durationTimerRef.current) {
      clearInterval(durationTimerRef.current);
      durationTimerRef.current = null;
    }
  }, []);

  const cancelAnimation = useCallback(() => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
  }, []);

  const handleStartListening = useCallback(async () => {
    if (!recognitionRef.current) return;

    setTranscript('');
    setInterimTranscript('');
    setStatus('listening');

    try {
      await startAudioMonitoring();
      startDurationTimer();
      recognitionRef.current.start();
    } catch (err) {
      console.error('Failed to start recognition:', err);
      setStatus('error');
    }
  }, [startAudioMonitoring, startDurationTimer]);

  const handleStopListening = useCallback(() => {
    if (!recognitionRef.current) return;

    setStatus('processing');
    stopAudioMonitoring();
    stopDurationTimer();

    try {
      recognitionRef.current.stop();
    } catch (_) {}
  }, [stopAudioMonitoring, stopDurationTimer]);

  const handleConfirm = useCallback(() => {
    const finalText = transcriptRef.current.trim();
    if (finalText) {
      onResult(finalText);
    }
    handleClose();
  }, [onResult]);

  const handleClose = useCallback(() => {
    setStatus('idle');
    setTranscript('');
    setInterimTranscript('');
    setDuration(0);
    stopAudioMonitoring();
    stopDurationTimer();
    onClose();
  }, [onClose, stopAudioMonitoring, stopDurationTimer]);

  // 格式化时长
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  if (!isOpen) return null;

  if (!isSupported) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
        <div className="w-[420px] bg-claude-input border border-claude-border rounded-2xl shadow-2xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-claude-text">语音输入</h3>
            <button
              onClick={handleClose}
              className="p-1.5 rounded-lg hover:bg-claude-hover transition-colors"
            >
              <X size={18} className="text-claude-textSecondary" />
            </button>
          </div>
          <div className="text-center py-8 text-claude-textSecondary">
            <Volume2 size={48} className="mx-auto mb-4 opacity-50" />
            <p className="text-sm">您的浏览器不支持语音识别功能</p>
            <p className="text-xs mt-2 opacity-70">请使用 Chrome、Edge 或 Safari 浏览器</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="w-[480px] max-h-[80vh] bg-claude-input border border-claude-border rounded-2xl shadow-2xl overflow-hidden">
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-claude-border">
          <h3 className="text-lg font-semibold text-claude-text">语音输入</h3>
          <button
            onClick={handleClose}
            className="p-1.5 rounded-lg hover:bg-claude-hover transition-colors"
          >
            <X size={18} className="text-claude-textSecondary" />
          </button>
        </div>

        <div className="p-6">
          {/* 录音按钮和动画 */}
          <div className="flex flex-col items-center mb-6">
            <div className="relative w-32 h-32 flex items-center justify-center mb-4">
              {/* 声波动画圆环 */}
              {status === 'listening' && (
                <>
                  <div
                    className="absolute inset-0 rounded-full bg-red-500/10 animate-ping"
                    style={{ animationDuration: '2s' }}
                  />
                  <div
                    className="absolute inset-2 rounded-full bg-red-500/20 animate-ping"
                    style={{ animationDuration: '1.5s' }}
                  />
                  {/* 音频级别可视化 */}
                  <div className="absolute inset-4 rounded-full border-4 border-red-500/30 transition-all duration-150"
                    style={{
                      transform: `scale(${0.9 + (audioLevel / 100) * 0.2})`,
                      borderColor: `rgba(239, 68, 68, ${0.3 + (audioLevel / 100) * 0.5})`
                    }}
                  />
                </>
              )}

              {/* 主按钮 */}
              <button
                onClick={status === 'listening' ? handleStopListening : handleStartListening}
                disabled={status === 'processing'}
                className={`relative z-10 w-20 h-20 rounded-full flex items-center justify-center transition-all duration-300 ${
                  status === 'listening'
                    ? 'bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/30'
                    : status === 'processing'
                    ? 'bg-gray-400 cursor-wait'
                    : 'bg-gradient-to-br from-orange-400 to-orange-600 hover:from-orange-500 hover:to-orange-700 text-white shadow-lg shadow-orange-500/30'
                }`}
              >
                {status === 'processing' ? (
                  <Loader2 size={28} className="animate-spin" />
                ) : (
                  <Mic size={28} />
                )}
              </button>
            </div>

            {/* 状态文字和时长 */}
            <div className="text-center">
              {status === 'idle' && (
                <p className="text-sm text-claude-textSecondary">点击麦克风开始录音</p>
              )}
              {status === 'listening' && (
                <>
                  <p className="text-sm font-medium text-red-500 flex items-center gap-2 justify-center">
                    <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                    正在录音...
                  </p>
                  <p className="text-xs text-claude-textSecondary mt-1 font-mono">
                    {formatDuration(duration)}
                  </p>
                </>
              )}
              {status === 'processing' && (
                <p className="text-sm text-claude-textSecondary">正在处理...</p>
              )}
              {status === 'done' && (
                <p className="text-sm text-green-500">识别完成</p>
              )}
              {status === 'error' && (
                <p className="text-sm text-red-500">识别失败，请重试</p>
              )}
            </div>
          </div>

          {/* 识别结果 */}
          {(transcript || interimTranscript) && (
            <div className="mb-4">
              <label className="block text-xs text-claude-textSecondary mb-2">
                识别结果（可编辑）
              </label>
              <textarea
                value={transcript + interimTranscript}
                onChange={(e) => setTranscript(e.target.value)}
                className="w-full h-32 px-4 py-3 bg-claude-bg border border-claude-border rounded-xl text-sm text-claude-text resize-none focus:outline-none focus:border-orange-400 transition-colors"
                placeholder="语音识别结果将显示在这里..."
              />
              {interimTranscript && (
                <p className="text-xs text-claude-textSecondary mt-2 italic">
                  正在识别: "{interimTranscript}"
                </p>
              )}
            </div>
          )}

          {/* 底部按钮 */}
          <div className="flex gap-3">
            <button
              onClick={handleClose}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-claude-textSecondary border border-claude-border rounded-xl hover:bg-claude-hover transition-colors"
            >
              取消
            </button>
            {status === 'done' && transcript.trim() && (
              <button
                onClick={handleConfirm}
                className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-gradient-to-r from-orange-400 to-orange-600 rounded-xl hover:from-orange-500 hover:to-orange-700 transition-all shadow-md"
              >
                确认输入
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default VoiceInput;
