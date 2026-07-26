import { useCallback, useEffect, useState } from 'react';
import type { SearchResult } from '../types/result';

interface UseKeyboardNavOptions {
  results: SearchResult[];
  onActivate: (result: SearchResult) => void;
  onDismiss: () => void;
}

export function useKeyboardNav({ results, onActivate, onDismiss }: UseKeyboardNavOptions) {
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    setSelectedIndex(0);
  }, [results]);

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setSelectedIndex((index) => Math.min(index + 1, results.length - 1));
          break;
        case 'ArrowUp':
          event.preventDefault();
          setSelectedIndex((index) => Math.max(index - 1, 0));
          break;
        case 'Enter': {
          event.preventDefault();
          const selected = results[selectedIndex];
          if (selected) onActivate(selected);
          break;
        }
        case 'Escape':
          event.preventDefault();
          onDismiss();
          break;
      }
    },
    [results, selectedIndex, onActivate, onDismiss],
  );

  return { selectedIndex, setSelectedIndex, onKeyDown };
}
