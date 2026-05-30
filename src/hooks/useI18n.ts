import { useCallback, useMemo } from 'react';
import { useUIStore } from '../stores/useUIStore';
import en from '../locales/en.json';
import zh from '../locales/zh.json';

const translations: Record<string, any> = { en, zh };

function getNestedValue(obj: any, path: string): any {
  return path.split('.').reduce((acc, part) => acc && acc[part], obj);
}

export function useI18n() {
  const language = useUIStore((state) => state.language);

  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    const lang = language || 'en';
    const trans = translations[lang] || translations.en;
    let text = getNestedValue(trans, key) || getNestedValue(translations.en, key) || key;
    
    if (params) {
      Object.entries(params).forEach(([k, v]) => {
        text = text.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
      });
    }
    
    return text;
  }, [language]);

  const setLanguage = useCallback((lang: string) => {
    useUIStore.getState().setLanguage(lang);
    localStorage.setItem('app_language', lang);
    document.documentElement.lang = lang;
  }, []);

  return useMemo(() => ({ t, language, setLanguage }), [t, language, setLanguage]);
}
