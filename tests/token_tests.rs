use nowdocs::token::count_tokens;

#[test]
fn empty_string_is_zero() {
    assert_eq!(count_tokens(""), 0);
}

#[test]
fn short_string_in_sane_range() {
    let n = count_tokens("hello world");
    assert!(
        n > 0 && n < 10,
        "hello world should be 1..10 tokens, got {}",
        n
    );
}

#[test]
fn deterministic_same_input_same_count() {
    let a = count_tokens("how to use clerkMiddleware in next.js");
    let b = count_tokens("how to use clerkMiddleware in next.js");
    assert_eq!(a, b);
}

#[test]
fn longer_text_has_more_tokens_than_shorter() {
    let short = count_tokens("rust");
    let long = count_tokens("Rust is a systems programming language that runs blazingly fast");
    assert!(long > short, "longer text should have more tokens");
}
