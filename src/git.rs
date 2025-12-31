use anyhow::{Context, Result};
use git2::{FetchOptions, build::RepoBuilder};
use std::path::Path;

pub fn clone_repo(repo: &str, branch: Option<&str>, destination: &Path) -> Result<()> {
	let repo_url = normalize_repo(repo)?;

	let mut builder = RepoBuilder::new();
	if !is_local_repo(&repo_url) {
		let mut fetch_options = FetchOptions::new();
		fetch_options.depth(1);
		builder.fetch_options(fetch_options);
	}

	if let Some(branch) = branch {
		builder.branch(branch);
	}

	builder
		.clone(&repo_url, destination)
		.with_context(|| format!("Failed to clone template repository {repo_url}"))?;

	Ok(())
}

fn normalize_repo(repo: &str) -> Result<String> {
	let trimmed = repo.trim();

	if cfg!(test) {
		let path = Path::new(trimmed);
		if path.exists() {
			return Ok(path.to_string_lossy().to_string());
		}
	}

	if trimmed.contains("://") || trimmed.contains("github.com") || trimmed.starts_with("git@") {
		anyhow::bail!("Repository must be in the form owner/repo (no URLs)");
	}

	let mut parts = trimmed.split('/');
	let owner = parts
		.next()
		.filter(|part| !part.is_empty())
		.context("Repository must be in the form owner/repo")?;
	let repo = parts
		.next()
		.filter(|part| !part.is_empty())
		.context("Repository must be in the form owner/repo")?;

	if parts.next().is_some() {
		anyhow::bail!("Repository must be in the form owner/repo");
	}

	Ok(format!("https://github.com/{owner}/{repo}"))
}

fn is_local_repo(repo: &str) -> bool {
	repo.starts_with("file://") || Path::new(repo).exists()
}
