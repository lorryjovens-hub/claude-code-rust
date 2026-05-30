import { useEffect, useState } from 'react';
import { useUIStore } from '../stores';

export function useZoom() {
  const zoomLevel = useUIStore(s => s.zoomLevel);
  const setZoomLevel = useUIStore(s => s.setZoomLevel);
  const [showIndicator, setShowIndicator] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+')) {
        e.preventDefault();
        setZoomLevel(Math.min(zoomLevel + 0.1, 2.0));
        setShowIndicator(true);
        setTimeout(() => setShowIndicator(false), 1500);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '-') {
        e.preventDefault();
        setZoomLevel(Math.max(zoomLevel - 0.1, 0.5));
        setShowIndicator(true);
        setTimeout(() => setShowIndicator(false), 1500);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === '0') {
        e.preventDefault();
        setZoomLevel(1);
        setShowIndicator(true);
        setTimeout(() => setShowIndicator(false), 1500);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [zoomLevel, setZoomLevel]);

  return { zoomLevel, setZoomLevel, showIndicator };
}
