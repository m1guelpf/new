use crate::hooks::{Context, Hook, Stage};
use anyhow::{Context as AnyhowContext, Result};
use std::process::{self, Command, Stdio};

pub struct RunCommands;

impl Hook for RunCommands {
	fn name(&self) -> &'static str {
		"Run Commands"
	}

	fn stage(&self) -> &'static [Stage] {
		&[Stage::PostClone]
	}

	fn run(&self, context: &Context) -> Result<()> {
		let Some(commands) = context.recipe.config::<Vec<String>>("commands")? else {
			return Ok(());
		};

		for command in commands {
			let status = run_command(&command, context)?;

			if !status.success() {
				match status.code() {
					Some(code) => {
						anyhow::bail!("Command `{command}` failed with exit code {code}");
					},
					None => {
						anyhow::bail!("Command `{command}` terminated by signal");
					},
				}
			}
		}

		Ok(())
	}
}

fn run_command(cmd: &str, context: &Context) -> Result<process::ExitStatus> {
	let mut command = Command::new(if cfg!(windows) { "cmd" } else { "sh" });

	let command = if cfg!(windows) {
		command.args(["/C", cmd])
	} else {
		command.args(["-c", cmd])
	};

	command
		.current_dir(context.project_dir)
		.stdin(Stdio::inherit())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()
		.with_context(|| format!("Failed to run command `{cmd}`"))
}
