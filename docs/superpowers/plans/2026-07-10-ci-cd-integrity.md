# CI/CD Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CI dependency resolution reproducible, remove known false-green behavior, and establish a truthful coverage baseline without weakening quality gates.

**Architecture:** PR quality gates continue to run fast, deterministic Rust checks with the committed lockfile. Real-model evaluation remains scheduled/manual and uses the existing cache-aware fixture script. Coverage is introduced as a reporting baseline first, so a future threshold is calibrated from a stable measurement rather than selected to pass current code.

**Tech Stack:** GitHub Actions, Cargo, cargo-llvm-cov, Bash, Rust.

## Global Constraints

- Do not modify branch protection, runner selection, or production credentials.
- Keep real-model tests off ordinary PR and main pushes.
- Every Cargo command that resolves dependencies must use `--locked`.
- Coverage is report-only until a baseline is measured and documented.

---

### Task 1: Lockfile enforcement and truthful local gates

**Files:**
- Modify: `.github/workflows/gates.yml`
- Modify: `scripts/check.sh`
- Modify: `scripts/ci-prepare-nextjs-fixture.sh`

- [x] Add `--locked` to every CI and local Cargo command that builds, lints, tests, runs, or produces a release binary.
- [x] Add a dedicated `cargo metadata --locked --no-deps --format-version 1` gate before Rust build jobs.
- [x] Verify the fixture script does not bypass the lockfile.
- [x] Run `bash -n scripts/check.sh scripts/ci-prepare-nextjs-fixture.sh` and `cargo metadata --locked --no-deps --format-version 1`.

### Task 2: Remove false-green maintenance behavior

**Files:**
- Modify: `.github/workflows/weekly-audit.yml`

- [x] Change the unused-dependency job so a `cargo udeps` finding fails the job instead of emitting a warning and exiting successfully.
- [x] Pin the cargo-udeps and nightly Rust versions to make weekly results reproducible.
- [x] Pass the documented advisory exceptions to cargo-audit and grant only the check-writing permission its action requires.
- [x] Verify workflow YAML structure and confirm no failure-masking shell construct remains.

### Task 3: Coverage baseline and scheduled evaluation alignment

**Files:**
- Modify: `.github/workflows/eval.yml`
- Modify: `CONTRIBUTING.md`

- [x] Add a report-only coverage job using `cargo-llvm-cov` at a fixed version and publish its summary to the workflow run.
- [x] Run coverage with the same serial, locked test command as the PR suite; no coverage threshold is introduced.
- [x] Make the scheduled evaluation build and test with `--locked` and keep it scheduled/manual only.
- [x] Verify workflow YAML and collect a local coverage baseline for lines, functions, and regions.

### Final verification

- [x] Run `cargo fmt --check`.
- [x] Run `cargo clippy --locked --all-targets -- -D warnings`.
- [x] Run `cargo test --locked -- --test-threads=1`.
- [x] Run `cargo build --locked --release`.
- [x] Run workflow/script static checks and `git diff --check`.
- [ ] Commit with DCO sign-off, push `fix/ci-cd-integrity`, and inspect the GitHub Actions run to completion.
