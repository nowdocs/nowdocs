use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nowdocs", version, about = "Local MCP doc server for LLM agents")]
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
    Ingest { dir: String, name: String },
    /// Package a local docset for registry contribution (text+manifest, NOT vectors)
    Share { docset: String },
    /// Remove an installed docset
    Uninstall { docset: String },
    /// List installed docsets
    ListInstalled,
    /// Update a docset to the latest registry version
    Update { docset: String },
}
