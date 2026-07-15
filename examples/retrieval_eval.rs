//! Internal retrieval evaluator (C03): evaluates a versioned `EvalQuery`
//! fixture suite against installed corpora and writes the JSON v1 report.
//!
//! This is NOT a `nowdocs` subcommand — it is a standalone internal tool:
//!
//! ```text
//! cargo run --example retrieval_eval -- \
//!   --fixtures-dir tests/fixtures/eval \
//!   --split development \
//!   --output /absolute/path/report.json \
//!   --code-commit <40-char git SHA> \
//!   [--benchmark-runs <positive integer, default 1>]
//! ```
//!
//! `--help` and argument validation (including the 40-character hex
//! `--code-commit` check) complete before any fixture, model, or store work.
//! With `--benchmark-runs > 1` each query is warmed once unmeasured and then
//! repeated in-process; median/p95 retrieval latency goes to stderr only —
//! the JSON report contract stays exact.

use anyhow::{Context, Result};
use clap::Parser;
use nowdocs::eval::{
    run_evaluation, validate_evidence_output_path, EvalRunConfig, RetrievalEvalArgs,
};

fn main() -> Result<()> {
    let args = RetrievalEvalArgs::parse();
    anyhow::ensure!(
        args.output.is_absolute(),
        "--output must be an absolute path, got {}",
        args.output.display()
    );
    validate_evidence_output_path(&args.output, args.evidence_output.as_deref())
        .map_err(anyhow::Error::msg)?;

    // Production search defaults, matching the behavior under evaluation.
    let config = EvalRunConfig {
        fixtures_dir: args.fixtures_dir.clone(),
        split: args.split.clone(),
        code_commit: args.code_commit.clone(),
        benchmark_runs: args.benchmark_runs,
        max_tokens: 4000,
        top_k: 5,
    };

    let outcome = run_evaluation(&config)?;

    if let Some(latency) = &outcome.latency {
        eprintln!(
            "retrieval-latency: queries={} runs_per_query={} median_ms={:.3} p95_ms={:.3}",
            latency.queries, latency.runs_per_query, latency.median_ms, latency.p95_ms
        );
    }

    let json = serde_json::to_string_pretty(&outcome.report).context("serialize eval report")?;
    write_report_atomic(&args.output, &json)?;
    eprintln!(
        "retrieval-eval: wrote {} ({} queries)",
        args.output.display(),
        outcome.report.queries.len()
    );

    // C07a: write the optional decision-evidence sidecar.
    if let Some(ref path) = args.evidence_output {
        let evidence_json =
            serde_json::to_string_pretty(&outcome.evidence_rows).context("serialize evidence")?;
        write_report_atomic(path, &evidence_json)?;
        eprintln!(
            "retrieval-eval: wrote evidence sidecar {} ({} rows)",
            path.display(),
            outcome.evidence_rows.len()
        );
    }

    Ok(())
}

/// Write `contents` to `path` atomically enough that an existing successful
/// report is never replaced by partial output: write a temporary sibling in
/// the same directory, then rename over the target. The temporary file is
/// removed if the rename fails.
fn write_report_atomic(path: &std::path::Path, contents: &str) -> Result<()> {
    let file_name = path.file_name().context("--output must name a file")?;
    let tmp = path.with_file_name(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&tmp, contents)
        .with_context(|| format!("write temporary report {}", tmp.display()))?;
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = std::fs::remove_file(&tmp);
            Err(err).with_context(|| format!("rename report into {}", path.display()))
        }
    }
}
