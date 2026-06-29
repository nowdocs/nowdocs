use nowdocs::input::{resolve_max_tokens, resolve_top_k, validate_docset, validate_query};

#[test]
fn docset_accepts_valid() {
    assert_eq!(validate_docset("nextjs").unwrap(), "nextjs");
    assert_eq!(validate_docset("react-18.v2").unwrap(), "react-18.v2");
    assert_eq!(validate_docset("a").unwrap(), "a");
}

#[test]
fn docset_rejects_uppercase() {
    assert!(validate_docset("Next.js").is_err());
}

#[test]
fn docset_rejects_path_traversal() {
    assert!(validate_docset("../etc").is_err());
    assert!(validate_docset("..").is_err());
    assert!(validate_docset("a..b").is_err());
}

#[test]
fn docset_rejects_too_long() {
    assert!(validate_docset(&"a".repeat(65)).is_err());
}

#[test]
fn docset_accepts_max_length() {
    assert!(validate_docset(&"a".repeat(64)).is_ok());
}

#[test]
fn docset_rejects_slash_and_space() {
    assert!(validate_docset("a/b").is_err());
    assert!(validate_docset("a b").is_err());
}

#[test]
fn query_accepts_boundary_length() {
    assert!(validate_query(&"x".repeat(4096)).is_ok());
}

#[test]
fn query_rejects_over_limit() {
    assert!(validate_query(&"x".repeat(4097)).is_err());
}

#[test]
fn query_rejects_empty() {
    assert!(validate_query("").is_err());
}

#[test]
fn resolve_max_tokens_default_and_clamp() {
    assert_eq!(resolve_max_tokens(None), 4000);
    assert_eq!(resolve_max_tokens(Some(100)), 100);
    assert_eq!(resolve_max_tokens(Some(99999)), 4000); // clamp to cap
    assert_eq!(resolve_max_tokens(Some(0)), 4000); // 0 treated as unset → default
}

#[test]
fn resolve_top_k_default_and_clamp() {
    assert_eq!(resolve_top_k(None), 5);
    assert_eq!(resolve_top_k(Some(3)), 3);
    assert_eq!(resolve_top_k(Some(0)), 1); // clamp to floor
    assert_eq!(resolve_top_k(Some(21)), 20); // clamp to ceiling
    assert_eq!(resolve_top_k(Some(20)), 20);
    assert_eq!(resolve_top_k(Some(1)), 1);
}
