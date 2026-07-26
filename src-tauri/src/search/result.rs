use serde::{Deserialize, Serialize};

/// What happens when the user activates (Enter/click) a result.
/// This is the only thing the frontend needs to interpret - it never has to
/// know which provider produced the result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResultAction {
    Launch { path: String },
    OpenUri { uri: String },
    CopyToClipboard { text: String },
    RunSubQuery { query: String },
}

/// The single, uniform shape every provider must return. The SearchEngine
/// only ever deals with `SearchResult` - it never reaches into provider
/// internals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    /// Owned rather than `&'static str` - this struct crosses the IPC
    /// boundary (deserialized from the frontend on `launch`), which
    /// requires owned, non-borrowed field types.
    pub provider: String,
    pub title: String,
    pub subtitle: Option<String>,
    /// Either a symbolic name (e.g. "app", "calculator") that the frontend
    /// renders with a built-in glyph, or a `data:` URI for a real icon.
    pub icon: String,
    pub score: f32,
    pub action: ResultAction,
}
