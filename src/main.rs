//! Entry point for the `sgt` binary.

mod api;
mod backend;
mod cli;
mod daemon;
mod domain;
mod runtime;
mod telemetry;
mod tui;
mod web;

use clap::Parser;

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
