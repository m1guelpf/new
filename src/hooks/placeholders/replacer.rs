use anyhow::{Context, Result};
use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use ignore::WalkBuilder;
use regex::escape as regex_escape;
use std::{
	cmp::Reverse,
	collections::HashMap,
	ffi::OsStr,
	fs,
	path::{Path, PathBuf},
};

pub struct Replacer {
	matcher: grep_regex::RegexMatcher,
	replacements: HashMap<Vec<u8>, Vec<u8>>,
}

impl Replacer {
	pub fn new(replacements: HashMap<String, String>) -> Result<Self> {
		let mut literals = Vec::with_capacity(replacements.len());
		let mut replacement_map = HashMap::with_capacity(replacements.len());

		for (key, value) in replacements {
			let placeholder = format!("{{{{{key}}}}}");
			literals.push(placeholder.clone());
			replacement_map.insert(placeholder.into_bytes(), value.into_bytes());
		}

		let pattern = literals
			.iter()
			.map(|literal| regex_escape(literal))
			.collect::<Vec<_>>()
			.join("|");
		let matcher = RegexMatcherBuilder::new()
			.build(&pattern)
			.context("Failed to build placeholder matcher")?;

		Ok(Self {
			matcher,
			replacements: replacement_map,
		})
	}

	pub fn apply(&self, root: &Path) -> Result<()> {
		self.rename_directories(root)?;
		self.rename_files(root)?;
		self.replace_file_contents(root)?;
		Ok(())
	}

	fn rename_directories(&self, root: &Path) -> Result<()> {
		let entries = walk_entries(root)?;
		let mut dirs: Vec<PathBuf> = entries
			.iter()
			.filter(|entry| entry.file_type().is_some_and(|ft| ft.is_dir()))
			.map(|entry| entry.path().to_path_buf())
			.collect();

		dirs.sort_by_key(|dir| Reverse(dir.components().count()));

		for dir in dirs {
			if dir == root {
				continue;
			}

			let Some(name) = dir.file_name().and_then(|name| name.to_str()) else {
				continue;
			};

			let replaced = self.replace_text(name)?;
			if replaced == name {
				continue;
			}

			let Some(parent) = dir.parent() else {
				continue;
			};

			fs::rename(&dir, parent.join(replaced))?;
		}

		Ok(())
	}

	fn rename_files(&self, root: &Path) -> Result<()> {
		let entries = walk_entries(root)?;
		for entry in entries {
			if !entry.file_type().is_some_and(|ft| ft.is_file()) {
				continue;
			}

			let path = entry.path();
			let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
				continue;
			};

			let replaced = self.replace_text(name)?;
			if replaced == name {
				continue;
			}

			let Some(parent) = path.parent() else {
				continue;
			};

			fs::rename(path, parent.join(replaced))?;
		}

		Ok(())
	}

	fn replace_file_contents(&self, root: &Path) -> Result<()> {
		let entries = walk_entries(root)?;
		for entry in entries {
			if !entry.file_type().is_some_and(|ft| ft.is_file()) {
				continue;
			}

			let path = entry.path();
			let contents = fs::read(path)
				.with_context(|| format!("Failed to read file {}", path.display()))?;

			if contents.contains(&0) || std::str::from_utf8(&contents).is_err() {
				continue;
			}

			if let Some(replaced) = self.replace_bytes(&contents)? {
				fs::write(path, replaced)
					.with_context(|| format!("Failed to write file {}", path.display()))?;
			}
		}

		Ok(())
	}

	fn replace_text(&self, input: &str) -> Result<String> {
		let Some(replaced) = self.replace_bytes(input.as_bytes())? else {
			return Ok(input.to_string());
		};

		String::from_utf8(replaced).context("Failed to decode replacement result")
	}

	fn replace_bytes(&self, input: &[u8]) -> Result<Option<Vec<u8>>> {
		let mut output = Vec::with_capacity(input.len());
		let mut did_replace = false;
		self.matcher.replace(input, &mut output, |matched, dst| {
			let needle = &input[matched];
			if let Some(replacement) = self.replacements.get(needle) {
				dst.extend_from_slice(replacement);
				did_replace = true;
			} else {
				dst.extend_from_slice(needle);
			}
			true
		})?;

		if did_replace {
			Ok(Some(output))
		} else {
			Ok(None)
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
