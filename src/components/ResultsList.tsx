import { AnimatePresence, motion } from 'framer-motion';
import type { SearchResult } from '../types/result';
import { ResultItem } from './ResultItem';

interface ResultsListProps {
  results: SearchResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onActivate: (result: SearchResult) => void;
}

export function ResultsList({ results, selectedIndex, onSelect, onActivate }: ResultsListProps) {
  if (results.length === 0) return null;

  return (
    <AnimatePresence>
      <motion.ul
        role="listbox"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.12 }}
        className="max-h-[360px] overflow-y-auto px-2 pb-2"
      >
        {results.map((result, index) => (
          <ResultItem
            key={result.id}
            result={result}
            isSelected={index === selectedIndex}
            onSelect={() => onSelect(index)}
            onActivate={() => onActivate(result)}
          />
        ))}
      </motion.ul>
    </AnimatePresence>
  );
}
