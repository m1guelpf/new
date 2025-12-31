use anyhow::{Context, Result};
use dirs::config_dir;
use eserde::Deserialize;
use serde::de::DeserializeOwned;
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
};

use crate::{git, hooks};

#[derive(Debug, Deserialize)]
struct RecipeDeclaration {
	recipe: Recipe,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
	pub name: String,
	pub repo: String,
	#[serde(default)]
	pub branch: Option<String>,
	#[eserde(compat)]
	#[serde(default, flatten)]
	pub extra: toml::value::Table,
}

impl Recipe {
	pub fn find(name: &str) -> Result<Self> {
		let recipes = load()?;

		recipes
			.into_iter()
			.find(|recipe| recipe.name == name)
			.with_context(|| {
				format!(
					"Recipe {name} not found. Make sure you have a recipe named {name} in your recipes directory"
				)
			})
	}

	pub fn run(&self, directory: &Path, name: &str) -> Result<()> {
		let registry = hooks::Registry::with_defaults();
		let context = hooks::Context::new(self, directory, name);

		registry.run(hooks::Stage::PreClone, &context)?;
		git::clone_repo(&self.repo, self.branch.as_deref(), directory)?;
		registry.run(hooks::Stage::PostClone, &context)
	}
}

impl Recipe {
	pub fn config<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
		let Some(value) = self.extra.get(key) else {
			return Ok(None);
		};

		let config = value
			.clone()
			.try_into()
			.with_context(|| format!("Failed to parse recipe.{key} config"))?;

		Ok(Some(config))
	}
}

pub fn load() -> Result<Vec<Recipe>> {
	let files = recipe_files()?;

	Ok(files
		.into_iter()
		.map(|path| read(&path))
		.collect::<Result<Vec<_>, _>>()?)
}

pub fn list() -> Result<HashMap<PathBuf, Result<Recipe, ReadError>>> {
	let mut recipes = HashMap::new();

	for path in recipe_files()? {
		let recipe = read(&path);
		recipes.insert(path, recipe);
	}

	Ok(recipes)
}

pub fn recipes_dir() -> Result<PathBuf> {
	let config_root = config_dir().context("Unable to resolve configuration directory")?;

	Ok(Path::new(&config_root)
		.join("build.m1guelpf.new")
		.join("recipes"))
}

fn recipe_files() -> Result<Vec<PathBuf>> {
	let recipes_dir = recipes_dir()?;
	fs::create_dir_all(&recipes_dir).with_context(|| {
		format!(
			"Failed to create recipes directory {}",
			recipes_dir.display()
		)
	})?;

	let files = fs::read_dir(&recipes_dir)
		.with_context(|| format!("Failed to read recipes directory {}", recipes_dir.display()))?
		.filter_map(Result::ok)
		.map(|entry| entry.path())
		.filter(|path| path.is_file())
		.collect::<Vec<_>>();

	Ok(files)
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
	#[error("Failed to read recipe")]
	Reading(#[from] std::io::Error),
	#[error("Failed to parse recipe")]
	Parse(#[from] eserde::DeserializationErrors),
}

fn read(path: &Path) -> Result<Recipe, ReadError> {
	let content = fs::read_to_string(path)?;

	Ok(eserde::toml::from_str::<RecipeDeclaration>(&content).map(|decl| decl.recipe)?)
}
