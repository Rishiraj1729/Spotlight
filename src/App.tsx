import { useEffect, useRef, useState } from 'react';
import { SearchBar } from './components/SearchBar';
import { ResultsList } from './components/ResultsList';
import { useSearch } from './hooks/useSearch';
import { useKeyboardNav } from './hooks/useKeyboardNav';
import { dismissWelcome, getSettings, hideWindow, launchResult, resizeWindow } from './lib/tauri';
import type { SearchResult } from './types/result';

function App() {
  const [query, setQuery] = useState('');
  const [hotkey, setHotkey] = useState('Alt+Space');
  const [showWelcome, setShowWelcome] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const cardRef = useRef<HTMLDivElement>(null);
  const { results } = useSearch(query);

  useEffect(() => {
    getSettings().then((settings) => {
      setHotkey(settings.hotkey);
      setShowWelcome(!settings.welcomeDismissed);
    });
  }, []);

  useEffect(() => {
    const card = cardRef.current;
    if (!card) return;

    const observer = new ResizeObserver(([entry]) => {
      resizeWindow(entry.contentRect.height);
    });
    observer.observe(card);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const refocus = () => inputRef.current?.focus();
    window.addEventListener('focus', refocus);
    refocus();
    return () => window.removeEventListener('focus', refocus);
  }, []);

  const activate = async (result: SearchResult) => {
    await launchResult(result);
    setQuery('');
    await hideWindow();
  };

  const dismiss = async () => {
    setQuery('');
    await hideWindow();
  };

  const handleDismissWelcome = async () => {
    await dismissWelcome();
    setShowWelcome(false);
  };

  const { selectedIndex, setSelectedIndex, onKeyDown } = useKeyboardNav({
    results,
    onActivate: activate,
    onDismiss: dismiss,
  });

  const welcomeVisible = showWelcome && query.trim() === '';

  return (
    <div
      ref={cardRef}
      className="spotlight-card w-screen overflow-hidden rounded-2xl border border-white/10 bg-[#08080acc] shadow-[0_12px_40px_rgba(0,0,0,0.55)] ring-1 ring-inset ring-white/8 backdrop-blur-3xl backdrop-saturate-150"
    >
      {welcomeVisible && (
        <div className="border-b border-white/8 px-5 py-3">
          <div className="flex items-start justify-between gap-4 rounded-xl border border-white/8 bg-black/30 px-4 py-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] backdrop-blur-xl">
            <div className="min-w-0 space-y-1 text-[12px] leading-relaxed text-white/60">
              <p>Welcome — Spotlight is ready. Search apps, files, folders, settings, and the web.</p>
              <p>
                Press <span className="font-medium text-white/90">{hotkey}</span> anytime to open.
              </p>
              <p>
                Tap <span className="text-white/80">⚙</span> to change the hotkey. Starts at login and
                runs in the background — find <span className="font-medium text-white/80">Spotlight</span>{' '}
                in the taskbar <span className="font-medium text-white/80">^</span> menu to quit.
              </p>
            </div>
            <button
              type="button"
              tabIndex={-1}
              onMouseDown={(event) => {
                event.preventDefault();
                void handleDismissWelcome();
              }}
              className="shrink-0 rounded-lg border border-white/10 bg-black/35 px-3 py-1 text-[12px] font-medium text-white/80 transition hover:border-white/16 hover:bg-black/45 hover:text-white"
            >
              Got it
            </button>
          </div>
        </div>
      )}
      <SearchBar ref={inputRef} value={query} onChange={setQuery} onKeyDown={onKeyDown} />
      {results.length > 0 && <div className="h-px bg-white/8" />}
      <ResultsList
        results={results}
        selectedIndex={selectedIndex}
        onSelect={setSelectedIndex}
        onActivate={activate}
      />
    </div>
  );
}

export default App;
