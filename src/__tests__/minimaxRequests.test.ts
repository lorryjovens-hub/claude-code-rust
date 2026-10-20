import { afterEach, describe, expect, it, vi } from 'vitest';
import { APIProxy, type AnthropicRequest, type ProviderConfig } from '../utils/apiProxy';
import { MINIMAX_MODELS, mergeDiscoveredModels } from '../utils/minimax';

const model = (id: string) => ({ id, name: id, enabled: true });
afterEach(() => vi.unstubAllGlobals());

describe('MiniMax discovery', () => {
  it('puts Flash first and filters obsolete discovered text while retaining saved and nontext models', () => {
    const saved = [{ ...model('MiniMax-M2.7'), enabled: false }, model('my-custom-model')];
    const before = structuredClone(saved);
    const models = mergeDiscoveredModels('https://api.minimax.io/v1', [
      model('MiniMax-M1'), model('MiniMax-Text-01'), model('MiniMax-M3'), model('speech-02-hd'),
    ], saved);
    expect(models.map(m => m.id)).toEqual([
      'MiniMax-M3.1-Flash', 'MiniMax-M3', 'MiniMax-M2.7', 'my-custom-model', 'speech-02-hd',
    ]);
    expect(models[2].enabled).toBe(false);
    expect(saved).toEqual(before);
  });

  it('does not apply direct metadata or filtering to gateways/custom hosts', () => {
    const discovered = [model('MiniMax-M2.7')];
    expect(mergeDiscoveredModels('https://gateway.example/minimax', discovered)).toBe(discovered);
  });
});

for (const host of ['api.minimax.io', 'api.minimaxi.com']) {
  for (const format of ['openai', 'anthropic'] as const) {
    describe(`${host} ${format} serialized requests`, () => {
      const base = `https://${host}/${format === 'openai' ? 'v1' : 'anthropic'}`;
      async function send(id: string, overrides: Partial<AnthropicRequest> = {}, url = base, streaming = false) {
        const fetchMock = vi.fn().mockResolvedValue(new Response('{}', { status: 200 }));
        vi.stubGlobal('fetch', fetchMock);
        const proxy = new APIProxy();
        const provider: ProviderConfig = {
          id: 'fixture', name: 'Fixture', base_url: url, api_key: 'test-only',
          api_format: format, enabled: true, models: [model(id)],
        };
        proxy.setProviders([provider]);
        const request: AnthropicRequest = {
          model: id, messages: [{ role: 'user', content: 'Hello' }], max_tokens: 1_000_000,
          ...overrides,
        };
        const before = structuredClone(request);
        if (streaming) await proxy.chatStream(request);
        else await proxy.chat(request);
        expect(request).toEqual(before);
        const [endpoint, init] = fetchMock.mock.calls[0];
        return { endpoint, body: JSON.parse(init.body) };
      }

      it('sends the first preset with supported defaults at the HTTP boundary', async () => {
        const { endpoint, body } = await send(MINIMAX_MODELS[0].id);
        expect(endpoint).toBe(`${base}/${format === 'openai' ? 'chat/completions' : 'v1/messages'}`);
        expect(body.model).toBe('MiniMax-M3.1-Flash');
        expect(body.max_tokens).toBe(524_288);
        expect(body.thinking).toEqual({ type: 'adaptive' });
        expect(body).not.toHaveProperty('enable_thinking');
        expect(body).not.toHaveProperty('service_tier');
        expect(body).not.toHaveProperty('reasoning_split');
      });

      it('serializes the default streaming chat path used by the UI', async () => {
        const { body } = await send(MINIMAX_MODELS[0].id, {}, base, true);
        expect(body.model).toBe('MiniMax-M3.1-Flash');
        expect(body.stream).toBe(true);
        expect(body.max_tokens).toBe(524_288);
        expect(body.thinking).toEqual({ type: 'adaptive' });
      });

      it.each(['low', 'medium', 'high', 'xhigh', 'max'] as const)('serializes explicit %s effort and keeps lower output limits', async effort => {
        const { body } = await send('MiniMax-M3.1-Flash', {
          max_tokens: 4096, thinking: { type: 'enabled', budget_tokens: 1024 }, output_config: { effort }, stream: true,
        });
        expect(body.max_tokens).toBe(4096);
        expect(body.stream).toBe(true);
        expect(body.thinking).toEqual({ type: 'adaptive' });
        expect(format === 'openai' ? body.reasoning_effort : body.output_config.effort).toBe(effort);
      });

      it('rejects explicit disabled Flash thinking before transport', async () => {
        await expect(send('MiniMax-M3.1-Flash', { thinking: { type: 'disabled' } })).rejects.toThrow('requires adaptive thinking');
        expect(fetch).not.toHaveBeenCalled();
      });

      it.each(['MiniMax-M3', 'MiniMax-M2.7', 'custom-model'])('retains explicit %s requests', async id => {
        const { body } = await send(id, { max_tokens: 4096 });
        expect(body.model).toBe(id);
        expect(body.max_tokens).toBe(4096);
        expect(body).not.toHaveProperty('thinking');
      });

      it('keeps M3 disabled thinking', async () => {
        const { body } = await send('MiniMax-M3', { thinking: { type: 'disabled' } });
        expect(body.thinking).toEqual({ type: 'disabled' });
      });

      it('does not rewrite gateway requests', async () => {
        const { body } = await send('MiniMax-M3.1-Flash', {}, 'https://gateway.example/v1');
        expect(body.max_tokens).toBe(1_000_000);
        expect(body).not.toHaveProperty('thinking');
      });

      it('preserves image and tool conversation content at serialization', async () => {
        const messages = [
          { role: 'user', content: [{ type: 'image', source: { type: 'base64', media_type: 'image/png', data: 'AAAA' } }, { type: 'text', text: 'Inspect' }] },
          { role: 'assistant', content: [{ type: 'thinking', thinking: 'Inspect the image first', signature: 'fixture-signature' }, { type: 'tool_use', id: 'call-1', name: 'inspect', input: { path: 'sample' } }] },
          { role: 'user', content: [{ type: 'tool_result', tool_use_id: 'call-1', content: 'Found' }] },
        ] as unknown as AnthropicRequest['messages'];
        const { body } = await send('MiniMax-M3.1-Flash', {
          messages, tools: [{ name: 'inspect', input_schema: { type: 'object' } }],
        });
        if (format === 'anthropic') {
          expect(body.messages).toEqual(messages);
        } else {
          expect(body.messages[0].content).toContainEqual({ type: 'image_url', image_url: { url: 'data:image/png;base64,AAAA' } });
          expect(body.messages[1].tool_calls[0].id).toBe('call-1');
          expect(body.messages[1].reasoning_content).toBe('Inspect the image first');
          expect(body.messages[2]).toEqual({ role: 'tool', tool_call_id: 'call-1', content: 'Found' });
        }
        expect(body.tools).toHaveLength(1);
      });
    });
  }
}
