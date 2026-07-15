use std::path::PathBuf;

use nowdocs::agent_contract::CapabilitySupport;
use nowdocs::clients::{
    all_adapters, approved_root, ClientExecutionOutcome, ClientExecutionRequest,
};

#[test]
fn execution_request_rejects_blank_operation_id_and_relative_binary() {
    let root_dir = tempfile::tempdir().unwrap();
    let root = approved_root(root_dir.path()).unwrap();

    assert!(ClientExecutionRequest::new("", root.clone(), PathBuf::from("nowdocs")).is_err());
    assert!(ClientExecutionRequest::new("op-1", root, PathBuf::from("nowdocs")).is_err());
}

#[test]
fn unsupported_execution_capabilities_default_to_unsupported() {
    let root_dir = tempfile::tempdir().unwrap();
    let root = approved_root(root_dir.path()).unwrap();
    let request =
        ClientExecutionRequest::new("op-1", root, root_dir.path().join("nowdocs")).unwrap();

    for adapter in all_adapters() {
        let capabilities = adapter.capabilities();
        if capabilities.apply == CapabilitySupport::Unsupported {
            assert_eq!(
                adapter.apply(&request).unwrap().outcome,
                ClientExecutionOutcome::Unsupported
            );
        }
        if capabilities.verify == CapabilitySupport::Unsupported {
            assert_eq!(
                adapter.verify(&request).unwrap().outcome,
                ClientExecutionOutcome::Unsupported
            );
        }
        if capabilities.apply == CapabilitySupport::Unsupported {
            assert_eq!(
                adapter.rollback(&request).unwrap().outcome,
                ClientExecutionOutcome::Unsupported
            );
        }
    }
    assert!(std::fs::read_dir(root_dir.path()).unwrap().next().is_none());
}
