//! Token counting via tiktoken cl100k_base (the GPT-4/3.5 BPE tokenizer).
//!
//! Used by the chunker (1c) to enforce token budgets. The BPE is loaded once
//! and cached for the process lifetime; `count_tokens` is cheap afterward.

use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

static BPE: OnceLock<Result<CoreBPE, String>> = OnceLock::new();

fn get_bpe() -> Result<&'static CoreBPE, &'static String> {
    BPE.get_or_init(|| tiktoken_rs::cl100k_base().map_err(|e| e.to_string()))
        .as_ref()
}

/// Count the number of cl100k_base tokens in `text`.
///
/// Uses `encode_ordinary` (no special-token handling) so documentation text is
/// tokenized literally — a `<|endoftext|>`-style marker in prose counts as
/// ordinary text, not a special token. Empty input returns 0.
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    match get_bpe() {
        Ok(b) => b.encode_ordinary(text).len(),
        Err(_) => fallback_estimate(text),
    }
}

fn fallback_estimate(text: &str) -> usize {
    text.split_whitespace().count() * 4 / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_estimate() {
        assert_eq!(fallback_estimate(""), 0);
        assert_eq!(fallback_estimate("hello world"), 2);
        assert_eq!(fallback_estimate("Rust is systems programming"), 5);
    }
}
