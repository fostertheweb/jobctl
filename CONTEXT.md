# CONTEXT.md

## Build, Lint, and Test Commands

- **Build:**  
  `cargo build`
- **Lint:**  
  `cargo clippy -- -D warnings` (treat warnings as errors)
- **Format:**  
  `cargo fmt -- --check` (auto-format, enforce rustfmt)
- **Test:**  
  `cargo test`  
  *No tests exist now; to run one:*
  `cargo test <test_name>`
- **Check only:**  
  `cargo check`
- **Run:**  
  `cargo run -- [args]`

## Code Style Guidelines

- **Imports:**  Group std, then external, then crate modules. Use explicit (not glob) imports.
- **Formatting:**  `rustfmt` required. 4-space indent, ≤ 100-char lines.
- **Types:**  Take arguments by reference when possible (e.g., `&Path`, not `&PathBuf`).
- **Naming:**  Types/enums/structs: `CamelCase`, vars & fns: `snake_case`, consts: `SCREAMING_SNAKE_CASE`.
- **Error Handling:**  Use `Result`/`Option`, prefer `?` operator, avoid `unwrap()` and `expect()` in libs.
- **Serde:**  Add `#[serde(rename_all = "snake_case")]` to serializable types for API consistency.
- **Command-line:**  Use `clap` derive for CLIs, document all commands and flags.
- **Docs:**  Public items have `///` doc-comments. Keep public API clean.
- **Misc:**  Avoid unused vars; address all Clippy lints before merging.
