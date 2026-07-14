//! C3 automation plan storage tests.
//!
//! C3-R1: tests use a poison-resistant `EnvGuard` (static mutex + save/restore
//! `XDG_CACHE_HOME` in Drop) so they are hermetic under explicit parallelism
//! (`--test-threads=4`). No test depends on `RUST_TEST_THREADS=1`.
//!
//! No test here reads a real cache, client config, model, or network.

use std::sync::Mutex;

use nowdocs::automation::plan::{
    self, AutomationPlan, DocsetPrecondition, PlanInputs, PlanPreconditions, PlannedAction,
    RiskLevel, TargetFilePrecondition, PLAN_TTL_SECS,
};

// C3-R1: env-mutation guard. A static mutex serializes XDG_CACHE_HOME access
// across tests; Drop restores the prior value. A poisoned mutex is recovered
// so subsequent tests can still run.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _g: std::sync::MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let old = std::env::var(key).ok();
        std::env::set_var(key, val);
        Self { key, old, _g: g }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

/// Build a minimal valid plan with a deterministic `created_at` so tests
/// control time. Two actions, one docset precondition, one target file.
fn sample_plan(created_at: u64) -> AutomationPlan {
    plan::new_plan(
        PlanInputs {
            client: Some("cursor".to_string()),
            docset: Some("nextjs".to_string()),
            online: true,
        },
        PlanPreconditions {
            cache_layout: "1".to_string(),
            model_present: true,
            docset_state: vec![DocsetPrecondition {
                docset: "nextjs".to_string(),
                installed: false,
                manifest_sha256: None,
            }],
            target_files: vec![TargetFilePrecondition {
                logical_id: "cursor-mcp-json".to_string(),
                exists: false,
                sha256: None,
            }],
        },
        vec![
            PlannedAction {
                id: "prepare-model".to_string(),
                kind: "model_download".to_string(),
                risk: RiskLevel::Additive,
                summary: "Download the pinned embedding model".to_string(),
                changes_state: true,
                network_access: true,
                requires_confirmation: true,
                reversible: true,
                target_paths: vec![],
                estimated_download_bytes: Some(69206016),
            },
            PlannedAction {
                id: "install-nextjs".to_string(),
                kind: "docset_install".to_string(),
                risk: RiskLevel::Additive,
                summary: "Install the nextjs docset".to_string(),
                changes_state: true,
                network_access: true,
                requires_confirmation: true,
                reversible: true,
                target_paths: vec![],
                estimated_download_bytes: Some(1024),
            },
        ],
        created_at,
    )
    .expect("sample plan must construct")
}

// --- Test 1: deterministic plan id for equivalent semantics ---

#[test]
fn plan_id_is_deterministic_for_equivalent_semantics() {
    let base = sample_plan(1_700_000_000);
    let id_base = plan::plan_id(&base).expect("plan_id");

    // 64 lowercase ASCII hex.
    assert_eq!(id_base.len(), 64, "plan id must be 64 lowercase hex chars");
    assert!(
        id_base
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "plan id must be lowercase hex, got: {id_base}"
    );

    // Equivalent plan: reorder the canonicalizable lists (docset preconditions
    // and target files sort by declared identity; action order is semantic and
    // preserved). The ID must be identical.
    let mut equiv_inputs = base.inputs.clone();
    equiv_inputs.client = base.inputs.client.clone();
    let equiv = plan::new_plan(
        equiv_inputs,
        PlanPreconditions {
            cache_layout: base.preconditions.cache_layout.clone(),
            model_present: base.preconditions.model_present,
            // Reversed order: canonicalization must sort back to the same.
            docset_state: base
                .preconditions
                .docset_state
                .iter()
                .rev()
                .cloned()
                .collect(),
            target_files: base
                .preconditions
                .target_files
                .iter()
                .rev()
                .cloned()
                .collect(),
        },
        // Same actions in the same order (action order is semantic).
        base.actions.clone(),
        1_700_000_000,
    )
    .expect("equivalent plan must construct");
    let id_equiv = plan::plan_id(&equiv).expect("plan_id equiv");
    assert_eq!(
        id_base, id_equiv,
        "equivalent plans (reordered canonical lists) must share an id"
    );

    // Changing action order changes the ID (order is semantic execution order).
    let mut reordered = base.clone();
    reordered.actions.swap(0, 1);
    let id_reordered = plan::plan_id(&reordered).expect("plan_id reordered");
    assert_ne!(
        id_base, id_reordered,
        "changing action order must change the plan id"
    );

    // Changing a target file digest changes the ID.
    let mut changed_target = base.clone();
    changed_target.preconditions.target_files[0].sha256 = Some("a".repeat(64));
    let id_changed_target = plan::plan_id(&changed_target).expect("plan_id changed target");
    assert_ne!(
        id_base, id_changed_target,
        "changing a target digest must change the plan id"
    );

    // Changing the expiry changes the ID.
    let later_created = sample_plan(1_700_000_000 + PLAN_TTL_SECS);
    let id_later = plan::plan_id(&later_created).expect("plan_id later");
    assert_ne!(
        id_base, id_later,
        "changing the expiry (via created_at) must change the plan id"
    );
}

// --- Test 2: store/load round trip, no overwrite, 0600 on Unix ---

#[test]
fn plan_store_load_round_trip_and_no_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let created = 1_700_000_000_u64;
    let p = sample_plan(created);
    let id = plan::store_plan(&p).expect("store_plan");

    // Strict filename: <id>.json under the plans dir.
    let plans_dir = nowdocs::cache::automation_root().join("plans");
    let expected_path = plans_dir.join(format!("{id}.json"));
    assert!(
        expected_path.is_file(),
        "plan must be stored at {}",
        expected_path.display()
    );

    // File name stem equals the plan id exactly.
    let stem = expected_path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("file stem");
    assert_eq!(stem, &id, "file name stem must equal the plan id");

    // 0600 on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&expected_path)
            .expect("metadata")
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "plan file must be 0600 on Unix, got {:o}",
            mode
        );
    }

    // Load before expiry returns an equivalent plan.
    let loaded = plan::load_plan(&id, created + 60).expect("load_plan");
    assert_eq!(
        plan::plan_id(&loaded).expect("loaded plan_id"),
        id,
        "loaded plan id must match"
    );
    assert_eq!(loaded.inputs, p.inputs);
    assert_eq!(loaded.actions, p.actions);

    // Storing the same plan again is success (byte-identical content).
    let id_again = plan::store_plan(&p).expect("store_plan identical");
    assert_eq!(id, id_again, "identical re-store returns same id");

    // Storing a different plan that would hash to a different id is fine; but a
    // tampered same-id file must be rejected. Mutate on-disk bytes and re-store
    // the original plan: the stored bytes differ -> error (no overwrite).
    let mut tampered = std::fs::read(&expected_path).unwrap();
    // Flip a character in the summary without changing the file name.
    let mut json: serde_json::Value = serde_json::from_slice(&tampered).unwrap();
    json["summary"] = serde_json::Value::String("tampered".to_string());
    tampered = serde_json::to_vec(&json).unwrap();
    std::fs::write(&expected_path, &tampered).unwrap();
    let res = plan::store_plan(&p);
    assert!(
        res.is_err(),
        "store_plan must not overwrite a same-id file with non-byte-identical content"
    );
}

// --- Test 3: rejects tampering, expiry, and symlink ---

#[test]
fn plan_rejects_tampering_expiry_and_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let created = 1_700_000_000_u64;
    let p = sample_plan(created);
    let id = plan::store_plan(&p).expect("store_plan");
    let path = nowdocs::cache::automation_root()
        .join("plans")
        .join(format!("{id}.json"));

    // Mutate the stored JSON after store -> PLAN_TAMPERED on load.
    let mut raw = std::fs::read_to_string(&path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    json["summary"] = serde_json::Value::String("mutated".to_string());
    raw = serde_json::to_string(&json).unwrap();
    std::fs::write(&path, &raw).unwrap();
    let err = plan::load_plan(&id, created + 60).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.starts_with("PLAN_TAMPERED"),
        "tampered plan must yield PLAN_TAMPERED, got: {msg}"
    );

    // Restore the original bytes for the rest of the test. The tampered file
    // differs from the candidate, so store_plan would refuse to overwrite it;
    // remove the tampered file and re-store the original.
    std::fs::remove_file(&path).unwrap();
    plan::store_plan(&p).expect("restore original store");

    // Expired plan -> PLAN_EXPIRED.
    let err = plan::load_plan(&id, created + PLAN_TTL_SECS + 1).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.starts_with("PLAN_EXPIRED"),
        "expired plan must yield PLAN_EXPIRED, got: {msg}"
    );

    // Replace the plan path with a symlink to an external target and confirm
    // the loader does not follow it (O_NOFOLLOW refuses at open time on Unix).
    #[cfg(unix)]
    {
        std::fs::remove_file(&path).unwrap();
        let external = dir.path().join("external-target.json");
        std::fs::write(&external, b"not a real plan").unwrap();
        std::os::unix::fs::symlink(&external, &path).unwrap();
        let err = plan::load_plan(&id, created + 60).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.starts_with("PLAN_TAMPERED") || msg.starts_with("PLAN_NOT_FOUND"),
            "symlinked plan path must not be followed, got: {msg}"
        );
        // The external target must be unchanged (never written to).
        let after = std::fs::read_to_string(&external).unwrap();
        assert_eq!(
            after, "not a real plan",
            "external target must be untouched"
        );
    }
}

// --- Test 4: rejects destructive or ambiguous semantics at construction ---

#[test]
fn plan_rejects_destructive_or_ambiguous_semantics() {
    let created = 1_700_000_000_u64;
    let good_inputs = PlanInputs {
        client: None,
        docset: Some("nextjs".to_string()),
        online: false,
    };
    let good_pre = PlanPreconditions {
        cache_layout: "1".to_string(),
        model_present: true,
        docset_state: vec![DocsetPrecondition {
            docset: "nextjs".to_string(),
            installed: false,
            manifest_sha256: None,
        }],
        target_files: vec![],
    };
    let good_action = PlannedAction {
        id: "a1".to_string(),
        kind: "docset_install".to_string(),
        risk: RiskLevel::Additive,
        summary: "install".to_string(),
        changes_state: true,
        network_access: true,
        requires_confirmation: true,
        reversible: true,
        target_paths: vec![],
        estimated_download_bytes: None,
    };

    // Destructive risk is rejected.
    let mut destructive = good_action.clone();
    destructive.risk = RiskLevel::Destructive;
    let err = plan::new_plan(
        good_inputs.clone(),
        good_pre.clone(),
        vec![destructive],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("destructive") || msg.contains("Destructive"),
        "destructive risk must be rejected, got: {msg}"
    );

    // Duplicate action IDs are rejected.
    let dup_err = plan::new_plan(
        good_inputs.clone(),
        good_pre.clone(),
        vec![good_action.clone(), good_action.clone()],
        created,
    )
    .unwrap_err();
    let msg = format!("{dup_err}");
    assert!(
        msg.contains("duplicate"),
        "duplicate action id must be rejected, got: {msg}"
    );

    // Invalid docset in a precondition is rejected.
    let mut bad_docset_pre = good_pre.clone();
    bad_docset_pre.docset_state[0].docset = "../evil".to_string();
    let err = plan::new_plan(
        good_inputs.clone(),
        bad_docset_pre,
        vec![good_action.clone()],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("docset"),
        "invalid docset precondition must be rejected, got: {msg}"
    );

    // Empty action id is rejected.
    let mut empty_id = good_action.clone();
    empty_id.id = "".to_string();
    let err = plan::new_plan(
        good_inputs.clone(),
        good_pre.clone(),
        vec![empty_id],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("id"),
        "empty action id must be rejected, got: {msg}"
    );

    // Empty action kind is rejected.
    let mut empty_kind = good_action.clone();
    empty_kind.kind = "".to_string();
    let err = plan::new_plan(
        good_inputs.clone(),
        good_pre.clone(),
        vec![empty_kind],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("kind"),
        "empty action kind must be rejected, got: {msg}"
    );

    // Mismatch between risk and changes_state policy: an Additive action that
    // does not change state is ambiguous and rejected.
    let mut no_change = good_action.clone();
    no_change.changes_state = false;
    let err = plan::new_plan(
        good_inputs.clone(),
        good_pre.clone(),
        vec![no_change],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("state") || msg.contains("changes_state") || msg.contains("risk"),
        "risk/changes_state mismatch must be rejected, got: {msg}"
    );

    // Duplicate docset preconditions are rejected.
    let mut dup_docset_pre = good_pre.clone();
    dup_docset_pre
        .docset_state
        .push(good_pre.docset_state[0].clone());
    let err = plan::new_plan(
        good_inputs.clone(),
        dup_docset_pre,
        vec![good_action.clone()],
        created,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("duplicate"),
        "duplicate docset precondition must be rejected, got: {msg}"
    );

    // Duplicate target logical IDs are rejected.
    let mut dup_target_pre = good_pre.clone();
    dup_target_pre.target_files = vec![
        TargetFilePrecondition {
            logical_id: "same".to_string(),
            exists: false,
            sha256: None,
        },
        TargetFilePrecondition {
            logical_id: "same".to_string(),
            exists: false,
            sha256: None,
        },
    ];
    let err = plan::new_plan(good_inputs, dup_target_pre, vec![good_action], created).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("duplicate"),
        "duplicate target logical id must be rejected, got: {msg}"
    );
}

// --- C3-R1 Test: public construction bypass is rejected at every entry point ---

/// Build a hand-built public AutomationPlan that would bypass new_plan's
/// validation. Every lifecycle entry point must reject it.
fn forged_plan() -> AutomationPlan {
    AutomationPlan {
        schema_version: 999,                         // wrong schema
        nowdocs_version: "evil-version".to_string(), // wrong version
        created_at_unix_secs: 100,
        expires_at_unix_secs: 999, // wrong expiry
        inputs: PlanInputs {
            client: Some("../../../etc/passwd".to_string()), // unsafe
            docset: Some("../evil".to_string()),             // invalid docset
            online: true,
        },
        preconditions: PlanPreconditions {
            cache_layout: "1".to_string(),
            model_present: true,
            docset_state: vec![],
            target_files: vec![],
        },
        actions: vec![PlannedAction {
            id: "a1".to_string(),
            kind: "rm-rf".to_string(),
            risk: RiskLevel::Destructive, // destructive!
            summary: "destroy".to_string(),
            changes_state: false, // mismatch with Destructive (though Destructive is rejected first)
            network_access: false,
            requires_confirmation: false,
            reversible: false,
            target_paths: vec!["/absolute/path".to_string(), "../../etc".to_string()], // unsafe
            estimated_download_bytes: None,
        }],
    }
}

#[test]
fn plan_id_rejects_forged_public_construction() {
    let forged = forged_plan();
    let err = plan::plan_id(&forged).unwrap_err();
    let msg = format!("{err}");
    assert!(
        !msg.is_empty(),
        "plan_id must normalize and reject a forged plan, got: {msg}"
    );
}

#[test]
fn store_plan_rejects_forged_public_construction() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let forged = forged_plan();
    let err = plan::store_plan(&forged).unwrap_err();
    let msg = format!("{err}");
    assert!(
        !msg.contains("already exists"),
        "store_plan must reject a forged plan at normalization, not reach file creation: {msg}"
    );
}

#[test]
fn load_plan_rejects_forged_stored_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Craft a file whose bytes serialize a forged plan but whose filename
    // matches the forged plan's (invalid) hash. To do this we need to bypass
    // store_plan's normalization. We write the forged plan's JSON directly
    // to disk, then compute the hash of the forged plan's bytes to name the
    // file. However, plan_id also normalizes, so it will reject the forged
    // plan. Instead, we write arbitrary valid-looking JSON to a file named
    // with a hash we compute manually, simulating a tampered stored plan.

    // Write a forged plan's JSON directly to disk under a fake id.
    let forged = forged_plan();
    let raw = serde_json::to_vec(&forged).unwrap();

    // Use a dummy 64-hex id (all zeros). The load must reject it because the
    // plan fails normalization (PLAN_TAMPERED), regardless of hash mismatch.
    let fake_id = "0".repeat(64);
    plan::ensure_automation_root().unwrap();
    let path = nowdocs::cache::automation_root()
        .join("plans")
        .join(format!("{fake_id}.json"));
    std::fs::write(&path, &raw).unwrap();

    let err = plan::load_plan(&fake_id, 200).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.starts_with("PLAN_TAMPERED"),
        "load_plan must reject a forged stored plan as PLAN_TAMPERED, got: {msg}"
    );
}

// --- C3-R1 Test: specific forged variants are each rejected ---

#[test]
fn store_plan_rejects_wrong_schema_version() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.schema_version = 999;
    assert!(
        plan::store_plan(&p).is_err(),
        "wrong schema must be rejected"
    );
}

#[test]
fn store_plan_rejects_wrong_nowdocs_version() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.nowdocs_version = "0.0.0-evil".to_string();
    assert!(
        plan::store_plan(&p).is_err(),
        "wrong nowdocs_version must be rejected"
    );
}

#[test]
fn store_plan_rejects_destructive_via_public_fields() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.actions[0].risk = RiskLevel::Destructive;
    assert!(
        plan::store_plan(&p).is_err(),
        "destructive risk via public fields must be rejected"
    );
}

#[test]
fn store_plan_rejects_unsafe_target_path() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.actions[0].target_paths = vec!["../../../etc/passwd".to_string()];
    assert!(
        plan::store_plan(&p).is_err(),
        "unsafe target path via public fields must be rejected"
    );
}

#[test]
fn store_plan_rejects_absolute_target_path() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.actions[0].target_paths = vec!["/etc/passwd".to_string()];
    assert!(
        plan::store_plan(&p).is_err(),
        "absolute target path must be rejected"
    );
}

#[test]
fn store_plan_rejects_invalid_optional_docset() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.inputs.docset = Some("../evil".to_string());
    assert!(
        plan::store_plan(&p).is_err(),
        "invalid optional docset must be rejected"
    );
}

#[test]
fn store_plan_rejects_wrong_expiry() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    let mut p = sample_plan(1_700_000_000);
    p.expires_at_unix_secs = p.created_at_unix_secs + 1; // not TTL
    assert!(
        plan::store_plan(&p).is_err(),
        "wrong expiry must be rejected"
    );
}

// --- C3-R1 Test: symlinked automation root/plans component is refused ---

#[test]
#[cfg(unix)]
fn plan_storage_refuses_symlinked_automation_root() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Create the cache root, then replace automation/ with a symlink to an
    // external directory.
    let cache_root = nowdocs::cache::cache_root();
    std::fs::create_dir_all(&cache_root).unwrap();
    let external = dir.path().join("external-automation");
    std::fs::create_dir_all(&external).unwrap();
    let auto_root = cache_root.join("automation");
    std::os::unix::fs::symlink(&external, &auto_root).unwrap();

    // Plan storage must refuse to create plans/ through a symlinked root.
    let p = sample_plan(1_700_000_000);
    let err = plan::store_plan(&p).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("not a directory") || msg.contains("symlink"),
        "symlinked automation root must be refused, got: {msg}"
    );

    // The external target must not have plans/ created inside it.
    assert!(
        !external.join("plans").exists(),
        "external symlink target must not have plans/ created inside it"
    );
}

#[test]
#[cfg(unix)]
fn plan_storage_refuses_symlinked_plans_dir() {
    let dir = tempfile::tempdir().unwrap();
    let _g = EnvGuard::set("XDG_CACHE_HOME", dir.path().to_str().unwrap());

    // Create automation/ as a real dir, but plans/ as a symlink.
    let cache_root = nowdocs::cache::cache_root();
    let auto_root = cache_root.join("automation");
    std::fs::create_dir_all(&auto_root).unwrap();
    let external = dir.path().join("external-plans");
    std::fs::create_dir_all(&external).unwrap();
    let plans_dir = auto_root.join("plans");
    std::os::unix::fs::symlink(&external, &plans_dir).unwrap();

    // Plan storage must refuse to write through a symlinked plans/ dir.
    let p = sample_plan(1_700_000_000);
    let err = plan::store_plan(&p).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("not a directory") || msg.contains("symlink"),
        "symlinked plans dir must be refused, got: {msg}"
    );

    // The external target must not have any plan file written inside it.
    assert!(
        std::fs::read_dir(&external).unwrap().count() == 0,
        "external symlink target must not have any files written inside it"
    );
}

// --- C3-R1 Test: normalization round-trip preserves valid plan semantics ---

#[test]
fn normalize_preserves_valid_plan_and_canonicalizes_lists() {
    let created = 1_700_000_000_u64;
    let p = sample_plan(created);
    let id = plan::plan_id(&p).expect("plan_id on valid plan");

    // A plan with reversed docset/target preconditions normalizes to the same id.
    let mut reversed = p.clone();
    reversed.preconditions.docset_state.reverse();
    reversed.preconditions.target_files.reverse();
    let id_reversed = plan::plan_id(&reversed).expect("plan_id reversed");
    assert_eq!(
        id, id_reversed,
        "reordered canonical lists must normalize to the same id"
    );
}
