use std::fs;

fn cargo_toml() -> String {
    fs::read_to_string("Cargo.toml").expect("read Cargo.toml")
}

#[test]
fn release_metadata_matches_tar_gz_archive_layout() {
    let manifest = cargo_toml();
    assert!(manifest.contains("version = \"0.1.2\""));
    assert!(manifest.contains("rust-version = \"1.97\""));
    assert!(manifest.contains(
        "pkg-url = \"{ repo }/releases/download/v{ version }/{ name }-v{ version }-{ target }.tar.gz\""
    ));
    assert!(
        manifest.contains("bin-dir = \"{ name }-v{ version }-{ target }/{ bin }{ binary-ext }\"")
    );
    assert!(manifest.contains("pkg-fmt = \"tgz\""));
}

#[test]
fn registry_builder_is_not_a_default_install_binary() {
    let manifest = cargo_toml();
    assert!(manifest.contains("required-features = [\"registry-builder\"]"));
}

#[test]
fn package_allowlist_excludes_internal_release_inputs() {
    let manifest = cargo_toml();
    assert!(manifest.contains("include = ["));
    assert!(!manifest.contains("include = [\"**/*\"]"));
}
