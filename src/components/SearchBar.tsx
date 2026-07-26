import { forwardRef } from 'react';
import { openSettings } from '../lib/tauri';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (event: React.KeyboardEvent<HTMLInputElement>) => void;
}

function SettingsIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 20 20"
      fill="none"
      className="h-4 w-4"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M10 12.2a2.2 2.2 0 1 0 0-4.4 2.2 2.2 0 0 0 0 4.4Z"
      />
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M15.9 11.3c.05-.4.08-.8.08-1.3s-.03-.9-.08-1.3l1.4-1.1a.35.35 0 0 0 .08-.45l-1.3-2.3a.35.35 0 0 0-.42-.16l-1.7.7a7.2 7.2 0 0 0-2.2-1.3l-.25-1.8a.35.35 0 0 0-.35-.3H8.3a.35.35 0 0 0-.35.3l-.25 1.8a7.2 7.2 0 0 0-2.2 1.3l-1.7-.7a.35.35 0 0 0-.42.16l-1.3 2.3a.35.35 0 0 0 .08.45l1.4 1.1c-.05.4-.08.8-.08 1.3s.03.9.08 1.3l-1.4 1.1a.35.35 0 0 0-.08.45l1.3 2.3c.1.17.3.23.47.16l1.7-.7c.7.55 1.4.98 2.2 1.3l.25 1.8c.03.17.18.3.35.3h2.6c.17 0 .32-.13.35-.3l.25-1.8c.8-.32 1.5-.75 2.2-1.3l1.7.7c.17.07.37 0 .47-.16l1.3-2.3a.35.35 0 0 0-.08-.45l-1.4-1.1Z"
      />
    </svg>
  );
}

export const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(function SearchBar(
  { value, onChange, onKeyDown },
  ref,
) {
  const handleOpenSettings = () => {
    openSettings().catch((error) => {
      console.error('Failed to open settings', error);
    });
  };

  return (
    <div className="flex items-center gap-3 px-5 py-4">
      <span className="text-[20px] text-white/45">⌘</span>
      <input
        ref={ref}
        autoFocus
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={onKeyDown}
        placeholder="Spotlight Search"
        spellCheck={false}
        className="min-w-0 flex-1 bg-transparent text-[22px] font-light text-white placeholder:text-white/35 focus:outline-none"
      />
      <button
        type="button"
        tabIndex={-1}
        aria-label="Settings"
        onMouseDown={(event) => {
          event.preventDefault();
          event.stopPropagation();
          handleOpenSettings();
        }}
        className="flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-xl border border-white/10 bg-black/35 text-white/65 shadow-[inset_0_1px_0_rgba(255,255,255,0.08)] backdrop-blur-md transition hover:border-white/18 hover:bg-black/50 hover:text-white/90 active:scale-95"
      >
        <SettingsIcon />
      </button>
    </div>
  );
});
