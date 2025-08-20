# Release Process

This guide describes how to publish a release using Git tags and GitHub Actions.

## Prerequisites
- Ensure CI is green (`Windows CI` and `CI` workflows).
- Optional branding: add `src-tauri/icon.png` (1024x1024) before tagging.
- Optional signing: have code-signing PFX and password available.

## Steps
1. Update release notes at `docs/RELEASE_NOTES.md`.
2. Create a tag (semantic version recommended):
   ```powershell
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
3. GitHub Actions will run `.github/workflows/release.yml` on the tag and:
   - Build Windows bundles (NSIS/MSI) via Tauri.
   - Attach artifacts to a GitHub Release.

## Code Signing (optional)
- For signed installers, you can:
  - Sign locally using `scripts/build-windows.ps1` by setting env vars:
    - `SIGNTOOL`, `SIGN_CERT` (path to PFX), `SIGN_PASS`, `SIGN_TS_URL`.
  - Or extend the release workflow to import a PFX from GitHub Secrets and run `signtool`.

## Notes
- Playwright artifacts and other build outputs are already ignored by Git.
- Cargo and npm caches are configured in CI to speed up builds.
