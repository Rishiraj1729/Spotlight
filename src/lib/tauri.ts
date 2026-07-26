import { invoke } from '@tauri-apps/api/core';
import type { SearchResult } from '../types/result';
import type { AppSettings } from '../types/settings';

/**
 * True when running inside the Tauri webview. Lets us fall back to mock data
 * while iterating on UI in a plain browser (`pnpm dev` outside `tauri dev`).
 */
export const isTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const MOCK_RESULTS: SearchResult[] = [
  {
    id: 'mock-calculator',
    provider: 'calculator',
    title: '42',
    subtitle: 'Calculator',
    icon: 'calculator',
    score: 100,
    action: { type: 'copyToClipboard', text: '42' },
  },
  {
    id: 'mock-app-notepad',
    provider: 'app',
    title: 'Notepad',
    subtitle: 'Application',
    icon: 'app',
    score: 90,
    action: { type: 'launch', path: 'notepad.exe' },
  },
];

export async function searchQuery(query: string): Promise<SearchResult[]> {
  if (!query.trim()) return [];
  if (!isTauri()) {
    return MOCK_RESULTS.filter((r) => r.title.toLowerCase().includes(query.toLowerCase()) || query.length > 0);
  }
  return invoke<SearchResult[]>('search', { query });
}

export async function launchResult(result: SearchResult): Promise<void> {
  if (!isTauri()) {
    console.log('[dev] would launch', result);
    return;
  }
  return invoke('launch', { result });
}

export async function hideWindow(): Promise<void> {
  if (!isTauri()) return;
  return invoke('hide_window');
}

export async function resizeWindow(height: number): Promise<void> {
  if (!isTauri()) return;
  return invoke('resize_window', { height });
}

export async function getSettings(): Promise<AppSettings> {
  if (!isTauri()) {
    return { hotkey: 'Alt+Space', welcomeDismissed: false };
  }
  return invoke<AppSettings>('get_settings');
}

export async function setHotkey(combo: string): Promise<void> {
  if (!isTauri()) {
    console.log('[dev] would set hotkey', combo);
    return;
  }
  return invoke('set_hotkey', { combo });
}

export async function openSettings(): Promise<void> {
  if (!isTauri()) {
    console.log('[dev] would open settings');
    return;
  }
  return invoke('open_settings');
}

export async function dismissWelcome(): Promise<void> {
  if (!isTauri()) {
    console.log('[dev] would dismiss welcome');
    return;
  }
  return invoke('dismiss_welcome');
}
