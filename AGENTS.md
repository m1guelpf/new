# Repository Guidelines

## Project Structure & Module Organization

-   `Cargo.toml`: Package metadata and dependencies.
-   `README.md`: User-facing usage and recipe format.
-   `src/`: Rust source code for the CLI. Entry point in `src/main.rs`.
-   Tests: no dedicated `tests/` directory; unit tests should live in `#[cfg(test)]` modules next to the code they cover.

## Build, Test, and Development Commands

Run all cargo commands with `RUSTC_WRAPPER= cargo $1` to avoid permission issues.

-   `cargo test`: run unit tests.
-   `cargo build`: compile the CLI binary.
-   `cargo fmt`: format the codebase with rustfmt.
-   `cargo clippy`: run lint checks aligned with the project’s `clippy` warnings.
-   `cargo run -- <template> <directory>`: run locally, e.g., `cargo run -- ios-app MyProject`.

## Coding Style

-   After each change you make, run `cargo clippy` and fix any issues it highlights.
-   Keep modules small and focused; hooks belong in `src/hooks/` and should implement the `Hook` trait.
-   Use `snake_case` for functions/modules, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
-   Keep the source as idiomatic and Rust-like as possible; prefer standard library patterns and clear ownership.
-   When introducing or using third-party crates, first consult docs.rs via MCP (use `mcp__DocsRsMCP__docs_rs_readme` or `mcp__DocsRsMCP__docs_rs_get_item`).
-   The project uses custom formatting rules defined in `rustfmt.toml`. Run `cargo fmt` to format at the end of your turn.

## Testing Guidelines

-   No framework beyond Rust’s built-in test harness.
-   Add unit tests in `#[cfg(test)]` modules in the same file.
-   Run `cargo test` before submitting changes that affect logic.

## Commit & Pull Request Guidelines

-   Commit history uses short, imperative messages (e.g., "replace project name"). Keep messages concise and action-focused.
-   PRs should include: a brief summary, rationale, and how you verified the change (commands or manual steps). Note any new flags, hooks, or recipe schema changes.

## Configuration & Recipes

-   Each recipe is a TOML file with `[recipe]`.
-   Recipes are read from the user config directory (macOS example: `${config_dir()}/build.m1guelpf.new/recipes`).
