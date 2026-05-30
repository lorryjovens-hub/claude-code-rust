import { useEffect, useCallback, useRef, useState } from 'react';

export function useScrollToBottom(messages: any[], loading: boolean, inputHeight: number) {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);
  const userScrolledUpRef = useRef(false);
  const [scrollbarWidth, setScrollbarWidth] = useState(0);

  const scrollToBottom = useCallback((behavior: ScrollBehavior = 'auto') => {
    const el = scrollContainerRef.current;
    if (el) {
      el.scrollTo({ top: el.scrollHeight, behavior });
    }
  }, []);

  const scheduleScrollToBottomAfterRender = useCallback((attempts = 6) => {
    const run = (remaining: number) => {
      if (userScrolledUpRef.current || !isAtBottomRef.current) return;
      const el = scrollContainerRef.current;
      if (el) el.scrollTop = el.scrollHeight;
      if (remaining > 0) requestAnimationFrame(() => run(remaining - 1));
    };
    requestAnimationFrame(() => run(attempts));

    [80, 200, 400, 800, 1200].forEach((delay) => {
      window.setTimeout(() => {
        if (userScrolledUpRef.current || !isAtBottomRef.current) return;
        const el = scrollContainerRef.current;
        if (el) el.scrollTop = el.scrollHeight;
      }, delay);
    });
  }, []);

  const handleScroll = useCallback(() => {
    if (scrollContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef.current;
      const isBottom = Math.abs(scrollHeight - clientHeight - scrollTop) < 50;
      if (isBottom && userScrolledUpRef.current) {
        userScrolledUpRef.current = false;
      }
      if (!userScrolledUpRef.current) {
        isAtBottomRef.current = isBottom;
      }
    }
  }, []);

  // Detect scrollbar width
  useEffect(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const update = () => setScrollbarWidth(el.offsetWidth - el.clientWidth);
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, [messages]);

  // User wheel scroll up detection
  useEffect(() => {
    const el = scrollContainerRef.current;
    if (!el) return;
    const handleWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        userScrolledUpRef.current = true;
        isAtBottomRef.current = false;
        el.scrollTo({ top: el.scrollTop });
      }
    };
    el.addEventListener('wheel', handleWheel, { passive: true });
    return () => el.removeEventListener('wheel', handleWheel);
  }, []);

  // Auto-scroll when new messages arrive during loading
  useEffect(() => {
    if (isAtBottomRef.current && loading && !userScrolledUpRef.current) {
      scrollToBottom('auto');
    }
  }, [messages, loading, scrollToBottom]);

  // Stay at bottom when input height changes
  useEffect(() => {
    if (isAtBottomRef.current && scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop = scrollContainerRef.current.scrollHeight;
    }
  }, [inputHeight]);

  return {
    scrollContainerRef,
    isAtBottomRef,
    scrollbarWidth,
    scrollToBottom,
    scheduleScrollToBottomAfterRender,
    handleScroll,
  };
}
