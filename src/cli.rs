//! Command-line interface definitions and dispatch.

use clap::Subcommand;

/// Top-level `sgt` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Print current daemon/status information.
    Status,
}

/// Dispatch a parsed CLI command.
pub fn run(command: Command) {
    match command {
        Command::Status => println!("sgt status: not yet implemented"),
    }
}
