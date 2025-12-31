use crate::hooks::{Context, Hook, Stage, placeholders::replacer::Replacer};
use anyhow::Result;
use std::collections::HashMap;

mod replacer;

pub struct ReplacePlaceholders;

impl Hook for ReplacePlaceholders {
	fn name(&self) -> &'static str {
		"Replace Placeholders"
	}

	fn stage(&self) -> &'static [Stage] {
		&[Stage::PostClone]
	}

	fn run(&self, context: &Context) -> Result<()> {
		let mut replacements = Self::load_config(context)?;

		replacements
			.entry("NAME".to_string())
			.or_insert_with(|| context.project_name.to_string());

		let replacer = Replacer::new(replacements)?;
		replacer.apply(context.project_dir)
	}
}

impl ReplacePlaceholders {
	fn load_config(context: &Context) -> Result<HashMap<String, String>> {
		let replacements = context
			.recipe
			.config::<HashMap<String, String>>("replacements")?
			.unwrap_or_default();

		Ok(replacements)
	}
}
