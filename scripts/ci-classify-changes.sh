#!/usr/bin/env bash
# Emit GitHub Actions outputs describing the risk tier of changed paths.
set -euo pipefail

docs_only=true
run_cross=false

while IFS= read -r path; do
    [ -z "$path" ] && continue

    case "$path" in
        README.md|CHANGELOG.md|CONTRIBUTING.md|LICENSE*|docs/**|.github/ISSUE_TEMPLATE/**|.github/PULL_REQUEST_TEMPLATE.md)
            ;;
        *)
            docs_only=false
            ;;
    esac

    case "$path" in
        Cargo.toml|Cargo.lock|.cargo/**|build.rs|proto/**|deny.toml|.pre-commit-config.yaml|.github/workflows/**|src/automation/**|src/clients/**|src/cache.rs|src/registry.rs|src/mcp.rs|src/sanitize.rs|src/verify.rs|tests/automation_*|tests/client_*|tests/sanitize_tests.rs|tests/setup_tests.rs|tests/verify_tests.rs|tests/agent_setup_e2e_tests.rs|tests/test_isolation_tests.rs|tests/ci-classify-changes-tests.sh|scripts/audit_license/**|scripts/ci-classify-changes.sh)
            run_cross=true
            ;;
    esac
done

if [ "$docs_only" = true ]; then
    run_rust=false
else
    run_rust=true
fi

run_release="$run_cross"

printf 'docs_only=%s\nrun_rust=%s\nrun_release=%s\nrun_cross=%s\n' \
    "$docs_only" "$run_rust" "$run_release" "$run_cross"
