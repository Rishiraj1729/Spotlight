const MODIFIER_KEYS = new Set(['Control', 'Alt', 'Shift', 'Meta']);

const NAMED_KEYS: Record<string, string> = {
  Space: 'Space',
  Enter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Insert: 'Insert',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Minus: 'Minus',
  Equal: 'Equal',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
  Backslash: 'Backslash',
  Semicolon: 'Semicolon',
  Quote: 'Quote',
  Comma: 'Comma',
  Period: 'Period',
  Slash: 'Slash',
  Backquote: 'Backquote',
};

function keyFromCode(code: string): string | null {
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (/^F\d+$/.test(code)) return code;
  if (code.startsWith('Numpad')) return code;
  return NAMED_KEYS[code] ?? null;
}

/** Turn a browser keydown into a Tauri global-shortcut string (e.g. Alt+Space). */
export function hotkeyFromKeyboardEvent(event: KeyboardEvent): string | null {
  if (event.repeat || MODIFIER_KEYS.has(event.key)) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push('Super');

  const key = keyFromCode(event.code);
  if (!key) return null;

  // Global shortcuts should include a modifier so normal typing doesn't collide.
  if (parts.length === 0) return null;

  parts.push(key);
  return parts.join('+');
}

export function formatHotkeyDisplay(combo: string): string {
  return combo.split('+').join(' + ');
}
