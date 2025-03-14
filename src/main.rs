#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::Parser;
use dirs::config_dir;
use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{BinaryDetection, SearcherBuilder, Sink, sinks::UTF8},
};
use ignore::{DirEntry, Walk};
use inquire::Text;
use path_absolutize::Absolutize;
use std::{
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

    initialize_project(&directory, name).unwrap();
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

fn initialize_project<P: AsRef<Path>>(dir: P, name: &str) -> io::Result<()> {
    let files = Walk::new(dir)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?
        .into_iter()
        .map(DirEntry::into_path)
        .filter(|path| path.is_file());

    let matcher = RegexMatcher::new(r"\{\{NAME}}").unwrap();
    let mut searcher = SearcherBuilder::new()
        .line_number(false)
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();

    for file in files {
        searcher.search_path(
            &matcher,
            &file,
            SinkFn(|| {
                let contents = read_to_string(&file)?;

                fs::write(&file, contents.replace("{{NAME}}", name))
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
