//! Token counting via tiktoken cl100k_base (the GPT-4/3.5 BPE tokenizer).
//!
//! Used by the chunker (1c) to enforce token budgets. The BPE is loaded once
//! and cached for the process lifetime; `count_tokens` is cheap afterward.

use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

static BPE: OnceLock<CoreBPE> = OnceLock::new();

fn bpe() -> &'static CoreBPE {
    BPE.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("cl100k_base BPE must load; bundled with tiktoken-rs")
    })
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
    bpe().encode_ordinary(text).len()
}
