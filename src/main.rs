//! Entry point for the `sgt` binary.

use clap::Parser;
use sergeant_rs::cli;

/// `sgt` — sergeant-rs command-line entry point.
#[derive(Parser, Debug)]
#[command(name = "sgt", version, about = "sergeant-rs")]
struct Sgt {
    #[command(subcommand)]
    command: cli::Command,
}

fn main() {
    let sgt = Sgt::parse();
    cli::run(sgt.command);
}
