# Agent Instructions for jobctl Repo

## Build, Lint, and Test Commands

- Build project: `cargo build`
- Lint (deny warnings): `cargo clippy -- -D warnings`
- Format check: `cargo fmt -- --check`
- Run all tests: `cargo test`
- Run single test: `cargo test <test_name>`
- Type check only: `cargo check`
- Run application: `cargo run -- [args]`

## Code Style Guidelines

- **Imports:** group `std`, then external crates, then `crate` modules; use explicit imports only.
- **Formatting:** enforce `rustfmt` with 4‑space indent and max 100‑char lines.
- **Types:** prefer references (e.g., `&Path` vs `&PathBuf`).
- **Naming:** `CamelCase` for types/structs/enums; `snake_case` for fns/vars; `SCREAMING_SNAKE_CASE` for consts.
- **Error handling:** use `Result`/`Option` and `?`; avoid `unwrap()`/`expect()` in libraries.
- **Serde:** apply `#[serde(rename_all = "snake_case")]` on serializable types.
- **CLI:** use `clap` derive and document all commands and flags.
- **Docs:** add `///` doc comments to public items; keep public API clean.
- **Misc:** eliminate unused vars; resolve all Clippy lints before merging.

*No Cursor rules or Copilot instructions detected in this repo.*
