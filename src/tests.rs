use crate::recipes::Recipe;
use anyhow::Result;
use git2::{Repository, Signature};
use std::{fs, path::Path};
use tempdir::TempDir;

fn init_template_repo() -> Result<TempDir> {
	let template_dir = TempDir::new("new-cli-template")?;
	let root = template_dir.path();

	fs::write(root.join("README.md"), "Hello {{NAME}} ({{APP_ID}})")?;

	let nested_dir = root.join("{{NAME}}");
	fs::create_dir_all(&nested_dir)?;
	fs::write(nested_dir.join("config-{{APP_ID}}.txt"), "id={{APP_ID}}")?;

	let repo = Repository::init(root)?;
	let mut index = repo.index()?;
	index.add_path(Path::new("README.md"))?;
	index.add_path(Path::new("{{NAME}}/config-{{APP_ID}}.txt"))?;
	index.write()?;

	let tree_id = index.write_tree()?;
	let tree = repo.find_tree(tree_id)?;
	let signature = Signature::now("Test User", "test@example.com")?;
	repo.commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])?;

	Ok(template_dir)
}

fn build_recipe(repo_path: &Path) -> Recipe {
	let mut replacements = toml::value::Table::new();
	replacements.insert(
		"APP_ID".to_string(),
		toml::Value::String("com.example.app".to_string()),
	);

	let mut extra = toml::value::Table::new();
	extra.insert("replacements".to_string(), toml::Value::Table(replacements));
	extra.insert(
		"commands".to_string(),
		toml::Value::Array(vec![toml::Value::String(
			"echo done > done.txt".to_string(),
		)]),
	);

	Recipe {
		name: "local".to_string(),
		repo: repo_path.to_string_lossy().to_string(),
		branch: None,
		extra,
	}
}

#[test]
fn recipe_run_applies_hooks_end_to_end() -> Result<()> {
	let template_dir = init_template_repo()?;
	let project_root = TempDir::new("new-cli-project")?;
	let project_dir = project_root.path().join("MyProject");

	let recipe = build_recipe(template_dir.path());
	recipe.run(&project_dir, "MyProject")?;

	assert!(!project_dir.join(".git").exists());
	assert!(project_dir.join("MyProject").is_dir());
	assert!(
		project_dir
			.join("MyProject")
			.join("config-com.example.app.txt")
			.is_file()
	);
	assert!(project_dir.join("done.txt").is_file());

	let readme = fs::read_to_string(project_dir.join("README.md"))?;
	assert!(readme.contains("MyProject"));
	assert!(readme.contains("com.example.app"));

	Ok(())
}
