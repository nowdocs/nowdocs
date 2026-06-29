use std::process::Command;

#[test]
fn test_cli_help_lists_all_subcommands() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for sub in ["serve", "install", "ingest", "share", "uninstall", "list-installed", "update"] {
        assert!(stdout.contains(sub), "help must list `{}`", sub);
    }
    // serve must NOT take --host/--port (network-defense rule)
    assert!(!stdout.contains("--port"), "serve must be argless (stdio binds no port)");
}
