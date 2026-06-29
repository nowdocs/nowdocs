use clap::Parser;
use nowdocs::cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Serve => nowdocs::mcp::run_loop().expect("mcp loop error"),
        Commands::Install { docset } => println!("install {}", docset),
        Commands::Ingest { dir, name } => println!("ingest {} -> {}", dir, name),
        Commands::Share { docset } => println!("share {}", docset),
        Commands::Uninstall { docset } => println!("uninstall {}", docset),
        Commands::ListInstalled => println!("list-installed"),
        Commands::Update { docset } => println!("update {}", docset),
    }
}
