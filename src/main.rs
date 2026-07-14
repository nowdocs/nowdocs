use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use nowdocs::cli::{CacheCommands, Cli, Commands, RegistryCommands};

fn main() -> ExitCode {
    let args = Cli::parse();
    match run(args.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
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
        Commands::Install { docset } => {
            let (url, sha) = catalog_lookup_for(&docset)?;
            nowdocs::registry::install_with_sha256(&docset, &url, &sha)?;
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
            let (url, sha) = catalog_lookup_for(&docset)?;
            nowdocs::registry::install_with_sha256(&docset, &url, &sha)?;
            print_update_success(&docset);
            Ok(())
        }
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

/// Look up a docset's download URL and expected sha256 from the registry catalog index (S2).
///
/// The catalog is the source of truth for integrity: `fetch_index` already
/// validates every package's allowlisted download URL, license, and 64-hex
/// sha256. We bind the install to that hash so a tampered or corrupt artifact
/// is rejected before any active cache path is touched. An index fetch failure
/// or an unknown docset is a hard error — registry installs never skip
/// integrity verification.
fn catalog_lookup_for(docset: &str) -> anyhow::Result<(String, String)> {
    let idx = nowdocs::registry::fetch_index()
        .context("fetch registry index to verify artifact integrity")?;
    match idx.packages.iter().find(|p| p.docset == docset) {
        Some(p) => Ok((p.download_url.clone(), p.sha256.clone())),
        None => anyhow::bail!(
            "docset {docset} not found in the registry index; run `nowdocs registry list` to see available docsets"
        ),
    }
}
