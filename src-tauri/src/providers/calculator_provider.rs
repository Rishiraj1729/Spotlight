use crate::search::{Provider, Query, ResultAction, SearchResult};

/// Instant, no I/O, no index - just tries to evaluate the query as a math
/// expression. Always the highest-priority provider when it produces a
/// result, since a user typing `12*7` clearly wants the answer, not a file
/// named "12".
pub struct CalculatorProvider;

impl CalculatorProvider {
    pub fn new() -> Self {
        Self
    }

    /// Cheap heuristic so we don't waste time trying to parse "notepad" as
    /// an expression, and so plain filenames with numbers don't get treated
    /// as math.
    fn looks_like_math(input: &str) -> bool {
        let has_digit = input.chars().any(|c| c.is_ascii_digit());
        let has_operator = input.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')' | '%'));
        has_digit && has_operator
    }
}

impl Provider for CalculatorProvider {
    fn id(&self) -> &'static str {
        "calculator"
    }

    fn priority(&self) -> f32 {
        50.0
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        if !Self::looks_like_math(input) {
            return Vec::new();
        }

        let Ok(value) = meval::eval_str(input) else {
            return Vec::new();
        };
        if !value.is_finite() {
            return Vec::new();
        }

        let formatted = format_number(value);
        vec![SearchResult {
            id: format!("calculator:{formatted}"),
            provider: self.id().to_string(),
            title: formatted.clone(),
            subtitle: Some(format!("Calculator - {input} =")),
            icon: "calculator".to_string(),
            score: 1000.0,
            action: ResultAction::CopyToClipboard { text: formatted },
        }]
    }
}

fn format_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{value:.0}")
    } else {
        let rounded = (value * 1e10).round() / 1e10;
        rounded.to_string()
    }
}
