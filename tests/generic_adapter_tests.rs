//! Tests for the generic MCP client adapter.
//!
//! The generic adapter is generation-only. Detection and automatic application
//! are unsupported. `generate` must produce deterministic stdio argv plus a
//! redacted JSON fragment and explicit manual copy / restart steps.

use std::path::PathBuf;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{
    all_adapters, approved_root, ClientAdapter, ClientExecutionOutcome, ClientExecutionRequest,
    ClientId,
};

fn generic() -> Box<dyn ClientAdapter> {
    all_adapters()
        .into_iter()
        .find(|a| a.id() == ClientId::Generic)
        .expect("generic adapter must be present in `all_adapters()`")
}

#[test]
fn generic_adapter_has_fixed_id_and_capability_matrix() {
    let a = generic();
    assert_eq!(a.id(), ClientId::Generic);
    let caps = a.capabilities();
    assert_eq!(caps.detect, CapabilitySupport::Unsupported);
    assert_eq!(caps.generate, CapabilitySupport::Supported);
    assert_eq!(caps.apply, CapabilitySupport::Unsupported);
    assert_eq!(caps.verify, CapabilitySupport::Unsupported);
}

#[test]
fn generic_adapter_generate_uses_exact_absolute_argv() {
    let a = generic();
    let binary = PathBuf::from("/absolute/nowdocs");
    let cfg = a.generate(&binary).unwrap();
    assert_eq!(cfg.stdio_command, vec!["/absolute/nowdocs", "serve"]);
}

#[test]
fn generic_adapter_generate_redacts_binary_path_in_fragment() {
    let a = generic();
    let cfg = a.generate(&PathBuf::from("/absolute/nowdocs")).unwrap();
    let frag = &cfg.redacted_fragment;
    assert!(
        frag.contains("\"<binary>\""),
        "redacted fragment must use the <binary> placeholder, got: {frag}"
    );
    assert!(
        !frag.contains("/absolute/nowdocs"),
        "redacted fragment must not contain an absolute binary path, got: {frag}"
    );
}

#[test]
fn generic_adapter_manual_steps_require_copy_and_restart() {
    let a = generic();
    let cfg = a.generate(&PathBuf::from("/absolute/nowdocs")).unwrap();
    let joined = cfg.manual_steps.join("\n").to_lowercase();
    assert!(
        joined.contains("copy"),
        "manual steps must instruct the user to copy the configuration, got: {:?}",
        cfg.manual_steps
    );
    assert!(
        joined.contains("restart") || joined.contains("reload"),
        "manual steps must instruct the user to restart or reload the client, got: {:?}",
        cfg.manual_steps
    );
}

#[test]
fn generic_adapter_manual_steps_do_not_name_unsupported_clients() {
    let a = generic();
    let cfg = a.generate(&PathBuf::from("/absolute/nowdocs")).unwrap();
    let joined = cfg.manual_steps.join("\n").to_lowercase();
    for forbidden in [
        "claude code",
        "claude desktop",
        "cursor",
        "aider",
        "registered",
        "verified",
        "applied",
    ] {
        assert!(
            !joined.contains(forbidden),
            "manual steps must not mention `{forbidden}`, got: {:?}",
            cfg.manual_steps
        );
    }
}

#[test]
fn generic_adapter_observations_contain_no_absolute_paths_or_secrets() {
    let a = generic();
    let root_dir = tempfile::tempdir().unwrap();
    let root = approved_root(root_dir.path()).unwrap();
    let request = ClientExecutionRequest::new(
        "op-generic-1",
        root.clone(),
        root_dir.path().join("nowdocs"),
    )
    .unwrap();

    let detect = a.detect(&root).unwrap();
    for line in &detect.observations {
        assert!(
            !line.contains("/Users/"),
            "detection observation leaks absolute path: {line}"
        );
    }

    let apply = a.apply(&request).unwrap();
    assert_eq!(apply.outcome, ClientExecutionOutcome::Unsupported);
    for line in &apply.observations {
        assert!(
            !line.contains("/Users/"),
            "apply observation leaks absolute path: {line}"
        );
        assert!(
            !line.contains(root_dir.path().to_str().unwrap()),
            "apply observation leaks approved root: {line}"
        );
    }

    let verify = a.verify(&request).unwrap();
    assert_eq!(verify.outcome, ClientExecutionOutcome::Unsupported);
    for line in &verify.observations {
        assert!(
            !line.contains("/Users/"),
            "verify observation leaks absolute path: {line}"
        );
        assert!(
            !line.contains(root_dir.path().to_str().unwrap()),
            "verify observation leaks approved root: {line}"
        );
    }

    let rollback = a.rollback(&request).unwrap();
    assert_eq!(rollback.outcome, ClientExecutionOutcome::Unsupported);
    for line in &rollback.observations {
        assert!(
            !line.contains("/Users/"),
            "rollback observation leaks absolute path: {line}"
        );
        assert!(
            !line.contains(root_dir.path().to_str().unwrap()),
            "rollback observation leaks approved root: {line}"
        );
    }
}

#[test]
fn generic_adapter_does_not_mutate_filesystem_under_unsupported_operations() {
    let a = generic();
    let root_dir = tempfile::tempdir().unwrap();
    let root = approved_root(root_dir.path()).unwrap();
    let request =
        ClientExecutionRequest::new("op-generic-2", root, root_dir.path().join("nowdocs")).unwrap();

    let _ = a.apply(&request).unwrap();
    let _ = a.verify(&request).unwrap();
    let _ = a.rollback(&request).unwrap();

    assert!(
        std::fs::read_dir(root_dir.path()).unwrap().next().is_none(),
        "generic adapter must not create files under the approved root"
    );
}
