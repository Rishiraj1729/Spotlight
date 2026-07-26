/// The parsed request handed to every provider. Kept intentionally tiny -
/// providers that need more context (e.g. a DB handle) get it injected at
/// construction time, not through the query.
#[derive(Debug, Clone)]
pub struct Query {
    pub raw: String,
}

impl Query {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        Self { raw }
    }

    pub fn trimmed(&self) -> &str {
        self.raw.trim()
    }

    pub fn is_empty(&self) -> bool {
        self.trimmed().is_empty()
    }
}
