use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use nowdocs::registry_build::{build_registry_release, RegistryReleaseBuild};

#[derive(Parser)]
#[command(
    name = "nowdocs-registry-build",
    about = "Build a trusted registry Lance artifact"
)]
struct Args {
    #[arg(long)]
    bundle: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    docset: String,
    #[arg(long)]
    source_url_base: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output = build_registry_release(&RegistryReleaseBuild {
        bundle_dir: args.bundle,
        output_dir: args.output,
        public_docset: args.docset,
        source_url_base: args.source_url_base,
    })?;
    println!("{}", output.display());
    Ok(())
}
