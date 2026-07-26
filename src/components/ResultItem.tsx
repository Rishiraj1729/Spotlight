import { motion } from 'framer-motion';
import type { SearchResult } from '../types/result';
import { ResultIcon } from './ResultIcon';

interface ResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: () => void;
  onActivate: () => void;
}

export function ResultItem({ result, isSelected, onSelect, onActivate }: ResultItemProps) {
  return (
    <li
      role="option"
      aria-selected={isSelected}
      onMouseEnter={onSelect}
      onClick={onActivate}
      className="relative flex cursor-default items-center gap-3 rounded-xl px-3 py-2"
    >
      {isSelected && (
        <motion.div
          layoutId="selected-row"
          className="absolute inset-0 rounded-xl bg-white/10"
          transition={{ type: 'spring', stiffness: 500, damping: 40 }}
        />
      )}
      <div className="relative z-10">
        <ResultIcon icon={result.icon} provider={result.provider} />
      </div>
      <div className="relative z-10 flex min-w-0 flex-1 flex-col">
        <span className="truncate text-[14px] font-medium text-white/95">{result.title}</span>
        {result.subtitle && (
          <span className="truncate text-[12px] text-white/55">{result.subtitle}</span>
        )}
      </div>
    </li>
  );
}
