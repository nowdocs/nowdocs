//! C6B — Claude Desktop guidance adapter tests.
//!
//! These tests pin the v0.2 capability matrix, the deterministic `.mcpb`
//! guidance, the explicit refusal of legacy `claude_desktop_config.json`
//! instructions, and the inherited unsupported apply/verify/rollback
//! behaviour. Tests inspect only temporary roots; no real client
//! configuration files, processes, network calls, or user configuration
//! are created or used.

use std::fs;
use std::path::PathBuf;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{
    all_adapters, approved_root, ClientExecutionOutcome, ClientExecutionRequest, ClientId,
};
use nowdocs::clients::{ClientAdapter, Detection, GeneratedConfig};

fn adapter() -> Box<dyn ClientAdapter> {
    for candidate in all_adapters() {
        if candidate.id() == ClientId::ClaudeDesktop {
            return candidate;
        }
    }
    panic!("claude-desktop adapter missing from all_adapters()");
}

fn make_request(root_dir: &tempfile::TempDir) -> ClientExecutionRequest {
    let root = approved_root(root_dir.path()).expect("approved root");
    let binary = root_dir.path().join("nowdocs");
    ClientExecutionRequest::new("op-c6b-desktop", root, binary).expect("valid request")
}

#[test]
fn adapter_keeps_locked_claude_desktop_identity() {
    let adapter = adapter();
    assert_eq!(adapter.id(), ClientId::ClaudeDesktop);
    assert_eq!(adapter.id().as_str(), "claude-desktop");
}

#[test]
fn capability_matrix_matches_v02_guidance_only_policy() {
    let caps = adapter().capabilities();
    assert_eq!(caps.detect, CapabilitySupport::Supported);
    assert_eq!(caps.generate, CapabilitySupport::Supported);
    assert_eq!(caps.apply, CapabilitySupport::Unsupported);
    assert_eq!(caps.verify, CapabilitySupport::Unsupported);
}

#[test]
fn detection_in_approved_root_is_deterministic_and_observation_free() {
    let root_dir = tempfile::tempdir().expect("temp dir");
    let root = approved_root(root_dir.path()).expect("approved root");
    let adapter = adapter();

    let first: Detection = adapter.detect(&root).expect("detect first");
    let second: Detection = adapter.detect(&root).expect("detect second");
    assert_eq!(first, second, "detection must be deterministic");

    // Deterministic observations explain the policy without absolute paths,
    // configuration paths, user names, tokens, or secrets.
    assert!(
        first.observations.iter().all(|o| !o.contains('/')),
        "observations must not include absolute or relative paths: {:?}",
        first.observations
    );
    for forbidden in [
        "/Users/",
        "/home/",
        "C:\\",
        "claude_desktop_config.json",
        "HOME",
        "USER",
        "TOKEN",
    ] {
        assert!(
            !first.observations.iter().any(|o| o.contains(forbidden)),
            "observations must not contain {forbidden:?}: {:?}",
            first.observations
        );
    }
}

#[test]
fn detection_marks_presence_when_desktop_marker_exists_in_root() {
    let root_dir = tempfile::tempdir().expect("temp dir");
    let desktop_root = root_dir.path().join("Claude");
    fs::create_dir_all(&desktop_root).expect("create desktop root");
    fs::write(desktop_root.join("claude_desktop_config.json"), "{}").expect("write marker");

    let root = approved_root(root_dir.path()).expect("approved root");
    let detection = adapter().detect(&root).expect("detect");
    assert!(detection.detected, "marker file should register detection");
}

#[test]
fn detection_reports_absent_when_no_marker_is_present() {
    let root_dir = tempfile::tempdir().expect("temp dir");
    let root = approved_root(root_dir.path()).expect("approved root");
    let detection = adapter().detect(&root).expect("detect");
    assert!(!detection.detected, "empty root must not report detection");
}

#[test]
fn generate_emits_stdio_command_with_serve_argument() {
    let binary = PathBuf::from("/opt/nowdocs/bin/nowdocs");
    let cfg: GeneratedConfig = adapter().generate(&binary).expect("generate");

    assert_eq!(cfg.stdio_command.len(), 2);
    assert_eq!(cfg.stdio_command[0], "/opt/nowdocs/bin/nowdocs");
    assert_eq!(cfg.stdio_command[1], "serve");
}

#[test]
fn generate_redacted_fragment_explains_mcpb_only_path_and_avoids_legacy_json() {
    let binary = PathBuf::from("/opt/nowdocs/bin/nowdocs");
    let cfg = adapter().generate(&binary).expect("generate");

    // Must mention the .mcpb extension route and the Settings > Extensions surface.
    assert!(
        cfg.manual_steps
            .iter()
            .any(|s| s.to_lowercase().contains(".mcpb")),
        "manual steps must mention .mcpb: {:?}",
        cfg.manual_steps
    );
    assert!(
        cfg.manual_steps
            .iter()
            .any(|s| s.to_lowercase().contains("extensions")),
        "manual steps must mention Settings > Extensions: {:?}",
        cfg.manual_steps
    );

    // Must NOT suggest legacy raw-JSON configuration.
    for forbidden in [
        "claude_desktop_config.json",
        "mcpServers",
        "\"command\"",
        "\"args\"",
        "\"stdio\"",
    ] {
        let mut found_in_steps = false;
        for step in &cfg.manual_steps {
            if step.contains(forbidden) {
                found_in_steps = true;
                break;
            }
        }
        let in_fragment = cfg.redacted_fragment.contains(forbidden);
        assert!(
            !found_in_steps && !in_fragment,
            "guidance must not contain {forbidden:?}: steps={:?} fragment={}",
            cfg.manual_steps,
            cfg.redacted_fragment
        );
    }

    // Redacted fragment must not embed the absolute binary path or any
    // secret-bearing or user-identifying value.
    assert!(
        !cfg.redacted_fragment.contains("/opt/nowdocs"),
        "redacted fragment must not embed absolute binary path: {}",
        cfg.redacted_fragment
    );
    for forbidden in ["token", "secret", "USER", "HOME"] {
        assert!(
            !cfg.redacted_fragment
                .to_lowercase()
                .contains(&forbidden.to_lowercase()),
            "redacted fragment must not contain {forbidden:?}: {}",
            cfg.redacted_fragment
        );
    }
}

#[test]
fn generate_states_signed_cross_platform_mcpb_is_future_work() {
    let binary = PathBuf::from("/opt/nowdocs/bin/nowdocs");
    let cfg = adapter().generate(&binary).expect("generate");

    let combined =
        format!("{} {}", cfg.manual_steps.join(" "), cfg.redacted_fragment).to_lowercase();
    assert!(
        combined.contains("future") || combined.contains("not yet") || combined.contains("later"),
        "guidance must mark a signed cross-platform MCPB as future work: {}",
        combined
    );
}

#[test]
fn generate_does_not_claim_installation_or_loading_or_verification() {
    let binary = PathBuf::from("/opt/nowdocs/bin/nowdocs");
    let cfg = adapter().generate(&binary).expect("generate");

    let combined =
        format!("{} {}", cfg.manual_steps.join(" "), cfg.redacted_fragment).to_lowercase();
    // The guidance must not claim that nowdocs is installed, loaded,
    // verified, active, configured, or ready to use. Phrases that describe
    // a future user action (e.g., "user-installed .mcpb") are allowed
    // because they do not claim current state.
    for forbidden in [
        "nowdocs is installed",
        "nowdocs is loaded",
        "nowdocs is verified",
        "is now installed",
        "is now loaded",
        "is now verified",
        "ready to use",
        "active nowdocs",
    ] {
        assert!(
            !combined.contains(forbidden),
            "guidance must not claim {forbidden:?}: {combined}"
        );
    }
}

#[test]
fn generate_output_is_deterministic_for_same_binary() {
    let binary = PathBuf::from("/opt/nowdocs/bin/nowdocs");
    let first = adapter().generate(&binary).expect("first");
    let second = adapter().generate(&binary).expect("second");
    assert_eq!(first, second, "generate must be deterministic");
    assert_eq!(first.stdio_command, second.stdio_command);
    assert_eq!(first.redacted_fragment, second.redacted_fragment);
    assert_eq!(first.manual_steps, second.manual_steps);
}

#[test]
fn apply_verify_rollback_remain_inherited_unsupported() {
    let root_dir = tempfile::tempdir().expect("temp dir");
    let request = make_request(&root_dir);
    let adapter = adapter();

    let apply = adapter.apply(&request).expect("apply");
    assert_eq!(apply.outcome, ClientExecutionOutcome::Unsupported);
    assert!(
        apply.observations.iter().all(|o| !o.is_empty()),
        "unsupported observations must explain the policy"
    );

    let verify = adapter.verify(&request).expect("verify");
    assert_eq!(verify.outcome, ClientExecutionOutcome::Unsupported);

    let rollback = adapter.rollback(&request).expect("rollback");
    assert_eq!(rollback.outcome, ClientExecutionOutcome::Unsupported);

    // No real client configuration was created or modified.
    assert!(fs::read_dir(root_dir.path()).unwrap().next().is_none());
}
