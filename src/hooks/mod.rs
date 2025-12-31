mod commands;
mod placeholders;
mod remove_git;

use crate::{hooks::commands::RunCommands, recipes::Recipe};
use anyhow::{Context as AnyhowContext, Result};
use std::path::Path;

pub use placeholders::ReplacePlaceholders;
pub use remove_git::RemoveGit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
	PreClone,
	PostClone,
}

pub struct Context<'a> {
	pub recipe: &'a Recipe,
	pub project_dir: &'a Path,
	pub project_name: &'a str,
}

impl<'a> Context<'a> {
	pub const fn new(recipe: &'a Recipe, project_dir: &'a Path, project_name: &'a str) -> Self {
		Self {
			recipe,
			project_dir,
			project_name,
		}
	}
}

pub trait Hook {
	fn stage(&self) -> &'static [Stage];
	fn name(&self) -> &'static str;
	fn run(&self, context: &Context) -> Result<()>;
}

pub struct Registry {
	hooks: Vec<Box<dyn Hook>>,
}

impl Registry {
	pub fn new() -> Self {
		Self { hooks: Vec::new() }
	}

	pub fn with_defaults() -> Self {
		let mut registry = Self::new();

		registry.register(RemoveGit);
		registry.register(ReplacePlaceholders);
		registry.register(RunCommands);

		registry
	}

	pub fn register<H: Hook + 'static>(&mut self, hook: H) {
		self.hooks.push(Box::new(hook));
	}

	pub fn run(&self, stage: Stage, context: &Context) -> Result<()> {
		self.hooks
			.iter()
			.filter(|hook| hook.stage().contains(&stage))
			.try_for_each(|hook| {
				hook.run(context)
					.with_context(|| format!("🔴 {} FAILED", hook.name()))
			})
	}
}
