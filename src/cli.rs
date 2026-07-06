use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "nowdocs",
    version,
    about = "Local MCP doc server for LLM agents"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCP stdio server (no host/port — stdio binds no port)
    Serve,
    /// Install a pre-built doc crate from the registry
    Install { docset: String },
    /// Import a Markdown directory as a local docset
    Ingest {
        dir: String,
        name: String,
        /// SPDX-style license for the docset (MIT, Apache-2.0, CC-BY-4.0). Defaults to MIT.
        #[arg(long)]
        license: Option<String>,
        /// Copyright holder line for the manifest legal block.
        #[arg(long)]
        copyright_holder: Option<String>,
        /// Attribution text (required when --license is CC-BY-4.0).
        #[arg(long)]
        attribution: Option<String>,
        /// Upstream source repo URL (e.g. https://github.com/vercel/next.js).
        #[arg(long)]
        source_url: Option<String>,
        /// Entry/docs site URL for the source block.
        #[arg(long)]
        entry_url: Option<String>,
    },
    /// Package a local docset for registry contribution (text+manifest, NOT vectors)
    Share {
        docset: String,
        /// Output directory (defaults to ./{docset}-share in cwd).
        #[arg(long)]
        out_dir: Option<String>,
    },
    /// Remove an installed docset
    Uninstall { docset: String },
    /// List installed docsets
    ListInstalled,
    /// Update a docset to the latest registry version
    Update { docset: String },
    /// Run read-only diagnostics on nowdocs environment
    Doctor {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Deep-check a specific docset
        #[arg(long)]
        docset: Option<String>,
        /// Run MCP smoke test (in-process, no network)
        #[arg(long)]
        mcp: bool,
        /// Check model cache state
        #[arg(long)]
        model: bool,
        /// Repair mode (not implemented yet, staging cleanup only)
        #[arg(long)]
        repair: bool,
    },
}
