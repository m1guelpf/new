# new-cli

Quickly create new projects from templates

## Usage

Recipes live in your config directory at `${config_dir}/build.m1guelpf.new/recipes`. You can quickly open that folder in your editor by running `new edit`.

Each recipe is a TOML file that looks like this:

```toml
[recipe]
name = "ios-app"
repo = "m1guelpf/ios-template"
branch = "main"

[recipe.replacements]
APP_ID = "com.example.app"

commands = ["git init", "git add ."]
```

Run it with:

```sh
new ios-app MyProject
```

Placeholders are written as `{{KEY}}`. The `NAME` key is always available and defaults to the
project directory name.

## Writing recipes

Every recipe is a TOML file with a single `[recipe]` table. Required keys are `name` and `repo`.
`branch` is optional and defaults to the repo default branch.

```toml
[recipe]
name = "ios-app"
repo = "m1guelpf/ios-template"
branch = "main"

[recipe.replacements]
APP_ID = "com.example.app"
TEAM_ID = "ABCDE12345"

commands = ["git init", "git add ."]
```

### Hooks (optional)

Hooks run automatically after the template is cloned. The CLI ships with the following built-in
hooks, two of which are configurable from the recipe file:

-   Replace placeholders (optional):
    -   Configure with `[recipe.replacements]`.
    -   Any `{{KEY}}` placeholders in file names, directory names, or file contents are replaced.
    -   Missing keys are prompted interactively.
    -   `NAME` is always available (defaults to the project directory name).
-   Run commands (optional):
    -   Configure with `commands = ["..."]` under `[recipe]`.
    -   Commands are executed in the project directory after cloning.

## Installation

```sh
cargo install new-cli
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
