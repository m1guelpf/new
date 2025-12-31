use crate::hooks::{Context, Hook, Stage, placeholders::replacer::Replacer};
use anyhow::{Context as AnyhowContext, Result};
use ignore::WalkBuilder;
use inquire::Text;
use regex::Regex;
use std::{
	collections::{HashMap, HashSet},
	ffi::OsStr,
	fs,
	path::Path,
	sync::LazyLock,
};

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

		Self::prompt_for_missing_placeholders(context, &mut replacements)?;

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

	fn prompt_for_missing_placeholders(
		context: &Context,
		replacements: &mut HashMap<String, String>,
	) -> Result<()> {
		let missing = find_missing_placeholders(context.project_dir, replacements)?;
		for key in missing {
			let prompt = format!("What should {key} be replaced with?");
			let value = Text::new(&prompt)
				.prompt()
				.with_context(|| format!("Failed to prompt for {key}"))?;

			if !value.trim().is_ascii() {
				replacements.insert(key, value);
			}
		}

		Ok(())
	}
}

static PLACEHOLDER_RE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"\{\{ *([^{}]*?) *\}\}").expect("valid regex"));

fn find_missing_placeholders(
	root: &Path,
	replacements: &HashMap<String, String>,
) -> Result<Vec<String>> {
	let found = collect_placeholders(root)?;

	let mut missing: Vec<_> = found
		.into_iter()
		.filter(|key| !replacements.contains_key(key))
		.collect();

	missing.sort();

	Ok(missing)
}

fn collect_placeholders(root: &Path) -> Result<HashSet<String>> {
	let entries = walk_entries(root)?;
	let mut keys = HashSet::new();

	for entry in entries {
		let path = entry.path();
		if path != root
			&& let Some(name) = path.file_name().and_then(|name| name.to_str())
		{
			extract_placeholders(name, &mut keys);
		}

		if !entry.file_type().is_some_and(|ft| ft.is_file()) {
			continue;
		}

		let contents =
			fs::read(path).with_context(|| format!("Failed to read file {}", path.display()))?;

		if contents.contains(&0) {
			continue;
		}

		let Ok(text) = str::from_utf8(&contents) else {
			continue;
		};

		extract_placeholders(text, &mut keys);
	}

	Ok(keys)
}

fn extract_placeholders(input: &str, keys: &mut HashSet<String>) {
	for caps in PLACEHOLDER_RE.captures_iter(input) {
		if let Some(matched) = caps.get(1) {
			let trimmed = matched.as_str().trim();
			if !trimmed.is_empty() {
				keys.insert(trimmed.to_string());
			}
		}
	}
}

fn walk_entries(root: &Path) -> Result<Vec<ignore::DirEntry>> {
	WalkBuilder::new(root)
		.hidden(false)
		.git_ignore(false)
		.git_exclude(false)
		.git_global(false)
		.filter_entry(|entry| entry.file_name() != OsStr::new(".git"))
		.build()
		.map(|entry| entry.context("Failed to read directory entry"))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::extract_placeholders;
	use std::collections::HashSet;

	#[test]
	fn extract_placeholders_trims_and_collects() {
		let mut keys = HashSet::new();
		extract_placeholders("Hello {{NAME}} and {{ APP_ID }}!", &mut keys);

		assert!(keys.contains("NAME"));
		assert!(keys.contains("APP_ID"));
		assert_eq!(keys.len(), 2);
	}

	#[test]
	fn extract_placeholders_ignores_empty() {
		let mut keys = HashSet::new();
		extract_placeholders("{{}} {{   }}", &mut keys);
		assert!(keys.is_empty());
	}

	#[test]
	fn extract_placeholders_ignores_nested_braces() {
		let mut keys = HashSet::new();
		extract_placeholders("{{OUTER {{INNER}} OUTER}}", &mut keys);

		assert!(keys.contains("INNER"));
		assert_eq!(keys.len(), 1);
	}
}
