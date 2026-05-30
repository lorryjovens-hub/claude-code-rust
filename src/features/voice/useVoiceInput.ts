import { useState, useEffect, useRef, useCallback } from 'react';
import { useChatStore } from '../../stores/useChatStore';

export function useVoiceInput(adjustTextareaHeight: () => void) {
  const setInputText = useChatStore((s) => s.setInputText);

  const [isListening, setIsListening] = useState(false);
  const [speechSupported, setSpeechSupported] = useState(false);
  const [showVoicePanel, setShowVoicePanel] = useState(false);
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    const SpeechRecognition =
      (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (SpeechRecognition) {
      setSpeechSupported(true);
      const recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang = 'auto';
      recognition.onresult = (event: any) => {
        let transcript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          transcript += event.results[i][0].transcript;
        }
        if (transcript) {
          setInputText((prev) => {
            const base = prev.endsWith(' ') || prev === '' ? prev : prev + ' ';
            return base + transcript;
          });
        }
      };
      recognition.onerror = () => {
        setIsListening(false);
      };
      recognition.onend = () => {
        setIsListening(false);
      };
      recognitionRef.current = recognition;
    }
  }, [setInputText]);

  const toggleVoiceInput = useCallback(() => {
    if (!recognitionRef.current) return;
    if (isListening) {
      recognitionRef.current.stop();
      setIsListening(false);
    } else {
      try {
        recognitionRef.current.start();
        setIsListening(true);
      } catch (_) {
        setIsListening(false);
      }
    }
  }, [isListening]);

  const handleVoiceResult = useCallback(
    (text: string) => {
      setInputText((prev) => {
        const base = prev.endsWith(' ') || prev === '' ? prev : prev + ' ';
        return base + text;
      });
      adjustTextareaHeight();
    },
    [setInputText, adjustTextareaHeight],
  );

  const openVoicePanel = useCallback(() => {
    setShowVoicePanel(true);
  }, []);

  const closeVoicePanel = useCallback(() => {
    setShowVoicePanel(false);
  }, []);

  return {
    isListening,
    speechSupported,
    showVoicePanel,
    toggleVoiceInput,
    handleVoiceResult,
    openVoicePanel,
    closeVoicePanel,
  };
}
