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
  - Or use the release workflow which imports a PFX from GitHub Secrets and runs `signtool`.

### GitHub Secrets for signing
- Configure repository secrets:
  - `SIGN_PFX_BASE64`: Your PFX certificate encoded in base64.
  - `SIGN_PFX_PASSWORD`: Password for the PFX.
  - `SIGN_TS_URL` (optional): Timestamp server URL (defaults to `http://timestamp.digicert.com`).

How to produce base64 from a PFX (PowerShell):

```powershell
$in = "C:\\path\\to\\codesign.pfx"
$out = "pfx-base64.txt"
[IO.File]::WriteAllText($out, [Convert]::ToBase64String([IO.File]::ReadAllBytes($in)))
```

Copy the contents of `pfx-base64.txt` into the `SIGN_PFX_BASE64` secret.

Notes:
- The workflow signs `.exe` and `.msi` in `src-tauri/target/release/bundle` before creating the GitHub Release, so uploaded artifacts are already signed.

## Notes
- Playwright artifacts and other build outputs are already ignored by Git.
- Cargo and npm caches are configured in CI to speed up builds.
