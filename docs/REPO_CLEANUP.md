# Repository Cleanup & Build Artifacts Guide

This document explains the standard, safe procedures to clean build artifacts and caches, and the unified target strategy for Rust/Cargo.

## Unified Cargo Target

- All Rust build artifacts are centralized under `target/` at the repository root.
- Configured in `.cargo/config.toml`:
  ```toml
  [build]
  target-dir = "target"
  ```
- Benefit: avoids duplicate `target/` folders between workspace crates (including `src-tauri/`).

## Safe Cleanup Script

Use `scripts/oxide-clean.bat` (Windows) for staged, safe cleanup:

1. Preview ignored files (recommended): `git clean -ndX`
2. Confirm deletion (optional): `git clean -fdX -e src-frontend/node_modules`
3. Remove frontend caches: `.svelte-kit`, `dist`, `node_modules/.vite`
4. Remove Rust incremental: `%TARGET_DIR%/debug/incremental`
5. Remove hidden workspace caches: `.target-workspace/`
6. Optionally run `cargo clean` in `src-tauri/`
7. Deep clean: remove unified `target/` and npm cache

Notes:
- On Windows, `node_modules` can be locked by Vite/VSCode. Close processes or remove with `npm ci`/`npx rimraf`.
- LLVM coverage `*.profraw` files are removed by the script.

## Developer Launcher

Use `scripts/oxide-dev.bat` to start dev:
- Prompts to clear frontend caches
- Lets you pick port (default 5317)
- Patches `src-tauri/tauri.conf.json` for dev, then restores
- Relies on unified Cargo target via `.cargo/config.toml`

## Workspace Hygiene

- Keep `[workspace].members` matching real directories.
- If a crate is deleted (e.g., `oxide-cognee-bridge`), remove it from `Cargo.toml` to ensure `cargo` works from the repo root.

## CI Caching

GitHub Actions workflow `.github/workflows/ci.yml` includes caching:
- Cargo registry & git index keyed by `rustc` version and `Cargo.lock`
- Unified `target/` cache keyed similarly
- Node/npm cache via `actions/setup-node@v4` with `npm ci`

## Optional: sccache

- Speeds up Rust builds locally/CI by caching compiler outputs
- To enable: install `sccache` and add to `.cargo/config.toml`:
  ```toml
  # rustc-wrapper = "sccache"
  ```

## Commands Reference

- Preview ignored deletions: `git clean -ndX`
- Delete ignored files: `git clean -fdX`
- Cargo clean (crate): `cargo clean`
- Reinstall Node deps cleanly: `npm ci`
