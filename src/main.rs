#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;
use dirs::config_dir;
use grep::{
    regex::RegexMatcherBuilder,
    searcher::{BinaryDetection, SearcherBuilder, Sink},
};
use ignore::{DirEntry, Walk};
use inquire::Text;
use path_absolutize::Absolutize;
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io::{self},
    path::{Path, PathBuf},
    process::{self, Stdio},
};

#[derive(Debug, serde::Deserialize)]
struct RecipeDeclaration {
    recipe: Recipe,
}

#[derive(Debug, serde::Deserialize)]
struct Recipe {
    name: String,
    command: String,
    #[serde(default)]
    replacements: HashMap<String, String>,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Command {
    template: String,
    directory: Option<PathBuf>,
}

fn main() {
    let command = Command::parse();

    let Some(recipe) = get_recipes()
        .unwrap()
        .into_iter()
        .find(|recipe| recipe.name == command.template)
    else {
        eprintln!(
            "Recipe {} not found. Make sure you have a recipe named {} in your recipes directory",
            command.template, command.template
        );

        process::exit(1);
    };

    let directory = command
        .directory
        .or_else(|| {
            Text::new("What is your project named?")
                .prompt()
                .ok()
                .map(PathBuf::from)
        })
        .map(|path| PathBuf::from(path.absolutize().unwrap()))
        .unwrap();

    let name = directory
        .components()
        .next_back()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    let process = process::Command::new("sh")
        .arg("-c")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg(
            recipe
                .command
                .replace("{DIR}", directory.to_str().unwrap())
                .replace("{NAME}", name),
        )
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !process.success() {
        eprintln!("Failed to initialize project");
        process::exit(process.code().unwrap_or(1));
    }

    initialize_project(&directory, name, &recipe).unwrap();
}

fn get_recipes() -> io::Result<Vec<Recipe>> {
    let recipes_dir = config_dir()
        .unwrap()
        .join("build.m1guelpf.new")
        .join("recipes");

    fs::create_dir_all(&recipes_dir)?;

    let files = fs::read_dir(recipes_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .map(|path| fs::read_to_string(&path).map(|contents| (path, contents)))
        .collect::<Result<Vec<_>, _>>()?;

    files
        .into_iter()
        .map(|(_, content)| toml::from_str::<RecipeDeclaration>(&content).map(|decl| decl.recipe))
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)
}

fn initialize_project<P: AsRef<Path>>(dir: P, name: &str, recipe: &Recipe) -> io::Result<()> {
    let mut replacements = recipe.replacements.clone();
    replacements.insert("NAME".to_string(), name.to_string());
    replacements = replacements
        .into_iter()
        .map(|(key, value)| (format!("{{{{{}}}}}", regex::escape(&key)), value))
        .collect::<HashMap<_, _>>();

    let all_keys_regex =
        regex::RegexSet::new(replacements.keys().map(|key| regex::escape(key))).unwrap();

    let files = Walk::new(dir)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?
        .into_iter()
        .map(DirEntry::into_path)
        .filter(|path| path.is_file())
        .map(|path| {
            let path_str = path.to_str().unwrap();
            if !all_keys_regex.is_match(path_str) {
                return path;
            }

            let mut new_path = path_str.to_string();
            for (key, value) in &replacements {
                new_path = new_path.replace(key, value);
            }

            fs::rename(path, &new_path).unwrap();

            PathBuf::from(new_path)
        });

    let matcher = RegexMatcherBuilder::new()
        .build_many(
            &replacements
                .keys()
                .map(|key| regex::escape(key))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let mut searcher = SearcherBuilder::new()
        .line_number(false)
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();

    for file in files {
        searcher.search_path(
            &matcher,
            &file,
            SinkFn(|| {
                let mut contents = read_to_string(&file)?;

                for (key, value) in &replacements {
                    contents = contents.replace(key, value);
                }

                fs::write(&file, contents)
            }),
        )?;
    }

    Ok(())
}

pub struct SinkFn<F: Fn() -> io::Result<()>>(pub F);

impl<F: Fn() -> io::Result<()>> Sink for SinkFn<F> {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep::searcher::Searcher,
        _mat: &grep::searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        self.0()?;

        Ok(false)
    }
}
