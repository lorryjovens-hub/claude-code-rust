import { describe, expect, it } from 'vitest';
import { detectProvider } from '../components/ProviderSettings';

describe('MiniMax provider preset', () => {
  it('recognizes the documented regional endpoints', () => {
    const globalOpenAI = detectProvider('https://api.minimax.io/v1');
    const chinaOpenAI = detectProvider('https://api.minimaxi.com/v1');
    const globalAnthropic = detectProvider('https://api.minimax.io/anthropic');
    const chinaAnthropic = detectProvider('https://api.minimaxi.com/anthropic');

    expect(globalOpenAI?.name).toBe('MiniMax');
    expect(chinaOpenAI?.name).toBe('MiniMax');
    expect(globalOpenAI?.format).toBe('openai');
    expect(chinaOpenAI?.format).toBe('openai');
    expect(globalAnthropic?.format).toBe('anthropic');
    expect(chinaAnthropic?.format).toBe('anthropic');
    expect(detectProvider('https://api.minimax.io/anthropic/v1')?.format).toBe('anthropic');
    expect(detectProvider('https://gateway.example/minimax')).toBeNull();
    expect(globalOpenAI?.endpoints).toEqual([
      { region: 'global_en', format: 'openai', url: 'https://api.minimax.io/v1' },
      { region: 'cn_zh', format: 'openai', url: 'https://api.minimaxi.com/v1' },
      { region: 'global_en', format: 'anthropic', url: 'https://api.minimax.io/anthropic' },
      { region: 'cn_zh', format: 'anthropic', url: 'https://api.minimaxi.com/anthropic' },
    ]);
  });

  it('includes the current model parameters and input capabilities', () => {
    const preset = detectProvider('https://api.minimax.io/v1');

    expect(preset?.defaultModels).toEqual([
      {
        id: 'MiniMax-M3.1-Flash',
        name: 'MiniMax M3.1 Flash',
        contextWindow: 1_000_000,
        maxTokens: 524_288,
        pricingUsdPerMillionTokens: { input: 0.15, output: 0.6, cacheRead: 0.015, cacheWrite: null },
        inputModalities: ['text', 'image', 'video'],
        thinking: ['adaptive'],
      },
      {
        id: 'MiniMax-M3',
        name: 'MiniMax M3',
        contextWindow: 1_000_000,
        inputModalities: ['text', 'image', 'video'],
        thinking: ['adaptive', 'disabled'],
      },
    ]);
  });
});
