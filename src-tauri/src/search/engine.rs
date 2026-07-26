use std::sync::Arc;

use rayon::prelude::*;

use crate::indexer::db::Db;

use super::provider::Provider;
use super::query::Query;
use super::result::SearchResult;

/// Maximum results shown - keeps ranking/serialization cheap and the UI
/// list scannable at a glance, matching the <100ms results budget.
const MAX_RESULTS: usize = 9;

/// The SearchEngine is deliberately dumb: it fans a query out to every
/// registered provider, merges the results, applies a small frecency boost,
/// sorts, and truncates. It never knows *how* a provider finds its data -
/// that's the whole point of the Provider trait boundary.
pub struct SearchEngine {
    providers: Vec<Box<dyn Provider>>,
    db: Arc<Db>,
}

impl SearchEngine {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            providers: Vec::new(),
            db,
        }
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) {
        self.providers.push(provider);
    }

    pub fn search(&self, raw_query: &str) -> Vec<SearchResult> {
        let query = Query::new(raw_query);
        if query.is_empty() {
            return Vec::new();
        }

        let mut results: Vec<SearchResult> = self
            .providers
            .par_iter()
            .flat_map(|provider| provider.search(&query))
            .collect();

        for result in &mut results {
            result.score += self.db.frecency_bonus(&result.id);
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(MAX_RESULTS);
        results
    }

    pub fn record_launch(&self, result_id: &str, provider: &str) {
        self.db.record_launch(result_id, provider);
    }
}
