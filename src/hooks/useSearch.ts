import { useEffect, useRef, useState } from 'react';
import type { SearchResult } from '../types/result';
import { searchQuery } from '../lib/tauri';

const DEBOUNCE_MS = 30;

export function useSearch(query: string) {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const requestIdRef = useRef(0);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setIsSearching(false);
      return;
    }

    setIsSearching(true);
    const requestId = ++requestIdRef.current;

    const timer = setTimeout(async () => {
      try {
        const nextResults = await searchQuery(query);
        // Ignore stale responses from superseded keystrokes.
        if (requestIdRef.current === requestId) {
          setResults(nextResults);
          setIsSearching(false);
        }
      } catch (err) {
        console.error('search failed', err);
        if (requestIdRef.current === requestId) {
          setIsSearching(false);
        }
      }
    }, DEBOUNCE_MS);

    return () => clearTimeout(timer);
  }, [query]);

  return { results, isSearching };
}
