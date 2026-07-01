//! Prompt-injection + danger-flag sanitizer for retrieved chunks.
//!
//! Retrieval returns text authored by third parties (doc authors). Before that
//! text is handed to the model as context, it passes through `sanitize_chunk`:
//!
//! 1. strip HTML comments `<!-- ... -->`
//! 2. strip zero-width / invisible chars
//! 3. strip `display:none` elements (hidden injection vectors)
//! 4. strip assistant-override phrases (case-insensitive)
//! 5. strip danger-flag tokens (`-y`, `--yes`, `--force`, `sudo`, `rm -rf`)
//!
//! `sanitize_metadata` is lighter: zero-width removal + a length cap. Metadata
//! is short and trusted-ish; full HTML/phrase stripping is unnecessary there.

use std::sync::OnceLock;

use regex::Regex;

const METADATA_MAX_CHARS: usize = 500;

static RE_COMMENT: OnceLock<Regex> = OnceLock::new();
static RE_DISPLAY_NONE: OnceLock<Regex> = OnceLock::new();
static RE_OVERRIDE: OnceLock<Regex> = OnceLock::new();
static RE_DANGER: OnceLock<Regex> = OnceLock::new();

fn re_comment() -> &'static Regex {
    RE_COMMENT.get_or_init(|| Regex::new(r"(?s)<!--.*?-->").unwrap())
}

fn re_display_none() -> &'static Regex {
    RE_DISPLAY_NONE.get_or_init(|| {
        // (?s) so `.*?` spans newlines; matches a hidden element + its close tag.
        Regex::new(r"(?s)<[^>]*display:\s*none[^>]*>.*?</[^>]+>").unwrap()
    })
}

fn re_override() -> &'static Regex {
    RE_OVERRIDE.get_or_init(|| {
        Regex::new(
            r"(?i)ignore (?:previous|prior) instructions|note for the assistant|you (?:may|can) (?:run|execute)|as an ai|system prompt",
        )
        .unwrap()
    })
}

fn re_danger() -> &'static Regex {
    // Danger flags only as standalone tokens (preceded by start/space), so a
    // flag substring inside a version string like "forceful" is not touched.
    RE_DANGER.get_or_init(|| Regex::new(r"(^|\s)(?:-y|--yes|--force|sudo|rm\s+-rf)\b").unwrap())
}

fn strip_zero_width(s: &str) -> String {
    s.chars()
        .filter(|&c| {
            !matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{2060}'
            )
        })
        .collect()
}

pub fn sanitize_chunk(text: &str) -> String {
    let s = re_comment().replace_all(text, "").into_owned();
    let s = re_display_none().replace_all(&s, "").into_owned();
    let s = strip_zero_width(&s);
    let s = re_override().replace_all(&s, "").into_owned();
    // Collapse any whitespace left behind by removed flags/phrases.
    let s = re_danger().replace_all(&s, " ").into_owned();
    collapse_whitespace(&s)
}

pub fn sanitize_metadata(text: &str) -> String {
    let s = strip_zero_width(text);
    let s = s.chars().take(METADATA_MAX_CHARS).collect::<String>();
    collapse_whitespace(&s)
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}
