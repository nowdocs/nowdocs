use std::path::Path;
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
            Ok(())
        }
        Commands::Ingest { dir, name } => {
            let stats = nowdocs::ingest::ingest_dir(Path::new(&dir), &name)?;
            println!("ingested {} files, {} chunks", stats.files, stats.chunks);
            Ok(())
        }
        Commands::Share { docset } => {
            let out_dir = std::env::current_dir()?.join(format!("{docset}-share"));
            let product = nowdocs::registry::share(&docset, &out_dir)?;
            println!("wrote {}", product.display());
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
            Ok(())
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
