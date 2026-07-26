use std::sync::Arc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::indexer::Db;
use crate::search::{Provider, Query, ResultAction, SearchResult};

const CANDIDATE_LIMIT: usize = 25;
const RESULT_LIMIT: usize = 5;

/// Same index as `FileProvider`, filtered to directories. Kept as a
/// separate provider (per the architecture) so folders can be ranked,
/// throttled, or disabled independently of file results.
pub struct FolderProvider {
    db: Arc<Db>,
    matcher: SkimMatcherV2,
}

impl FolderProvider {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            db,
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl Provider for FolderProvider {
    fn id(&self) -> &'static str {
        "folder"
    }

    fn priority(&self) -> f32 {
        6.0
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        let candidates = self.db.search_folders(input, CANDIDATE_LIMIT);

        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|entry| {
                let score = self.matcher.fuzzy_match(&entry.name, input)?;
                Some(SearchResult {
                    id: format!("folder:{}", entry.path),
                    provider: self.id().to_string(),
                    title: entry.name,
                    subtitle: Some(entry.path.clone()),
                    icon: "folder".to_string(),
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
