import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// Auto-cleanup after each test
afterEach(() => {
  cleanup();
});

// Mock Tauri API for testing
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    const mockResponses: Record<string, unknown> = {
      secure_get_api_key: null,
      secure_set_api_key: undefined,
      secure_delete_api_key: undefined,
      secure_get_gateway_user: null,
      secure_set_gateway_user: undefined,
      secure_delete_gateway_user: undefined,
      secure_get_gateway_quota: null,
      secure_set_gateway_quota: undefined,
      secure_delete_gateway_quota: undefined,
      secure_get_auth_token: null,
      secure_set_auth_token: undefined,
      secure_delete_auth_token: undefined,
      secure_clear_all: undefined,
      get_platform: 'win32',
      get_system_status: { platform: 'win32', gitBash: { required: false, found: true, path: null } },
    };
    return mockResponses[cmd] ?? null;
  }),
}));

// Mock window.__TAURI__ for secureStorage fallback detection
Object.defineProperty(window, '__TAURI__', {
  value: undefined,
  writable: true,
  configurable: true,
});

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value.toString();
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
  writable: true,
  configurable: true,
});

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  readonly root: Element | Document | null = null;
  readonly rootMargin: string = '0px';
  readonly thresholds: ReadonlyArray<number> = [0];
  constructor() {}
  disconnect() {}
  observe() {}
  takeRecords(): IntersectionObserverEntry[] {
    return [];
  }
  unobserve() {}
} as any;

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  unobserve() {}
};
