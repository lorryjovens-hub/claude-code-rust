import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import ProviderSettings from '../components/ProviderSettings';
import { createProvider, updateProvider, type Provider } from '../api';
import { APIProxy } from '../utils/apiProxy';

vi.mock('../api', () => ({
  getProviders: vi.fn().mockResolvedValue([]),
  createProvider: vi.fn(async (provider: Partial<Provider>) => ({ ...provider, id: 'fixture' })),
  updateProvider: vi.fn(async (id: string, provider: Partial<Provider>) => ({ ...provider, id })),
  deleteProvider: vi.fn(),
  testProviderWebSearch: vi.fn().mockResolvedValue({ ok: false }),
}));

afterEach(() => vi.unstubAllGlobals());

it('quick-adds Flash first and retains it after discovery through the outgoing request', async () => {
  const fetchMock = vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ data: [{ id: 'MiniMax-M1' }, { id: 'MiniMax-M2.7' }, { id: 'MiniMax-M3' }] }),
  });
  vi.stubGlobal('fetch', fetchMock);
  render(<ProviderSettings />);
  fireEvent.click(screen.getByTitle('Add provider'));
  fireEvent.change(screen.getByPlaceholderText(/https:/), { target: { value: 'https://api.minimax.io/v1' } });
  const key = screen.getByPlaceholderText('API Key');
  fireEvent.change(key, { target: { value: 'test-only' } });
  fireEvent.keyDown(key, { key: 'Enter' });
  await waitFor(() => expect(updateProvider).toHaveBeenCalled());
  const created = vi.mocked(createProvider).mock.calls[0][0];
  const discovered = vi.mocked(updateProvider).mock.calls[0][1].models!;
  expect(created.models?.map(model => model.id)).toEqual(['MiniMax-M3.1-Flash', 'MiniMax-M3']);
  expect(discovered.map(model => model.id)).toEqual(['MiniMax-M3.1-Flash', 'MiniMax-M3']);

  const proxy = new APIProxy();
  proxy.setProviders([{
    id: 'fixture', name: created.name!, base_url: created.baseUrl!, api_key: 'test-only',
    api_format: created.format!, enabled: true,
    models: discovered.map(model => ({ ...model, enabled: true })),
  }]);
  await proxy.chat({ model: discovered[0].id, messages: [{ role: 'user', content: 'Hello' }], max_tokens: 1_000_000 });
  const [endpoint, request] = fetchMock.mock.calls[fetchMock.mock.calls.length - 1];
  expect(endpoint).toBe('https://api.minimax.io/v1/chat/completions');
  expect(JSON.parse(request.body)).toMatchObject({ model: 'MiniMax-M3.1-Flash', max_tokens: 524_288, thinking: { type: 'adaptive' } });
});
