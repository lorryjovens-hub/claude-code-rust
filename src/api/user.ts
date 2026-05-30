import { request, isTauriApp, CHENGDU_API, getGatewayUsage } from './client';
import { getSecure } from '../services/secureStorage';
import { clearSyncState } from '../services/syncService';

export async function getUserProfile() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try {
      const data = await chengduRequest('/user/profile');
      // Update local cache
      if (data.user || data) {
        const user = data.user || data;
        localStorage.setItem('user', JSON.stringify(user));
      }
      return data;
    } catch (e) {
      // Fallback to cached
      const userStr = localStorage.getItem('user');
      return { user: userStr ? JSON.parse(userStr) : {} };
    }
  }
  const userStr = localStorage.getItem('user');
  return { user: userStr ? JSON.parse(userStr) : {} };
}

async function chengduRequest(path: string, options?: RequestInit) {
  const { CHENGDU_API } = await import('./client');
  const token = localStorage.getItem('auth_token');
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;
  if (options?.method && options.method !== 'GET') headers['Content-Type'] = 'application/json';
  const url = `${CHENGDU_API}${path}`;
  console.log('[chengduRequest]', url);
  const res = await fetch(url, { ...options, headers: { ...headers, ...(options?.headers as Record<string, string> || {}) } });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    console.error('[chengduRequest] Failed:', res.status, text.slice(0, 200));
    throw new Error(`Chengdu ${path} failed: ${res.status}`);
  }
  return res.json();
}

export async function updateUserProfile(data: Record<string, any>) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/user/profile`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify(data),
    });
    const result = await res.json();
    const userStr = localStorage.getItem('user');
    const user = userStr ? JSON.parse(userStr) : {};
    localStorage.setItem('user', JSON.stringify({ ...user, ...result }));
    return result;
  }
  const userStr = localStorage.getItem('user');
  const user = userStr ? JSON.parse(userStr) : {};
  const updated = { ...user, ...data };
  localStorage.setItem('user', JSON.stringify(updated));
  return updated;
}

export async function getUserUsage() {
  let usage: any = null;

  // Get plan info from Chengdu backend (requires auth_token from session-based login)
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try {
      usage = await chengduRequest('/user/usage');
    } catch (_) {}
  }

  // In Electron mode, overlay gateway usage (the real usage data) onto Chengdu's plan info
  if (isTauriApp) {
    try {
      const gwUsage = await getGatewayUsage();
      if (gwUsage) {
        if (usage && usage.quota) {
          // Both sources available: combine
          if (usage.quota.window) {
            usage.quota.window.used = (usage.quota.window.used || 0) + (gwUsage.window_used || 0);
          }
          if (usage.quota.week) {
            usage.quota.week.used = (usage.quota.week.used || 0) + (gwUsage.week_used || 0);
          }
        } else if (!usage) {
          // No Chengdu auth_token — use gateway usage as primary source.
          usage = gwUsage;
        }
      }
    } catch (_) {}
  }

  if (usage) return usage;

  // selfhosted mode (no gateway, no Chengdu) — unlimited placeholder
  return {
    plan: { id: 999, name: 'Self-hosted', status: 'active', price: 0 },
    token_quota: 0,
    token_remaining: 0,
    used: 0,
    reset_date: '',
    is_unlimited: true
  };
}

export async function getUserModels() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest('/user/models'); } catch (_) {}
  }
  try {
    const res = await request('/user/models');
    return res.json();
  } catch (_) {
    return { all: [] };
  }
}

export function getUser() {
  const userStr = localStorage.getItem('user');
  return userStr ? JSON.parse(userStr) : null;
}

export function logout() {
  localStorage.removeItem('auth_token');
  localStorage.removeItem('user');
  localStorage.removeItem('ANTHROPIC_API_KEY');
  localStorage.removeItem('ANTHROPIC_BASE_URL');
  localStorage.removeItem('gateway_user');
  localStorage.removeItem('gateway_quota');
  clearSyncState();
  window.location.hash = '#/login'; window.location.reload();
}

export async function changePassword(currentPassword: string, newPassword: string) {
  const res = await request('/user/change-password', {
    method: 'POST',
    body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
  });
  return res.json();
}

export async function deleteAccount(password: string) {
  const res = await request('/user/delete-account', {
    method: 'POST',
    body: JSON.stringify({ password }),
  });
  return res.json();
}

export async function getSessions() {
  const res = await request('/user/sessions');
  return res.json();
}

export async function deleteSession(id: string) {
  const res = await request(`/user/sessions/${id}`, { method: 'DELETE' });
  return res.json();
}

export async function logoutOtherSessions() {
  const res = await request('/user/sessions/logout-others', { method: 'POST' });
  return res.json();
}

export async function getUnreadAnnouncements() {
  const res = await request('/user/announcements');
  return res.json();
}

export async function markAnnouncementRead(id: number) {
  const res = await request(`/user/announcements/${id}/read`, {
    method: 'POST',
  });
  return res.json();
}

// Payment / Plans / Redemption
export async function getPlans() {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest('/payment/plans'); } catch (_) {}
  }
  const res = await request('/payment/plans');
  return res.json();
}

export async function createPaymentOrder(planId: number, paymentMethod: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/payment/create`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ plan_id: planId, payment_method: paymentMethod }),
    });
    return res.json();
  }
  const res = await request('/payment/create', {
    method: 'POST',
    body: JSON.stringify({ plan_id: planId, payment_method: paymentMethod }),
  });
  return res.json();
}

export async function getPaymentStatus(orderId: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    try { return await chengduRequest(`/payment/status/${orderId}`); } catch (_) {}
  }
  const res = await request(`/payment/status/${orderId}`);
  return res.json();
}

export async function redeemCode(code: string) {
  if (isTauriApp && localStorage.getItem('auth_token')) {
    const token = localStorage.getItem('auth_token');
    const res = await fetch(`${CHENGDU_API}/redemption/redeem`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
      body: JSON.stringify({ code }),
    });
    return res.json();
  }
  const res = await request('/redemption/redeem', {
    method: 'POST',
    body: JSON.stringify({ code }),
  });
  return res.json();
}

// Code API
export async function getCodeSSO() {
  const res = await request('/code/sso');
  return res.json();
}

export async function getCodeQuota() {
  const res = await request('/code/quota');
  return res.json();
}
