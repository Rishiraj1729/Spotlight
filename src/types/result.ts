export type ResultAction =
  | { type: 'launch'; path: string }
  | { type: 'openUri'; uri: string }
  | { type: 'copyToClipboard'; text: string }
  | { type: 'runSubQuery'; query: string };

export interface SearchResult {
  id: string;
  provider: string;
  title: string;
  subtitle?: string;
  icon: string;
  score: number;
  action: ResultAction;
}
