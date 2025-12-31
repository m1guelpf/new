use std::process::Command;

use anyhow::{Context, Result};

use crate::recipes;

pub fn run(editor: Option<String>) -> Result<()> {
	let dir = recipes::recipes_dir()?;
	let editor = editor.unwrap_or_else(|| "code".to_string());

	let status = Command::new(editor)
		.arg(dir)
		.status()
		.context("Failed to launch editor")?;

	if !status.success() {
		return Err(anyhow::anyhow!("Editor exited with a non-zero status"));
	}

	Ok(())
}
