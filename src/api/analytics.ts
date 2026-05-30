import { request } from './client';

export async function trackEvent(eventType: string, properties?: Record<string, any>, sessionId?: string) {
  try {
    const res = await request('/analytics/track', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ event_type: eventType, properties, session_id: sessionId }),
    });
    return res.json();
  } catch (_) { return { success: false }; }
}

export async function getAnalyticsDaily(date: string) {
  const res = await request(`/analytics/daily/${date}`);
  return res.json();
}

export async function getAnalyticsRange(from: string, to: string) {
  const res = await request(`/analytics/range?from=${from}&to=${to}`);
  return res.json();
}

export async function getAnalyticsSummary(days = 30) {
  const res = await request(`/analytics/summary?days=${days}`);
  return res.json();
}

export async function getAnalyticsEventCounts(days = 30) {
  const res = await request(`/analytics/event-counts?days=${days}`);
  return res.json();
}

export async function getAnalyticsRecentEvents(limit = 50) {
  const res = await request(`/analytics/recent-events?limit=${limit}`);
  return res.json();
}

export async function getAnalyticsDashboard() {
  const res = await request('/analytics/dashboard');
  return res.json();
}
