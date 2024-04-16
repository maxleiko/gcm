#![feature(io_error_more)]

mod cmd;
mod package;
mod registry;

use anyhow::Result;
use clap::{Parser, Subcommand};

use cmd::*;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    List(List),
    Install(Install),
    Update(Update),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List(cmd) => cmd.run(),
        Command::Install(cmd) => cmd.run(),
        Command::Update(cmd) => cmd.run(),
    }
}
