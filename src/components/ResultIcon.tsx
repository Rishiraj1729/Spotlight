interface ResultIconProps {
  icon: string;
  provider: string;
}

// v0.1: providers send a symbolic icon name or a data: URI. Real app icon
// extraction (.exe/.lnk icons) lands with the App Provider implementation;
// this component already supports swapping in a real <img> src transparently.
const PROVIDER_GLYPH: Record<string, string> = {
  app: '◆',
  file: '▤',
  folder: '▥',
  settings: '⚙',
  calculator: '=',
  web: '⌁',
};

export function ResultIcon({ icon, provider }: ResultIconProps) {
  if (icon.startsWith('data:') || icon.startsWith('http')) {
    return <img src={icon} alt="" className="h-8 w-8 rounded-md object-contain" />;
  }

  return (
    <div className="flex h-8 w-8 items-center justify-center rounded-md bg-white/10 text-[15px] text-white/80">
      {PROVIDER_GLYPH[provider] ?? '•'}
    </div>
  );
}
