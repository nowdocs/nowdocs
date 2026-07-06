use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use nowdocs::cli::{Cli, Commands};

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
        Commands::Install { docset } => {
            let url = registry_url_for(&docset);
            nowdocs::registry::install(&docset, &url)?;
            println!("installed {docset}");
            println!("next: nowdocs smoke {docset}");
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
        } => {
            let meta = nowdocs::ingest::IngestMeta {
                license: license.unwrap_or_else(|| "MIT".to_string()),
                copyright_holder: copyright_holder.unwrap_or_default(),
                attribution: attribution.unwrap_or_default(),
                source_url: source_url.unwrap_or_default(),
                entry_url: entry_url.unwrap_or_default(),
            };
            let stats = nowdocs::ingest::ingest_dir(Path::new(&dir), &name, &meta)?;
            println!("ingested {} files, {} chunks", stats.files, stats.chunks);
            println!("next: nowdocs smoke {name}");
            Ok(())
        }
        Commands::Share { docset, out_dir } => {
            let out_dir = match out_dir {
                Some(p) => PathBuf::from(p),
                None => std::env::current_dir()?.join(format!("{docset}-share")),
            };
            let product = nowdocs::registry::share(&docset, &out_dir)?;
            println!("wrote {}", product.display());
            println!("next: submit to nowdocs-registry via PR");
            Ok(())
        }
        Commands::Uninstall { docset } => {
            nowdocs::registry::uninstall(&docset)?;
            println!("uninstalled {docset}");
            Ok(())
        }
        Commands::ListInstalled => {
            let docsets = list_installed()?;
            if docsets.is_empty() {
                println!("no docsets installed");
            } else {
                println!("{}", docsets.join(", "));
            }
            Ok(())
        }
        Commands::Update { docset } => {
            nowdocs::registry::update(&docset)?;
            println!("updated {docset}");
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
                nowdocs::doctor::run_model_check()
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
    }
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

fn list_installed() -> std::io::Result<Vec<String>> {
    let db_dir = nowdocs::cache::cache_root().join("db");
    let mut docsets: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&db_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(stem) = name.strip_suffix(".lance") {
                        docsets.push(stem.to_string());
                    }
                }
            }
        }
    }
    docsets.sort();
    Ok(docsets)
}

fn registry_url_for(docset: &str) -> String {
    // Test-only override: when NOWDOCS_TEST_URL is set (typically to a
    // file:// path), use it directly so install/update can be exercised
    // without a real network round-trip. The registry's own
    // `update()` reads this same env var.
    if let Ok(url) = std::env::var("NOWDOCS_TEST_URL") {
        if !url.is_empty() {
            return url;
        }
    }
    format!("https://github.com/nowdocs-registry/{docset}/releases/latest/download/{docset}.tar")
}
