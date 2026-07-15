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
    /// Report agent-automation capabilities (read-only, offline-safe)
    Capabilities {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Report a read-only, offline snapshot of local nowdocs state
    Status {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
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
        /// Base URL prepended to each chunk's relative path to form a traceable
        /// per-chunk source_url (e.g. https://github.com/vercel/next.js/tree/canary/docs).
        /// Canonical registry docsets should always set this. Omit for local private docs.
        #[arg(long)]
        source_url_base: Option<String>,
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
    /// Idempotently ensure a docset is installed from the registry
    Ensure {
        docset: String,
        /// Fetch and validate registry metadata in memory (required to plan a new install/update)
        #[arg(long)]
        online: bool,
        /// Apply a previously created ensure plan by hash
        #[arg(long)]
        apply: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Plan, apply, or roll back a one-plan setup for one client plus one docset
    Setup {
        #[command(subcommand)]
        command: SetupCommands,
    },
    /// Browse the nowdocs registry catalog (list / search available docsets; may access the network)
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Rebuild a local docset cache from stored text using the current embedder/schema
    Rebuild { docset: String },
    /// Smoke-test a docset with real retrieval to verify installation
    Smoke {
        /// Docset to smoke-test
        docset: String,
        /// Query to search for (default: "installation configuration example")
        query: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Number of top results to return
        #[arg(long)]
        top_k: Option<u32>,
    },
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
        /// Check model cache state, pre-downloading the embedder if missing (N5)
        #[arg(long)]
        model: bool,
        /// Repair mode: remove stale staging dirs (safe, non-destructive)
        #[arg(long)]
        repair: bool,
    },
    /// Inspect or safely clean nowdocs cache state
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache paths, sizes, installed docsets, and staging state
    Status {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Remove only nowdocs-owned staging directories older than a threshold
    CleanStaging {
        /// Minimum age to remove (examples: 30m, 2h, 1d, 3600s)
        #[arg(long, default_value = "1h")]
        older_than: String,
    },
}

#[derive(Subcommand)]
pub enum RegistryCommands {
    /// List docsets available in the registry catalog
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Search the registry catalog by name or description
    Search {
        /// Search query (matched against docset name and description)
        query: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum SetupCommands {
    /// Create one reusable setup plan for one client plus one docset
    Plan {
        /// Client to configure (claude-code, claude-desktop, cursor, generic)
        #[arg(long)]
        client: String,
        /// Docset to ensure
        #[arg(long)]
        docset: String,
        /// Fetch and validate registry metadata in memory (required to plan a new install/update)
        #[arg(long)]
        online: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Apply a previously created setup plan by hash
    Apply {
        /// Plan hash (64 lowercase hex characters)
        #[arg(long)]
        plan_hash: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Roll back a setup operation by its operation id
    Rollback {
        /// Operation id to roll back
        #[arg(long)]
        operation_id: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}
