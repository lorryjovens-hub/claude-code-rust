import { useState, useEffect, useCallback } from 'react';
import { getUnreadAnnouncements, markAnnouncementRead } from '../api';

interface Announcement {
  id: number;
  title: string;
  content: string;
  created_at: string;
}

export function useAnnouncements() {
  const [unread, setUnread] = useState<Announcement[]>([]);
  const [activeId, setActiveId] = useState<number | null>(null);
  const [marking, setMarking] = useState(false);

  const load = useCallback(async () => {
    try {
      const data = await getUnreadAnnouncements();
      setUnread(Array.isArray(data?.announcements) ? data.announcements : []);
    } catch (err) {
      console.error('Failed to fetch announcements:', err);
    }
  }, []);

  useEffect(() => {
    load();
    const id = window.setInterval(load, 15000);
    const onFocus = () => load();
    const onVisible = () => { if (document.visibilityState === 'visible') load(); };
    window.addEventListener('focus', onFocus);
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      clearInterval(id);
      window.removeEventListener('focus', onFocus);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, [load]);

  useEffect(() => {
    if (unread.length === 0) { setActiveId(null); return; }
    if (activeId === null || !unread.some(a => a.id === activeId)) {
      setActiveId(unread[0].id);
    }
  }, [unread, activeId]);

  const active = unread.find(a => a.id === activeId) || null;

  const markRead = useCallback(async () => {
    if (!active || marking) return;
    setMarking(true);
    try {
      await markAnnouncementRead(active.id);
      setUnread(prev => prev.filter(a => a.id !== active.id));
    } catch (err: any) {
      alert(err?.message || '公告已读失败');
    } finally {
      setMarking(false);
    }
  }, [active, marking]);

  return { unread, active, activeId, marking, markRead };
}
