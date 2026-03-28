import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MessagePage {
  messages: any[];
  has_more: boolean;
  next_cursor: string | null;
  total_count: number;
}

export function useVirtualMessages(conversationId: string, pageSize = 30) {
  const [messages, setMessages] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const cursorRef = useRef<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);

  useEffect(() => {
    setMessages([]);
    cursorRef.current = null;
    setHasMore(true);
    loadPage();
  }, [conversationId]);

  const loadPage = useCallback(async () => {
    if (loading || !hasMore) return;
    setLoading(true);

    try {
      const page = await invoke<MessagePage>('get_messages_paginated', {
        conversationId,
        cursor: cursorRef.current,
        limit: pageSize,
        direction: 'older',
      });

      setMessages(prev => [...prev, ...page.messages]);
      setHasMore(page.has_more);
      cursorRef.current = page.next_cursor;
    } catch (err) {
      console.error('Load messages failed:', err);
    }

    setLoading(false);
  }, [conversationId, loading, hasMore, pageSize]);

  // Intersection observer for infinite scroll
  const sentinelRef = useCallback((node: HTMLDivElement | null) => {
    if (observerRef.current) observerRef.current.disconnect();
    if (!node) return;

    observerRef.current = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loading) {
          loadPage();
        }
      },
      { threshold: 0.1 }
    );

    observerRef.current.observe(node);
  }, [loadPage, hasMore, loading]);

  const prependMessage = useCallback((msg: any) => {
    setMessages(prev => [msg, ...prev]);
  }, []);

  return {
    messages,
    loading,
    hasMore,
    sentinelRef,
    listRef,
    prependMessage,
    reload: () => {
      setMessages([]);
      cursorRef.current = null;
      setHasMore(true);
      loadPage();
    },
  };
}
