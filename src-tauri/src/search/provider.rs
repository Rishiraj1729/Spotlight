use super::query::Query;
use super::result::SearchResult;

/// Every search source (apps, files, folders, settings, calculator...)
/// implements this trait. The SearchEngine only ever talks to providers
/// through this interface, so providers stay fully independent modules and
/// can be added/removed without touching the engine.
pub trait Provider: Send + Sync {
    /// Stable identifier, also used as the `provider` field on results.
    fn id(&self) -> &'static str;

    /// Relative priority when scores tie (higher runs first in display order
    /// for equal fuzzy scores). Calculator/App should generally outrank
    /// Files for the same textual match quality.
    fn priority(&self) -> f32 {
        0.0
    }

    /// Synchronous, expected to be fast (<10ms typical). Providers backed by
    /// slower I/O (disk index) should query pre-built in-memory/SQLite
    /// structures rather than doing live filesystem walks here.
    fn search(&self, query: &Query) -> Vec<SearchResult>;
}
