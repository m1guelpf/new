#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod commands;
mod git;
mod hooks;
mod recipes;

use anyhow::Result;
use clap::Parser;
use commands::init::InitArgs;

use crate::commands::Commands;

#[derive(Debug, Parser)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(author, version, about, long_about = None)]
struct Cli {
	#[clap(subcommand)]
	command: Option<Commands>,

	#[clap(flatten)]
	init: InitArgs,
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	match cli.command {
		None => commands::init::run(&cli.init),
		Some(Commands::List) => commands::list::run(),
		Some(Commands::Init(args)) => commands::init::run(&args),
		Some(Commands::Edit { editor }) => commands::edit::run(editor),
	}
}

#[cfg(test)]
mod tests;
