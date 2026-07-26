use crate::search::{Provider, Query, ResultAction, SearchResult};

const WEB_SCORE: f32 = 3.0;

/// One fallback row: opens Google search in the user's default browser.
/// No network from Spotlight — only builds a URL and hands off via OpenUri.
pub struct WebSearchProvider;

impl WebSearchProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for WebSearchProvider {
    fn id(&self) -> &'static str {
        "web"
    }

    fn priority(&self) -> f32 {
        WEB_SCORE
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        if input.is_empty() {
            return Vec::new();
        }

        let encoded = urlencoding::encode(input);
        let uri = format!("https://www.google.com/search?q={encoded}");

        vec![SearchResult {
            id: format!("web:{input}"),
            provider: self.id().to_string(),
            title: format!("Search the web for \"{input}\""),
            subtitle: Some("Browser".to_string()),
            icon: "web".to_string(),
            score: WEB_SCORE,
            action: ResultAction::OpenUri { uri },
        }]
    }
}
