use anyhow::{Context, Result};
use clap::Parser;
use inquire::Text;
use path_absolutize::Absolutize;
use std::{
	fs,
	path::{Path, PathBuf},
};

use crate::recipes::Recipe;

#[derive(Debug, Parser)]
pub struct InitArgs {
	/// Template recipe to use for the new project
	#[clap(required = true)]
	template: Option<String>,

	/// Directory where to create the new project
	directory: Option<PathBuf>,
}

pub fn run(args: &InitArgs) -> Result<()> {
	let recipe = Recipe::find(
		args.template
			.as_deref()
			.context("Missing template recipe. Use `new list` to see available templates")?,
	)?;

	let directory = resolve_directory(args.directory.clone())?;
	ensure_directory_available(&directory)?;

	let name = project_name(&directory)?;

	recipe.run(&directory, &name)
}

fn resolve_directory(directory: Option<PathBuf>) -> Result<PathBuf> {
	let directory = directory
		.or_else(|| {
			Text::new("What is your project named?")
				.prompt()
				.ok()
				.map(PathBuf::from)
		})
		.context("Missing project directory")?;

	Ok(directory
		.absolutize()
		.context("Failed to resolve project directory")?
		.into_owned())
}

fn project_name(directory: &Path) -> Result<String> {
	directory
		.file_name()
		.and_then(|name| name.to_str())
		.map(str::to_string)
		.context("Invalid project directory")
}

fn ensure_directory_available(directory: &Path) -> Result<()> {
	if directory.is_dir() {
		if directory
			.read_dir()
			.with_context(|| format!("Failed to read project directory {}", directory.display()))?
			.next()
			.is_some()
		{
			anyhow::bail!("Project directory already exists and is not empty");
		}

		return Ok(());
	}

	if directory.exists() {
		anyhow::bail!("Project path already exists and is not a directory");
	}

	if let Some(parent) = directory.parent() {
		fs::create_dir_all(parent)
			.with_context(|| format!("Failed to create parent directory {}", parent.display()))?;
	}

	Ok(())
}
