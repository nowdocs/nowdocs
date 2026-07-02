//! Code-aware markdown chunker.
//!
//! Splits documentation markdown into retrieval chunks with these guarantees:
//! - Heading path is tracked as a stack and attached to every chunk
//!   (`"H1 > H2 > H3"`), and prefixed into the chunk text for context.
//! - Fenced code blocks (``` ``` or `~~~`) are **never** split mid-fence — a
//!   code block larger than `target_tokens` becomes its own chunk and is
//!   allowed to exceed `max_tokens`.
//! - Prose is split at paragraph/sentence/word boundaries so every prose
//!   chunk stays at or under `max_tokens`.
//! - `chunk_type` = `Code` for a chunk whose body is majority fenced code,
//!   else `Info`.

use crate::token::count_tokens;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    Code,
    Info,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub idx: u32,
    pub heading_path: String,
    pub source_url: String,
    pub api_version: Option<String>,
    pub chunk_type: ChunkType,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub min_tokens: u32,
    pub max_tokens: u32,
    pub target_tokens: u32,
    pub window_tokens: u32,
}

pub fn default_config() -> ChunkConfig {
    ChunkConfig {
        min_tokens: 256,
        max_tokens: 512,
        target_tokens: 384,
        window_tokens: 2048,
    }
}

/// A contiguous raw block extracted from the markdown: either a fenced code
/// block (kept whole) or a run of prose lines under the current heading.
enum RawBlock {
    Code { path: String, body: String },
    Prose { path: String, body: String },
}

/// Detect a markdown ATX heading (`# .. ######`) and return its level + text.
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.starts_with(' ') {
        return None;
    }
    let text = rest.trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some((hashes, text))
}

/// Detect a fence open/close marker (``` or ~~~) and return its char length.
fn fence_marker(line: &str) -> Option<usize> {
    let t = line.trim_start();
    t.strip_prefix("```")
        .map(|rest| 3 + rest.chars().take_while(|&c| c == '`').count())
        .or_else(|| {
            t.strip_prefix("~~~")
                .map(|rest| 3 + rest.chars().take_while(|&c| c == '~').count())
        })
}

fn is_fence_line(line: &str) -> bool {
    fence_marker(line).is_some()
}

pub fn chunk_markdown(md: &str, cfg: &ChunkConfig) -> Vec<Chunk> {
    if md.trim().is_empty() {
        return Vec::new();
    }

    // --- Pass 1: split into raw blocks (code blocks whole, prose grouped) ---
    let mut blocks: Vec<RawBlock> = Vec::new();
    let mut heading_stack: Vec<String> = Vec::new();
    let mut prose_buf = String::new();
    let mut in_code = false;

    let flush_prose = |buf: &mut String, blocks: &mut Vec<RawBlock>, stack: &[String]| {
        if !buf.trim().is_empty() {
            blocks.push(RawBlock::Prose {
                path: stack.join(" > "),
                body: std::mem::take(buf),
            });
        } else {
            buf.clear();
        }
    };

    for line in md.split_inclusive('\n') {
        if in_code {
            prose_buf.push_str(line);
            if is_fence_line(line) {
                in_code = false;
                // code block complete — emit it as its own raw block.
                let body = std::mem::take(&mut prose_buf);
                blocks.push(RawBlock::Code {
                    path: heading_stack.join(" > "),
                    body,
                });
            }
            continue;
        }

        if let Some((level, text)) = parse_heading(line) {
            // heading boundary → close any open prose block, reset stack.
            flush_prose(&mut prose_buf, &mut blocks, &heading_stack);
            heading_stack.truncate(level - 1);
            while heading_stack.len() < level - 1 {
                heading_stack.push(String::new());
            }
            heading_stack.push(text);
            continue;
        }

        if is_fence_line(line) {
            flush_prose(&mut prose_buf, &mut blocks, &heading_stack);
            prose_buf.push_str(line);
            in_code = true;
            continue;
        }

        prose_buf.push_str(line);
    }

    if in_code {
        // unterminated fence — emit as a code block anyway.
        let body = std::mem::take(&mut prose_buf);
        blocks.push(RawBlock::Code {
            path: heading_stack.join(" > "),
            body,
        });
    } else {
        flush_prose(&mut prose_buf, &mut blocks, &heading_stack);
    }

    // --- Pass 2: turn raw blocks into chunks ---
    let max = cfg.max_tokens as usize;
    let target = cfg.target_tokens as usize;

    let mut chunks: Vec<Chunk> = Vec::new();
    let mut idx: u32 = 0;

    let emit =
        |chunks: &mut Vec<Chunk>, path: &str, ctype: ChunkType, text: String, idx: &mut u32| {
            chunks.push(Chunk {
                idx: *idx,
                heading_path: path.to_string(),
                source_url: String::new(),
                api_version: None,
                chunk_type: ctype,
                text,
            });
            *idx += 1;
        };

    for block in blocks {
        match block {
            RawBlock::Code { path, body } => {
                let text = with_path_prefix(&path, &body);
                emit(&mut chunks, &path, ChunkType::Code, text, &mut idx);
            }
            RawBlock::Prose { path, body } => {
                // The heading-path prefix counts toward the chunk's token
                // budget, so reserve its overhead before splitting the body.
                let prefix = path_prefix(&path);
                let overhead = count_tokens(&prefix);
                let eff_max = max.saturating_sub(overhead).max(1);
                let eff_target = target.saturating_sub(overhead).min(eff_max);
                let pieces = split_prose(&body, eff_target, eff_max);
                for piece in pieces {
                    let text = format!("{prefix}{piece}");
                    emit(&mut chunks, &path, ChunkType::Info, text, &mut idx);
                }
            }
        }
    }

    chunks
}

fn path_prefix(path: &str) -> String {
    if path.is_empty() {
        String::new()
    } else {
        format!("## {path}\n\n")
    }
}

fn with_path_prefix(path: &str, body: &str) -> String {
    let prefix = path_prefix(path);
    format!("{prefix}{body}")
}

/// Split a prose block into pieces each at or under `max` tokens, targeting
/// `target` tokens per piece. Splits prefer paragraph → sentence → word →
/// character boundaries (in that order) so prose stays readable.
fn split_prose(body: &str, target: usize, max: usize) -> Vec<String> {
    let paragraphs: Vec<&str> = body
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();

    let push_buf = |buf: &mut String, out: &mut Vec<String>| {
        if !buf.trim().is_empty() {
            out.push(std::mem::take(buf));
        } else {
            buf.clear();
        }
    };

    for para in paragraphs {
        let para = para.trim();
        let para_tokens = count_tokens(para);

        if para_tokens > max {
            // paragraph itself too big — flush current buf, then sub-split.
            push_buf(&mut buf, &mut out);
            for piece in split_long(para, max) {
                out.push(piece);
            }
            continue;
        }

        let would_be = if buf.is_empty() {
            para_tokens
        } else {
            count_tokens(&buf) + para_tokens
        };

        if !buf.is_empty() && would_be > target {
            push_buf(&mut buf, &mut out);
        }
        if !buf.is_empty() {
            buf.push_str("\n\n");
        }
        buf.push_str(para);
    }
    push_buf(&mut buf, &mut out);
    out
}

/// Split a too-long paragraph into pieces each ≤ `max` tokens, preferring
/// sentence boundaries, then words, then characters.
fn split_long(text: &str, max: usize) -> Vec<String> {
    let sentences = split_sentences(text);
    let mut out = Vec::new();
    let mut buf = String::new();

    for sent in sentences {
        let stoks = count_tokens(&sent);
        if stoks > max {
            if !buf.trim().is_empty() {
                out.push(std::mem::take(&mut buf));
            }
            out.extend(split_by_words(&sent, max));
            continue;
        }
        let would_be = if buf.is_empty() {
            stoks
        } else {
            count_tokens(&buf) + stoks
        };
        if !buf.is_empty() && would_be > max {
            out.push(std::mem::take(&mut buf));
        }
        buf.push_str(&sent);
    }
    if !buf.trim().is_empty() {
        out.push(buf);
    }
    out
}

/// Split on sentence terminators (`. `, `! `, `? `) plus newlines, keeping the
/// terminator attached to the preceding sentence.
fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        cur.push(c);
        if c == '.' || c == '!' || c == '?' {
            // include trailing spaces / quotes, then break at next word.
            while let Some(&n) = chars.peek() {
                if n == ' ' || n == '\t' || n == '\n' {
                    cur.push(n);
                    chars.next();
                } else {
                    break;
                }
            }
            if !cur.trim().is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

/// Split by whitespace-delimited words, packing to `max` tokens; a single word
/// over `max` is hard-split by characters until its token count fits.
fn split_by_words(text: &str, max: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    let mut buf = String::new();
    for w in words {
        let wtoks = count_tokens(w);
        if wtoks > max {
            if !buf.is_empty() {
                out.push(std::mem::take(&mut buf));
            }
            out.extend(split_by_chars(w, max));
            continue;
        }
        let would_be = if buf.is_empty() {
            wtoks
        } else {
            count_tokens(&buf) + wtoks
        };
        if !buf.is_empty() && would_be > max {
            out.push(std::mem::take(&mut buf));
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(w);
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

/// Hard-split a single over-long word by characters so each piece is ≤ `max`
/// tokens. Greedily grows the piece and checks the token count.
fn split_by_chars(word: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in word.chars() {
        cur.push(c);
        if count_tokens(&cur) >= max {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
