use std::sync::Arc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::indexer::Db;
use crate::search::{Provider, Query, ResultAction, SearchResult};

const CANDIDATE_LIMIT: usize = 25;
const RESULT_LIMIT: usize = 6;

/// Queries the shared SQLite/FTS5 index built by the crawler + watcher.
/// Deliberately has zero knowledge of *how* the index is populated - that's
/// the crawler/watcher's job (see `indexer/`).
pub struct FileProvider {
    db: Arc<Db>,
    matcher: SkimMatcherV2,
}

impl FileProvider {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            db,
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl Provider for FileProvider {
    fn id(&self) -> &'static str {
        "file"
    }

    fn priority(&self) -> f32 {
        5.0
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        let candidates = self.db.search_files(input, CANDIDATE_LIMIT);

        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|entry| {
                let score = self.matcher.fuzzy_match(&entry.name, input)?;
                Some(SearchResult {
                    id: format!("file:{}", entry.path),
                    provider: self.id().to_string(),
                    title: entry.name,
                    subtitle: Some(entry.path.clone()),
                    icon: "file".to_string(),
                    score: score as f32 + self.priority(),
                    action: ResultAction::Launch { path: entry.path },
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(RESULT_LIMIT);
        results
    }
}
