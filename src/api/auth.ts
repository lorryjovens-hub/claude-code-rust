import { request, GATEWAY_BASE, setCachedCredentials } from './client';
import { setSecure, removeSecure } from '../services/secureStorage';

export async function sendCode(email: string) {
  const res = await request('/auth/send-code', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
  return res.json();
}

export async function register(email: string, password: string, nickname: string, code: string) {
  const res = await request('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password, nickname, code }),
  });
  return res.json();
}

export async function login(email: string, password: string) {
  const res = await request('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
  return res.json();
}

// Gateway login for Electron app — authenticates via local API proxy, returns API key for Claude Code SDK
export async function gatewayLogin(email: string, password: string) {
  const res = await fetch(`${GATEWAY_BASE}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || '登录失败');
  }
  const data = await res.json();
  if (data.token) {
    await setSecure('ANTHROPIC_API_KEY', data.token);
    localStorage.setItem('ANTHROPIC_BASE_URL', GATEWAY_BASE);
    await setSecure('gateway_user', JSON.stringify(data.user || {}));
    if (data.apiKey) {
      await setSecure('CUSTOM_API_KEY', data.apiKey);
    }
    await setSecure('auth_token', data.token);
    if (data.user) {
      localStorage.setItem('user', JSON.stringify(data.user));
    }
    // Update cache in client module
    setCachedCredentials(data.token, data.apiKey || null);
  }
  return data;
}

export async function forgotPassword(email: string) {
  const res = await request('/auth/forgot-password', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
  return res.json();
}

export async function resetPassword(email: string, code: string, password: string) {
  const res = await request('/auth/reset-password', {
    method: 'POST',
    body: JSON.stringify({ email, code, password }),
  });
  return res.json();
}
