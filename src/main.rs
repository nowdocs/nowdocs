use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use nowdocs::cli::{CacheCommands, Cli, Commands, RegistryCommands, SetupCommands};

fn main() -> ExitCode {
    let args = Cli::parse();
    let cmd_name = command_name(&args.command);
    match run(args.command) {
        Ok(()) => {
            // After a successful eligible command, check for a binary update
            // and print any reminder to stderr. Failures are silent and never
            // change the exit code.
            if nowdocs::update::test_util::is_eligible_command(cmd_name) {
                if let Ok(Some(reminder)) =
                    nowdocs::update::UpdateService::new(env!("CARGO_PKG_VERSION"))
                        .and_then(|svc| svc.check_and_notify())
                {
                    eprintln!("{reminder}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Map a `Commands` variant to its CLI subcommand name string (e.g. "install",
/// "serve"). Used for update-check eligibility.
fn command_name(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Serve => "serve",
        Commands::Capabilities { .. } => "capabilities",
        Commands::Status { .. } => "status",
        Commands::Install { .. } => "install",
        Commands::Ingest { .. } => "ingest",
        Commands::Share { .. } => "share",
        Commands::Uninstall { .. } => "uninstall",
        Commands::ListInstalled => "list-installed",
        Commands::Update { .. } => "update",
        Commands::Ensure { .. } => "ensure",
        Commands::Setup { .. } => "setup",
        Commands::Registry { .. } => "registry",
        Commands::Rebuild { .. } => "rebuild",
        Commands::Smoke { .. } => "smoke",
        Commands::Doctor { .. } => "doctor",
        Commands::Cache { .. } => "cache",
    }
}

fn run(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Serve => nowdocs::mcp::run_loop().map_err(anyhow::Error::from),
        Commands::Capabilities { json } => {
            let data = nowdocs::agent_contract::capabilities_data();
            if json {
                let envelope = nowdocs::agent_contract::AgentEnvelope::new(
                    "capabilities",
                    nowdocs::agent_contract::AgentStatus::Ok,
                    nowdocs::agent_contract::ResultCode::Ready,
                    "nowdocs agent automation capabilities",
                    data,
                );
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                print!(
                    "{}",
                    nowdocs::agent_contract::format_capabilities_human(&data)
                );
            }
            Ok(())
        }
        Commands::Status { json } => {
            // Strictly read-only and offline; degraded observations are
            // reported as status=warning in the envelope, never as a process
            // error, so status always exits 0.
            let data = nowdocs::inspect::collect_status();
            if json {
                let envelope = nowdocs::inspect::status_envelope(&data);
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                print!("{}", nowdocs::inspect::format_status_human(&data));
            }
            Ok(())
        }
        Commands::Install { docset } => {
            let package = catalog_lookup_for(&docset)?;
            nowdocs::registry::install_verified_package(&package)?;
            print_install_success(&docset);
            Ok(())
        }
        Commands::Ingest {
            dir,
            name,
            license,
            copyright_holder,
            attribution,
            source_url,
            entry_url,
            source_url_base,
        } => {
            let meta = nowdocs::ingest::IngestMeta {
                license: license.unwrap_or_else(|| "MIT".to_string()),
                copyright_holder: copyright_holder.unwrap_or_default(),
                attribution: attribution.unwrap_or_default(),
                source_url: source_url.unwrap_or_default(),
                entry_url: entry_url.unwrap_or_default(),
                source_url_base,
            };
            let stats = nowdocs::ingest::ingest_dir(Path::new(&dir), &name, &meta)?;
            print_ingest_success(&name, stats.files, stats.chunks);
            Ok(())
        }
        Commands::Share { docset, out_dir } => {
            let out_dir = match out_dir {
                Some(p) => PathBuf::from(p),
                None => std::env::current_dir()?.join(format!("{docset}-share")),
            };
            let product = nowdocs::registry::share(&docset, &out_dir)?;
            println!("wrote {}", product.display());
            println!("vectors excluded (text + manifest only)");
            println!("next: submit PR to https://github.com/nowdocs-registry");
            Ok(())
        }
        Commands::Uninstall { docset } => {
            nowdocs::registry::uninstall(&docset)?;
            println!("uninstalled {docset}");
            Ok(())
        }
        Commands::ListInstalled => {
            let docsets = nowdocs::cache::list_installed()?;
            if docsets.is_empty() {
                println!("no docsets installed");
            } else {
                println!(
                    "{:<24} {:<10} {:<8} {:<12} STATUS",
                    "DOCSET", "VERSION", "CHUNKS", "LICENSE"
                );
                for d in &docsets {
                    println!(
                        "{:<24} {:<10} {:<8} {:<12} {}",
                        d.name, d.version, d.chunks, d.license, d.status
                    );
                }
            }
            Ok(())
        }
        Commands::Update { docset } => {
            // (S3) In production, the CLI `update` command fetches the catalog package index
            // (which only allows certified/allowed domains) to resolve both the download URL and
            // expected SHA-256 hash. The library's internal update() handler (which handles local test file://
            // redirection via NOWDOCS_TEST_URL in test mode) is bypassed here to enforce domain rules and
            // integrity symmetry.
            let package = catalog_lookup_for(&docset)?;
            nowdocs::registry::install_verified_package(&package)?;
            print_update_success(&docset);
            Ok(())
        }
        Commands::Ensure {
            docset,
            online,
            apply,
            json,
        } => {
            let now_unix_secs = now_unix_secs()?;
            if let Some(plan_hash) = apply {
                match nowdocs::automation::docset::ensure_apply(&docset, &plan_hash, now_unix_secs)
                {
                    Ok(nowdocs::automation::docset::EnsureApplyResult::AlreadySatisfied) => {
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::Ok,
                                nowdocs::agent_contract::ResultCode::AlreadySatisfied,
                                &format!("{docset} is already satisfied"),
                                serde_json::json!({"docset": docset}),
                                Vec::new(),
                            );
                        } else {
                            println!("{docset} is already satisfied");
                        }
                        Ok(())
                    }
                    Ok(nowdocs::automation::docset::EnsureApplyResult::Applied) => {
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::Ok,
                                nowdocs::agent_contract::ResultCode::SetupComplete,
                                &format!("{docset} ensured"),
                                serde_json::json!({"docset": docset}),
                                Vec::new(),
                            );
                        } else {
                            println!("ensured {docset}");
                        }
                        Ok(())
                    }
                    Err(e) => {
                        let (status, code, summary) = ensure_error_mapping(&e);
                        if json {
                            print_ensure_json(
                                &docset,
                                status,
                                code,
                                &summary,
                                serde_json::json!({"docset": docset, "error": format!("{e:#}")}),
                                Vec::new(),
                            );
                            // Plan/concurrency conflicts exit 10 per the agent contract.
                            if matches!(
                                code,
                                nowdocs::agent_contract::ResultCode::PlanNotFound
                                    | nowdocs::agent_contract::ResultCode::PlanExpired
                                    | nowdocs::agent_contract::ResultCode::PlanStale
                                    | nowdocs::agent_contract::ResultCode::PlanTampered
                                    | nowdocs::agent_contract::ResultCode::OperationInProgress
                            ) {
                                std::process::exit(code.exit_code().into());
                            }
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            } else {
                match nowdocs::automation::docset::ensure_plan(&docset, online, now_unix_secs) {
                    Ok(nowdocs::automation::docset::EnsurePlanResult::AlreadySatisfied {
                        ..
                    }) => {
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::Ok,
                                nowdocs::agent_contract::ResultCode::AlreadySatisfied,
                                &format!("{docset} is already satisfied"),
                                serde_json::json!({"docset": docset}),
                                Vec::new(),
                            );
                        } else {
                            println!("{docset} is already satisfied");
                        }
                        Ok(())
                    }
                    Ok(nowdocs::automation::docset::EnsurePlanResult::PlanCreated {
                        plan_id,
                        ..
                    }) => {
                        let next_actions = vec![nowdocs::agent_contract::NextAction {
                            id: "ensure-apply".to_string(),
                            kind: "ensure_apply".to_string(),
                            risk: nowdocs::agent_contract::RiskLevel::Additive,
                            summary: format!("Apply the ensure plan for {docset}"),
                            changes_state: true,
                            network_access: false,
                            requires_confirmation: true,
                            reversible: true,
                            argv: Some(vec![
                                "ensure".to_string(),
                                docset.clone(),
                                "--apply".to_string(),
                                plan_id.clone(),
                            ]),
                            target_paths: vec![],
                            estimated_download_bytes: None,
                        }];
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::ActionRequired,
                                nowdocs::agent_contract::ResultCode::ActionRequired,
                                &format!("run `nowdocs ensure {docset} --apply {plan_id}`"),
                                serde_json::json!({"docset": docset, "plan_hash": plan_id}),
                                next_actions,
                            );
                        } else {
                            println!("plan created: {plan_id}");
                            println!("run: nowdocs ensure {docset} --apply {plan_id}");
                        }
                        Ok(())
                    }
                    Ok(
                        nowdocs::automation::docset::EnsurePlanResult::RegistryMetadataRequired {
                            ..
                        },
                    ) => {
                        let next_actions = vec![nowdocs::agent_contract::NextAction {
                            id: "ensure-online".to_string(),
                            kind: "ensure_plan".to_string(),
                            risk: nowdocs::agent_contract::RiskLevel::ReadOnly,
                            summary: format!("Fetch registry metadata for {docset}"),
                            changes_state: false,
                            network_access: true,
                            requires_confirmation: false,
                            reversible: true,
                            argv: Some(vec![
                                "ensure".to_string(),
                                docset.clone(),
                                "--online".to_string(),
                            ]),
                            target_paths: vec![],
                            estimated_download_bytes: None,
                        }];
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::ActionRequired,
                                nowdocs::agent_contract::ResultCode::RegistryMetadataRequired,
                                &format!("registry metadata required for {docset}"),
                                serde_json::json!({"docset": docset}),
                                next_actions,
                            );
                        } else {
                            println!("registry metadata required for {docset}");
                            println!("run: nowdocs ensure {docset} --online");
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if json {
                            print_ensure_json(
                                &docset,
                                nowdocs::agent_contract::AgentStatus::Error,
                                nowdocs::agent_contract::ResultCode::InternalError,
                                &format!("{e:#}"),
                                serde_json::json!({"docset": docset, "error": format!("{e:#}")}),
                                Vec::new(),
                            );
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            }
        }
        Commands::Setup { command } => run_setup(command),
        Commands::Rebuild { docset } => {
            let stats = nowdocs::ingest::rebuild_docset(&docset)?;
            println!("rebuilt {docset}: {} chunks", stats.chunks);
            println!("next: nowdocs smoke {docset}");
            Ok(())
        }
        Commands::Smoke {
            docset,
            query,
            json,
            top_k,
        } => match nowdocs::smoke::smoke(&docset, query.as_deref(), top_k) {
            Ok(result) => {
                if result.result_count == 0 {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "docset": docset,
                                "error": "no results",
                                "hint": format!("nowdocs doctor --docset {docset}")
                            })
                        );
                    } else {
                        eprintln!("smoke: no results for {docset} — try `nowdocs doctor --docset {docset}`");
                    }
                    std::process::exit(1);
                }
                if json {
                    println!("{}", nowdocs::smoke::format_json(&result)?);
                } else {
                    print!("{}", nowdocs::smoke::format_human(&result));
                }
                Ok(())
            }
            Err(e) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "docset": docset,
                            "error": format!("{e:#}"),
                            "hint": "nowdocs doctor --model"
                        })
                    );
                } else {
                    eprintln!("error: {e:#}");
                }
                std::process::exit(1);
            }
        },
        Commands::Doctor {
            json,
            docset,
            mcp,
            model,
            repair,
        } => {
            let output = if repair {
                nowdocs::doctor::run_repair()
            } else if let Some(docset_name) = docset {
                nowdocs::doctor::run_docset_checks(&docset_name)
            } else if mcp {
                nowdocs::doctor::run_mcp_check()
            } else if model {
                // N5: `--model` is the pre-warm path — it actually downloads
                // the embedder, so install output can point users at it.
                nowdocs::doctor::run_model_prewarm()
            } else {
                nowdocs::doctor::run_default_checks()
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_doctor_output(&output);
            }

            // Exit with non-zero if any check failed
            if output.status == nowdocs::doctor::Severity::Fail {
                std::process::exit(1);
            }

            Ok(())
        }
        Commands::Registry { command } => match command {
            RegistryCommands::List { json } => {
                nowdocs::registry::list_index(json)?;
                Ok(())
            }
            RegistryCommands::Search { query, json } => {
                nowdocs::registry::search_index(&query, json)?;
                Ok(())
            }
        },
        Commands::Cache { command } => match command {
            CacheCommands::Status { json } => {
                let status = nowdocs::cache::cache_status()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    print_cache_status(&status);
                }
                Ok(())
            }
            CacheCommands::CleanStaging { older_than } => {
                let threshold = parse_duration(&older_than)?;
                let result = nowdocs::cache::clean_staging_older_than(threshold)?;
                println!(
                    "removed {} staging director{}",
                    result.removed.len(),
                    if result.removed.len() == 1 {
                        "y"
                    } else {
                        "ies"
                    }
                );
                for path in result.removed {
                    println!("removed {}", path.display());
                }
                if !result.skipped.is_empty() {
                    println!(
                        "skipped {} staging director{}",
                        result.skipped.len(),
                        if result.skipped.len() == 1 {
                            "y"
                        } else {
                            "ies"
                        }
                    );
                }
                Ok(())
            }
        },
    }
}

fn print_cache_status(status: &nowdocs::cache::CacheStatus) {
    println!("cache root: {}", status.cache_root);
    println!("installed docsets: {}", status.installed_docsets);
    println!("staging dirs: {}", status.staging_count);
    println!("sizes:");
    println!("  total: {} bytes", status.total_bytes);
    println!("  db: {} bytes", status.db_bytes);
    println!("  manifests: {} bytes", status.manifests_bytes);
    println!("  models: {} bytes", status.models_bytes);
    println!("  staging: {} bytes", status.staging_bytes);
    println!("  rollback: {} bytes", status.rollback_bytes);
}

fn parse_duration(input: &str) -> anyhow::Result<std::time::Duration> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("duration must not be empty; use examples like 30m, 2h, 1d, or 3600s");
    }
    let (number, unit) = trimmed.split_at(
        trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(trimmed.len()),
    );
    let value: u64 = number.parse().map_err(|_| {
        anyhow::anyhow!("invalid duration {input:?}; use examples like 30m, 2h, 1d, or 3600s")
    })?;
    let seconds = match unit {
        "" | "s" => value,
        "m" => value * 60,
        "h" => value * 60 * 60,
        "d" => value * 60 * 60 * 24,
        _ => anyhow::bail!("invalid duration unit {unit:?}; supported units are s, m, h, and d"),
    };
    Ok(std::time::Duration::from_secs(seconds))
}

fn print_doctor_output(output: &nowdocs::doctor::DoctorOutput) {
    use nowdocs::doctor::Severity;

    let status_str = match output.status {
        Severity::Ok => "ok",
        Severity::Warn => "warn",
        Severity::Fail => "fail",
    };

    println!("doctor status: {}", status_str);
    println!("---");

    for check in &output.checks {
        let severity_str = match check.severity {
            Severity::Ok => "  ok",
            Severity::Warn => "warn",
            Severity::Fail => "FAIL",
        };
        println!("[{}] {}: {}", severity_str, check.id, check.message);
        if let Some(remediation) = &check.remediation {
            println!("      hint: {}", remediation);
        }
    }
}

/// Print enriched success output after install.
fn print_install_success(docset: &str) {
    let (version, chunks, license) = nowdocs::cache::read_docset_meta(docset);
    println!("installed {docset} v{version} ({chunks} chunks, {license})");
    println!("next: nowdocs smoke {docset}");
    // N5: point users at the pre-warm path so the first search doesn't stall.
    println!("{}", nowdocs::doctor::MODEL_PREWARM_HINT);
}

/// Print enriched success output after update.
fn print_update_success(docset: &str) {
    let (version, chunks, license) = nowdocs::cache::read_docset_meta(docset);
    println!("updated {docset} v{version} ({chunks} chunks, {license})");
    println!("next: nowdocs smoke {docset}");
}

/// Print enriched success output after ingest.
fn print_ingest_success(docset: &str, files: u32, chunks: u32) {
    let (_, _, license) = nowdocs::cache::read_docset_meta(docset);
    println!("ingested {docset}: {files} files, {chunks} chunks ({license})");
    println!("next: nowdocs smoke {docset}");
}

/// Look up a docset's full package metadata from the registry catalog index (S2).
///
/// The catalog is the source of truth for integrity: `fetch_index` already
/// validates every package's allowlisted download URL, license, and 64-hex
/// sha256. We bind the install to that hash so a tampered or corrupt artifact
/// is rejected before any active cache path is touched. An index fetch failure
/// or an unknown docset is a hard error — registry installs never skip
/// integrity verification.
fn catalog_lookup_for(docset: &str) -> anyhow::Result<nowdocs::registry::RegistryPackage> {
    let idx = nowdocs::registry::fetch_index()
        .context("fetch registry index to verify artifact integrity")?;
    match idx.packages.into_iter().find(|p| p.docset == docset) {
        Some(p) => Ok(p),
        None => anyhow::bail!(
            "docset {docset} not found in the registry index; run `nowdocs registry list` to see available docsets"
        ),
    }
}

/// Current wall-clock time as whole seconds since the Unix epoch.
fn now_unix_secs() -> anyhow::Result<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .context("system time before Unix epoch")
}

/// Print one agent envelope for `ensure`.
fn print_ensure_json(
    _docset: &str,
    status: nowdocs::agent_contract::AgentStatus,
    code: nowdocs::agent_contract::ResultCode,
    summary: &str,
    data: serde_json::Value,
    next_actions: Vec<nowdocs::agent_contract::NextAction>,
) {
    let envelope = nowdocs::agent_contract::AgentEnvelope {
        schema_version: nowdocs::agent_contract::AGENT_CONTRACT_SCHEMA_VERSION,
        nowdocs_version: env!("CARGO_PKG_VERSION").to_string(),
        command: "ensure".to_string(),
        status,
        code,
        summary: summary.to_string(),
        data,
        next_actions,
        rollback: None,
    };
    // Unwrap is safe: the envelope is built from serializable types.
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

/// Map an `ensure_apply` error to an agent-contract status/code/summary.
fn ensure_error_mapping(
    e: &anyhow::Error,
) -> (
    nowdocs::agent_contract::AgentStatus,
    nowdocs::agent_contract::ResultCode,
    String,
) {
    let msg = format!("{e:#}");
    if msg.contains("PLAN_NOT_FOUND") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanNotFound,
            msg,
        )
    } else if msg.contains("PLAN_EXPIRED") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanExpired,
            msg,
        )
    } else if msg.contains("PLAN_STALE") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanStale,
            msg,
        )
    } else if msg.contains("PLAN_TAMPERED") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanTampered,
            msg,
        )
    } else if msg.contains("OPERATION_IN_PROGRESS") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::OperationInProgress,
            msg,
        )
    } else if msg.contains("OPERATION_LOCK_UNSAFE") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PermissionDenied,
            msg,
        )
    } else {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::InternalError,
            msg,
        )
    }
}

// ---- C7: setup orchestration ----

/// Resolve the approved client configuration root from the caller context.
///
/// For this slice, the approved root is the user's home directory. C7 never
/// silently uses a real home directory without disclosing it: the root is
/// reported as a redacted observation in the output. If the home directory
/// cannot be determined, setup returns `action_required` without a write.
fn resolve_approved_root() -> anyhow::Result<PathBuf> {
    let home =
        dirs::home_dir().context("cannot determine home directory for approved client root")?;
    if !home.is_absolute() {
        anyhow::bail!("home directory is not an absolute path");
    }
    Ok(home)
}

/// Dispatch a `setup` subcommand.
fn run_setup(command: SetupCommands) -> anyhow::Result<()> {
    match command {
        SetupCommands::Plan {
            client,
            docset,
            online,
            json,
        } => run_setup_plan(&client, &docset, online, json),
        SetupCommands::Apply { plan_hash, json } => run_setup_apply(&plan_hash, json),
        SetupCommands::Rollback { operation_id, json } => run_setup_rollback(&operation_id, json),
    }
}

/// Run `setup plan --client <client> --docset <docset> [--online] [--json]`.
fn run_setup_plan(client: &str, docset: &str, online: bool, json: bool) -> anyhow::Result<()> {
    let now = now_unix_secs()?;
    match nowdocs::automation::setup::setup_plan(docset, client, online, now) {
        Ok(nowdocs::automation::setup::SetupPlanResult::AlreadySatisfied { .. }) => {
            if json {
                print_setup_json(
                    "setup.plan",
                    nowdocs::agent_contract::AgentStatus::Ok,
                    nowdocs::agent_contract::ResultCode::AlreadySatisfied,
                    &format!("{docset} is already installed"),
                    serde_json::json!({"client": client, "docset": docset}),
                    Vec::new(),
                    None,
                );
            } else {
                println!("{docset} is already installed");
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupPlanResult::PlanCreated { plan_hash, .. }) => {
            let next_actions = vec![nowdocs::agent_contract::NextAction {
                id: "setup-apply".to_string(),
                kind: "setup_apply".to_string(),
                risk: nowdocs::agent_contract::RiskLevel::Additive,
                summary: format!("Apply the setup plan for {client} + {docset}"),
                changes_state: true,
                network_access: false,
                requires_confirmation: true,
                reversible: true,
                argv: Some(vec![
                    "setup".to_string(),
                    "apply".to_string(),
                    "--plan-hash".to_string(),
                    plan_hash.clone(),
                ]),
                target_paths: vec![],
                estimated_download_bytes: None,
            }];
            if json {
                print_setup_json(
                    "setup.plan",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::ActionRequired,
                    &format!("run `nowdocs setup apply --plan-hash {plan_hash}`"),
                    serde_json::json!({
                        "client": client,
                        "docset": docset,
                        "plan_hash": plan_hash
                    }),
                    next_actions,
                    None,
                );
            } else {
                println!("setup plan created: {plan_hash}");
                println!("run: nowdocs setup apply --plan-hash {plan_hash}");
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupPlanResult::RegistryMetadataRequired { .. }) => {
            let next_actions = vec![nowdocs::agent_contract::NextAction {
                id: "setup-plan-online".to_string(),
                kind: "setup_plan".to_string(),
                risk: nowdocs::agent_contract::RiskLevel::ReadOnly,
                summary: format!("Fetch registry metadata for {docset}"),
                changes_state: false,
                network_access: true,
                requires_confirmation: false,
                reversible: true,
                argv: Some(vec![
                    "setup".to_string(),
                    "plan".to_string(),
                    "--client".to_string(),
                    client.to_string(),
                    "--docset".to_string(),
                    docset.to_string(),
                    "--online".to_string(),
                ]),
                target_paths: vec![],
                estimated_download_bytes: None,
            }];
            if json {
                print_setup_json(
                    "setup.plan",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::RegistryMetadataRequired,
                    &format!("registry metadata required for {docset}"),
                    serde_json::json!({"client": client, "docset": docset}),
                    next_actions,
                    None,
                );
            } else {
                println!("registry metadata required for {docset}");
                println!("run: nowdocs setup plan --client {client} --docset {docset} --online");
            }
            Ok(())
        }
        Err(e) => {
            let (status, code, summary) = setup_error_mapping(&e);
            if json {
                print_setup_json(
                    "setup.plan",
                    status,
                    code,
                    &summary,
                    serde_json::json!({
                        "client": client,
                        "docset": docset,
                        "error": format!("{e:#}")
                    }),
                    Vec::new(),
                    None,
                );
                if is_plan_conflict_code(code) {
                    std::process::exit(code.exit_code().into());
                }
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Run `setup apply --plan-hash <hash> [--json]`.
fn run_setup_apply(plan_hash: &str, json: bool) -> anyhow::Result<()> {
    let root = match resolve_approved_root() {
        Ok(r) => r,
        Err(e) => {
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::ClientNotDetected,
                    "cannot determine approved client root; manual configuration required",
                    serde_json::json!({"plan_hash": plan_hash, "error": format!("{e:#}")}),
                    Vec::new(),
                    None,
                );
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };

    let now = now_unix_secs()?;
    match nowdocs::automation::setup::setup_apply(plan_hash, &root, now) {
        Ok(nowdocs::automation::setup::SetupApplyResult::SetupComplete {
            operation_id,
            observations,
        }) => {
            let rollback = make_rollback(&operation_id);
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::Ok,
                    nowdocs::agent_contract::ResultCode::SetupComplete,
                    "setup complete",
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "operation_id": operation_id,
                        "observations": observations,
                    }),
                    Vec::new(),
                    rollback,
                );
            } else {
                println!("setup complete (operation: {operation_id})");
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupApplyResult::ClientReloadRequired {
            operation_id,
            observations,
        }) => {
            let rollback = make_rollback(&operation_id);
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::Ok,
                    nowdocs::agent_contract::ResultCode::ClientReloadRequired,
                    "setup applied; client reload required",
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "operation_id": operation_id,
                        "observations": observations,
                    }),
                    Vec::new(),
                    rollback,
                );
            } else {
                println!(
                    "setup applied; reload the client to activate (operation: {operation_id})"
                );
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupApplyResult::ActionRequired { observations }) => {
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::ActionRequired,
                    "setup could not complete automatically; manual action required",
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "observations": observations,
                    }),
                    Vec::new(),
                    None,
                );
            } else {
                println!("manual action required:");
                for obs in &observations {
                    println!("  - {obs}");
                }
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupApplyResult::PartialNoRollback { observations }) => {
            // Docset succeeded but client apply could not start. No client
            // change committed, so no rollback metadata is retained.
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::Partial,
                    nowdocs::agent_contract::ResultCode::ActionRequired,
                    "docset installed but client configuration requires manual action",
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "observations": observations,
                    }),
                    Vec::new(),
                    None,
                );
            } else {
                println!("partial: docset installed but client requires manual configuration");
                for obs in &observations {
                    println!("  - {obs}");
                }
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupApplyResult::Partial {
            operation_id,
            observations,
        }) => {
            let rollback = make_rollback(&operation_id);
            if json {
                print_setup_json(
                    "setup.apply",
                    nowdocs::agent_contract::AgentStatus::Partial,
                    nowdocs::agent_contract::ResultCode::AppliedButUnverified,
                    "configuration applied but verification incomplete",
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "operation_id": operation_id,
                        "observations": observations,
                    }),
                    Vec::new(),
                    rollback,
                );
                // Exit 21 for applied-but-unverified.
                std::process::exit(
                    nowdocs::agent_contract::ResultCode::AppliedButUnverified
                        .exit_code()
                        .into(),
                );
            } else {
                println!("partial: configuration applied but verification incomplete");
                println!("rollback: nowdocs setup rollback --operation-id {operation_id}");
                std::process::exit(
                    nowdocs::agent_contract::ResultCode::AppliedButUnverified
                        .exit_code()
                        .into(),
                );
            }
        }
        Err(e) => {
            let (status, code, summary) = setup_error_mapping(&e);
            if json {
                print_setup_json(
                    "setup.apply",
                    status,
                    code,
                    &summary,
                    serde_json::json!({
                        "plan_hash": plan_hash,
                        "error": format!("{e:#}")
                    }),
                    Vec::new(),
                    None,
                );
                if is_plan_conflict_code(code) {
                    std::process::exit(code.exit_code().into());
                }
                if code == nowdocs::agent_contract::ResultCode::ConfigWriteUnsafe
                    || code == nowdocs::agent_contract::ResultCode::UnsupportedPlatform
                {
                    std::process::exit(code.exit_code().into());
                }
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Run `setup rollback --operation-id <id> [--json]`.
fn run_setup_rollback(operation_id: &str, json: bool) -> anyhow::Result<()> {
    let root = match resolve_approved_root() {
        Ok(r) => r,
        Err(e) => {
            if json {
                print_setup_json(
                    "setup.rollback",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::ClientNotDetected,
                    "cannot determine approved client root; manual rollback required",
                    serde_json::json!({
                        "operation_id": operation_id,
                        "error": format!("{e:#}")
                    }),
                    Vec::new(),
                    None,
                );
                return Ok(());
            } else {
                return Err(e);
            }
        }
    };

    match nowdocs::automation::setup::setup_rollback(operation_id, &root) {
        Ok(nowdocs::automation::setup::SetupRollbackResult::RolledBack { observations }) => {
            if json {
                print_setup_json(
                    "setup.rollback",
                    nowdocs::agent_contract::AgentStatus::Ok,
                    nowdocs::agent_contract::ResultCode::SetupComplete,
                    "rollback complete",
                    serde_json::json!({
                        "operation_id": operation_id,
                        "observations": observations,
                    }),
                    Vec::new(),
                    None,
                );
            } else {
                println!("rollback complete (operation: {operation_id})");
            }
            Ok(())
        }
        Ok(nowdocs::automation::setup::SetupRollbackResult::ManualRequired { observations }) => {
            if json {
                print_setup_json(
                    "setup.rollback",
                    nowdocs::agent_contract::AgentStatus::ActionRequired,
                    nowdocs::agent_contract::ResultCode::ActionRequired,
                    "automatic rollback not possible; manual action required",
                    serde_json::json!({
                        "operation_id": operation_id,
                        "observations": observations,
                    }),
                    Vec::new(),
                    None,
                );
            } else {
                println!("manual rollback required:");
                for obs in &observations {
                    println!("  - {obs}");
                }
            }
            Ok(())
        }
        Err(e) => {
            let (status, code, summary) = setup_error_mapping(&e);
            if json {
                print_setup_json(
                    "setup.rollback",
                    status,
                    code,
                    &summary,
                    serde_json::json!({
                        "operation_id": operation_id,
                        "error": format!("{e:#}")
                    }),
                    Vec::new(),
                    None,
                );
                if is_plan_conflict_code(code) {
                    std::process::exit(code.exit_code().into());
                }
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Build rollback metadata for a successful setup apply.
fn make_rollback(operation_id: &str) -> Option<nowdocs::agent_contract::RollbackMetadata> {
    Some(nowdocs::agent_contract::RollbackMetadata {
        operation_id: operation_id.to_string(),
        expires_at: nowdocs::automation::operation::retention_expiry(
            nowdocs::automation::operation::OperationState::AppliedVerified,
            Some(std::time::SystemTime::now()),
        )
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| format!("{}", d.as_secs()))
        })
        .unwrap_or_default(),
        argv: vec![
            "nowdocs".to_string(),
            "setup".to_string(),
            "rollback".to_string(),
            "--operation-id".to_string(),
            operation_id.to_string(),
        ],
    })
}

/// True when the result code is a plan/concurrency conflict (exit class 10).
fn is_plan_conflict_code(code: nowdocs::agent_contract::ResultCode) -> bool {
    matches!(
        code,
        nowdocs::agent_contract::ResultCode::PlanNotFound
            | nowdocs::agent_contract::ResultCode::PlanExpired
            | nowdocs::agent_contract::ResultCode::PlanStale
            | nowdocs::agent_contract::ResultCode::PlanTampered
            | nowdocs::agent_contract::ResultCode::OperationInProgress
    )
}

/// Print one agent envelope for a `setup` subcommand.
fn print_setup_json(
    command: &str,
    status: nowdocs::agent_contract::AgentStatus,
    code: nowdocs::agent_contract::ResultCode,
    summary: &str,
    data: serde_json::Value,
    next_actions: Vec<nowdocs::agent_contract::NextAction>,
    rollback: Option<nowdocs::agent_contract::RollbackMetadata>,
) {
    let envelope = nowdocs::agent_contract::AgentEnvelope {
        schema_version: nowdocs::agent_contract::AGENT_CONTRACT_SCHEMA_VERSION,
        nowdocs_version: env!("CARGO_PKG_VERSION").to_string(),
        command: command.to_string(),
        status,
        code,
        summary: summary.to_string(),
        data,
        next_actions,
        rollback,
    };
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

/// Map a `setup` error to an agent-contract status/code/summary.
fn setup_error_mapping(
    e: &anyhow::Error,
) -> (
    nowdocs::agent_contract::AgentStatus,
    nowdocs::agent_contract::ResultCode,
    String,
) {
    let msg = format!("{e:#}");
    if msg.contains("PLAN_NOT_FOUND") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanNotFound,
            msg,
        )
    } else if msg.contains("PLAN_EXPIRED") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanExpired,
            msg,
        )
    } else if msg.contains("PLAN_STALE") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanStale,
            msg,
        )
    } else if msg.contains("PLAN_TAMPERED") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PlanTampered,
            msg,
        )
    } else if msg.contains("OPERATION_IN_PROGRESS") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::OperationInProgress,
            msg,
        )
    } else if msg.contains("OPERATION_LOCK_UNSAFE") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::PermissionDenied,
            msg,
        )
    } else if msg.contains("VERIFICATION_FAILED") {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::VerificationFailed,
            msg,
        )
    } else {
        (
            nowdocs::agent_contract::AgentStatus::Error,
            nowdocs::agent_contract::ResultCode::InternalError,
            msg,
        )
    }
}
