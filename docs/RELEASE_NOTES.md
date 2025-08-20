# Oxide Pilot v0.9.0 — Release Preparation Milestone

Date: 2025-08-19

## Highlights
- Repository cleanup complete; removed obsolete `temp-qwen-code/`.
- E2E suite stabilized on Windows (fixed HAR write contention for parallel runs).
- Documentation unified and updated; implementation status set to 90%.
- Frontend artifacts (`playwright-report/`, `test-results/`) ignored and cleaned.
- Final build checks green (Rust `cargo check`, SvelteKit production build).

## Details
- OAuth (Gemini API Key + OAuth2) and Qwen Device Code flows implemented and tested.
- Security and Performance panels integrated and covered by Playwright tests.
- Unified Cargo target directory and cleanup scripts documented.

## Breaking Changes
- None.

## Upgrade Notes
- If you previously tracked Playwright artifacts, remove them locally: they are now ignored by Git.
- For CI: cache Cargo and npm to speed up builds; ensure Playwright browsers are installed.

## Changelog
- docs: consolidate docs under `docs/`, update indices and links.
- chore: remove `temp-qwen-code/`.
- test(e2e): per-worker HAR path to avoid Windows locks.
- chore(frontend): ignore Playwright artifacts.
- docs: update implementation tasks (90% readiness) and README polish.
