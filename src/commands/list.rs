use anyhow::Result;
use std::path::Path;

use crate::recipes::{self, ReadError, Recipe, recipes_dir};

pub fn run() -> Result<()> {
	let mut recipes: Vec<_> = recipes::list()?.into_iter().collect();
	recipes.sort_by(|(left_path, left), (right_path, right)| {
		entry_name(left_path, left).cmp(&entry_name(right_path, right))
	});

	println!("Recipes directory: {}\n", recipes_dir()?.display());

	if recipes.is_empty() {
		println!("No templates installed");
		return Ok(());
	}

	for (path, recipe) in recipes {
		match recipe {
			Ok(recipe) => println!("✅ {}", recipe.name),
			Err(ReadError::Reading(error)) => {
				println!("❌ {} (unreadable)", name_from_path(&path));
				println!("  - {error}");
			},
			Err(ReadError::Parse(error)) => {
				println!("❌ {} (invalid)", name_from_path(&path));
				for error in error.iter() {
					println!("  - {}", format_parse_error(error));
				}
			},
		}
	}

	Ok(())
}

fn entry_name(path: &Path, recipe: &Result<Recipe, ReadError>) -> String {
	recipe
		.as_ref()
		.map_or_else(|_| name_from_path(path), |recipe| recipe.name.clone())
}

fn name_from_path(path: &Path) -> String {
	path.file_stem()
		.or_else(|| path.file_name())
		.and_then(|name| name.to_str())
		.map_or_else(|| path.display().to_string(), str::to_string)
}

fn format_parse_error(error: &eserde::DeserializationError) -> String {
	let message = error.message().trim();

	error
		.path()
		.filter(|path| !path.is_empty())
		.map_or_else(|| message.to_string(), |path| format!("{path}: {message}"))
}
