use crate::hooks::{Context, Hook, Stage};
use anyhow::Result;
use std::fs;

pub struct RemoveGit;

impl Hook for RemoveGit {
	fn name(&self) -> &'static str {
		"Remove .git directory from template"
	}

	fn stage(&self) -> &'static [Stage] {
		&[Stage::PostClone]
	}

	fn run(&self, context: &Context) -> Result<()> {
		let git_dir = context.project_dir.join(".git");
		if git_dir.is_dir() {
			fs::remove_dir_all(git_dir)?;
		}

		Ok(())
	}
}
