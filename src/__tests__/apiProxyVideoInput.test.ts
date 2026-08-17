import { describe, it, expect } from 'vitest';
import { anthropicToOpenAIMessages } from '../utils/apiProxy';
import type { AnthropicMessage, OpenAIContent } from '../utils/apiProxy';

describe('anthropicToOpenAIMessages video input', () => {
  it('keeps a video part when it is the only attachment', () => {
    const messages = [
      {
        role: 'user',
        content: [
          { type: 'video_url', video_url: { url: 'data:video/mp4;base64,AAAA' } },
          { type: 'text', text: 'What happens in this clip?' },
        ],
      },
    ] as unknown as AnthropicMessage[];

    const converted = anthropicToOpenAIMessages(messages);

    expect(converted).toHaveLength(1);
    const parts = converted[0].content as OpenAIContent[];
    expect(Array.isArray(parts)).toBe(true);
    expect(parts).toHaveLength(2);
    expect(parts[0]).toEqual({ type: 'text', text: 'What happens in this clip?' });
    expect(parts[1]).toEqual({
      type: 'video_url',
      video_url: { url: 'data:video/mp4;base64,AAAA' },
    });
  });

  it('forwards the optional detail hint when present', () => {
    const messages = [
      {
        role: 'user',
        content: [
          { type: 'video_url', video_url: { url: 'https://example.com/clip.mp4', detail: 'low' } },
        ],
      },
    ] as unknown as AnthropicMessage[];

    const parts = anthropicToOpenAIMessages(messages)[0].content as OpenAIContent[];

    expect(parts[0].video_url).toEqual({ url: 'https://example.com/clip.mp4', detail: 'low' });
  });

  it('keeps images and videos together in block order', () => {
    const messages = [
      {
        role: 'user',
        content: [
          {
            type: 'image',
            source: { type: 'base64', media_type: 'image/png', data: 'BBBB' },
          },
          { type: 'video_url', video_url: { url: 'data:video/webm;base64,CCCC' } },
        ],
      },
    ] as unknown as AnthropicMessage[];

    const parts = anthropicToOpenAIMessages(messages)[0].content as OpenAIContent[];

    expect(parts.map(part => part.type)).toEqual(['image_url', 'video_url']);
  });

  it('still sends a plain string when no media is attached', () => {
    const messages = [
      { role: 'user', content: [{ type: 'text', text: 'hello' }] },
    ] as unknown as AnthropicMessage[];

    expect(anthropicToOpenAIMessages(messages)[0].content).toBe('hello');
  });
});
