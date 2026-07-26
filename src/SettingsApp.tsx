import { useCallback, useEffect, useRef, useState } from 'react';
import { formatHotkeyDisplay, hotkeyFromKeyboardEvent } from './lib/hotkeyCapture';
import { getSettings, setHotkey } from './lib/tauri';

const RESERVED_TIMEOUT_MS = 450;

export function SettingsApp() {
  const [hotkey, setHotkeyValue] = useState('Alt+Space');
  const [recording, setRecording] = useState(false);
  const [hint, setHint] = useState<string | null>(null);
  const [status, setStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [error, setError] = useState('');
  const pendingModifierRef = useRef<'alt' | 'meta' | 'other' | null>(null);
  const timeoutRef = useRef<number | null>(null);

  useEffect(() => {
    getSettings()
      .then((settings) => setHotkeyValue(settings.hotkey))
      .catch((err) => {
        console.error(err);
        setError('Could not load settings');
        setStatus('error');
      });
  }, []);

  const saveHotkey = useCallback(async (combo: string) => {
    setStatus('saving');
    setError('');
    try {
      await setHotkey(combo.trim());
      setHotkeyValue(combo.trim());
      setStatus('saved');
    } catch (err) {
      setStatus('error');
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const clearPending = useCallback(() => {
    pendingModifierRef.current = null;
    if (timeoutRef.current !== null) {
      window.clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const showReservedHint = useCallback((modifier: 'alt' | 'meta' | 'other') => {
    if (modifier === 'alt') {
      setHint('Windows reserves Alt+Space, Alt+Tab, and Alt+F4. Try Ctrl+Alt+ a key instead.');
    } else if (modifier === 'meta') {
      setHint('Windows reserves most Win-key combos. Try Ctrl+Shift+ a key instead.');
    } else {
      setHint('Didn\u2019t catch that \u2014 hold the modifier and press another key together.');
    }
  }, []);

  useEffect(() => {
    if (!recording) {
      clearPending();
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        clearPending();
        setRecording(false);
        return;
      }
      if (event.repeat) return;

      const combo = hotkeyFromKeyboardEvent(event);
      if (combo) {
        event.preventDefault();
        event.stopPropagation();
        clearPending();
        setHint(null);
        setRecording(false);
        void saveHotkey(combo);
        return;
      }

      // A lone modifier went down; give the OS a moment to either deliver a
      // full combo (fires another keydown above) or swallow it entirely
      // (Alt+Space/Alt+Tab/Win+* never reach the webview at all).
      if (event.key === 'Alt' || event.key === 'Meta' || event.key === 'Control' || event.key === 'Shift') {
        setHint(null);
        pendingModifierRef.current = event.key === 'Alt' ? 'alt' : event.key === 'Meta' ? 'meta' : 'other';
        if (timeoutRef.current !== null) window.clearTimeout(timeoutRef.current);
        timeoutRef.current = window.setTimeout(() => {
          if (pendingModifierRef.current) {
            showReservedHint(pendingModifierRef.current);
          }
        }, RESERVED_TIMEOUT_MS);
      }
    };

    const onKeyUp = (event: KeyboardEvent) => {
      if (
        pendingModifierRef.current &&
        (event.key === 'Alt' || event.key === 'Meta' || event.key === 'Control' || event.key === 'Shift')
      ) {
        showReservedHint(pendingModifierRef.current);
        clearPending();
      }
    };

    window.addEventListener('keydown', onKeyDown, true);
    window.addEventListener('keyup', onKeyUp, true);
    return () => {
      window.removeEventListener('keydown', onKeyDown, true);
      window.removeEventListener('keyup', onKeyUp, true);
    };
  }, [recording, saveHotkey, clearPending, showReservedHint]);

  return (
    <div className="flex min-h-screen flex-col bg-[#1c1c1e] px-6 py-6 text-white">
      <h1 className="text-[18px] font-semibold">Spotlight Settings</h1>
      <p className="mt-1 text-[13px] text-white/55">Configure how Spotlight behaves on your system.</p>

      <p className="mt-6 text-[13px] font-medium text-white/80">Global hotkey</p>
      <button
        type="button"
        onClick={() => {
          setRecording(true);
          setHint(null);
          setStatus('idle');
          setError('');
        }}
        className={`mt-2 w-full rounded-lg border px-4 py-3 text-left transition ${
          recording
            ? 'border-sky-400/60 bg-sky-400/10 ring-1 ring-sky-400/30'
            : 'border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/8'
        }`}
      >
        <span className="block text-[12px] text-white/45">
          {recording ? 'Press your shortcut…' : 'Click, then press keys'}
        </span>
        <span className="mt-1 block font-mono text-[16px] text-white">
          {recording ? 'Listening…' : formatHotkeyDisplay(hotkey)}
        </span>
      </button>
      <p className={`mt-2 text-[12px] ${hint ? 'text-amber-400' : 'text-white/45'}`}>
        {hint ?? (recording
          ? 'Press Escape to cancel. Include a modifier (Ctrl, Alt, Shift, or Win).'
          : 'Example: Alt + Space, Ctrl + Shift + K')}
      </p>

      <div className="mt-5 flex min-h-[20px] items-center gap-3">
        {status === 'saving' && <span className="text-[13px] text-white/55">Saving…</span>}
        {status === 'saved' && <span className="text-[13px] text-emerald-400">Saved</span>}
        {status === 'error' && error && <span className="text-[13px] text-red-400">{error}</span>}
      </div>

      <p className="mt-auto pt-6 text-[12px] text-white/40">
        Use the ⚙ button in Spotlight to change the hotkey, or find Spotlight in the taskbar ^ menu
        to open, adjust settings, or quit.
      </p>
    </div>
  );
}

export default SettingsApp;
