const API_BASE = '/api/admin';

function getToken() {
  return localStorage.getItem('auth_token');
}

async function adminRequest(path: string, options: RequestInit = {}) {
  const token = getToken();
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...(options.headers || {}),
  };
  if (token) {
    (headers as any)['Authorization'] = `Bearer ${token}`;
  }
  const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
  if (res.status === 401) {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
    window.location.href = '/login';
    throw new Error('认证失效');
  }
  if (res.status === 403) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error || '无权限');
  }
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error || `Request failed: ${res.status}`);
  }
  return res.json();
}

// Dashboard
export const getDashboard = () => adminRequest('/dashboard');

// Keys
export const getKeys = () => adminRequest('/keys');
export const addKey = (data: any) => adminRequest('/keys', { method: 'POST', body: JSON.stringify(data) });
export const updateKey = (id: number, data: any) => adminRequest(`/keys/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deleteKey = (id: number) => adminRequest(`/keys/${id}`, { method: 'DELETE' });
export const toggleKey = (id: number) => adminRequest(`/keys/${id}/toggle`, { method: 'POST' });
export const getPoolStatus = () => adminRequest('/keys/pool-status');
export const getUpstreamRoutes = () => adminRequest('/upstream-routes');
export const updateUpstreamRoutes = (routes: any) =>
  adminRequest('/upstream-routes', { method: 'PUT', body: JSON.stringify({ routes }) });

// Users
export const getUsers = (params: Record<string, any> = {}) => {
  const qs = new URLSearchParams(params).toString();
  return adminRequest(`/users?${qs}`);
};
export const banUser = (id: string) => adminRequest(`/users/${id}/ban`, { method: 'POST' });
export const unbanUser = (id: string) => adminRequest(`/users/${id}/unban`, { method: 'POST' });
export const resetUserPassword = (id: string, password: string) =>
  adminRequest(`/users/${id}/reset-password`, { method: 'POST', body: JSON.stringify({ password }) });
export const adjustUserQuota = (id: string, data: any) =>
  adminRequest(`/users/${id}/adjust-quota`, { method: 'POST', body: JSON.stringify(data) });
export const setUserRole = (id: string, role: string) =>
  adminRequest(`/users/${id}/role`, { method: 'POST', body: JSON.stringify({ role }) });
export const getAdminMe = () => adminRequest('/me');

// Plans
export const getPlans = () => adminRequest('/plans');
export const addPlan = (data: any) => adminRequest('/plans', { method: 'POST', body: JSON.stringify(data) });
export const updatePlan = (id: number, data: any) => adminRequest(`/plans/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deletePlan = (id: number) => adminRequest(`/plans/${id}`, { method: 'DELETE' });
export const togglePlan = (id: number) => adminRequest(`/plans/${id}/toggle`, { method: 'POST' });

// Announcements
export const getAnnouncements = () => adminRequest('/announcements');
export const addAnnouncement = (data: any) =>
  adminRequest('/announcements', { method: 'POST', body: JSON.stringify(data) });
export const updateAnnouncement = (id: number, data: any) =>
  adminRequest(`/announcements/${id}`, { method: 'PUT', body: JSON.stringify(data) });
export const deleteAnnouncement = (id: number) =>
  adminRequest(`/announcements/${id}`, { method: 'DELETE' });
export const toggleAnnouncement = (id: number) =>
  adminRequest(`/announcements/${id}/toggle`, { method: 'POST' });

// Redemption
export const generateRedemptionCodes = (data: any) =>
  adminRequest('/redemption/generate', { method: 'POST', body: JSON.stringify(data) });
export const getRedemptionCodes = (params: Record<string, any> = {}) => {
  const qs = new URLSearchParams(params).toString();
  return adminRequest(`/redemption/list?${qs}`);
};
export const disableRedemptionCodes = (codes: string[]) =>
  adminRequest('/redemption/disable', { method: 'POST', body: JSON.stringify({ codes }) });

// Stats
export const getStatsTrends = (days = 30) => adminRequest(`/stats/trends?days=${days}`);
export const getStatsCost = (days = 30) => adminRequest(`/stats/cost?days=${days}`);

// Recharges
export const getRecharges = () => adminRequest('/recharges');
export const addRecharge = (data: any) => adminRequest('/recharges', { method: 'POST', body: JSON.stringify(data) });
export const deleteRecharge = (id: number) => adminRequest(`/recharges/${id}`, { method: 'DELETE' });

// Models
export const getModels = () => adminRequest('/models');
export const addModel = (data: any) => adminRequest('/models', { method: 'POST', body: JSON.stringify(data) });
export const updateModel = (id: string, data: any) => adminRequest(`/models/${encodeURIComponent(id)}`, { method: 'PUT', body: JSON.stringify(data) });
export const deleteModel = (id: string) => adminRequest(`/models/${encodeURIComponent(id)}`, { method: 'DELETE' });
export const getCommonModelsConfig = () => adminRequest('/models/common');
export const updateCommonModelsConfig = (model_ids: string[]) =>
  adminRequest('/models/common', { method: 'PUT', body: JSON.stringify({ model_ids }) });
