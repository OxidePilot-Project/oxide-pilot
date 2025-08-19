---
description: Debug Fixer (Global, Robust Bug Investigation Workflow)
---

# Debug Fixer (Global, Robust Bug Investigation Workflow)

This workflow defines a rigorous, project-agnostic debugging procedure for Cascade to identify, reproduce, diagnose, and fix bugs. It also generates a high-quality Markdown report with metadata and artifacts, suitable for both technical and non-technical stakeholders.

Note: This workflow is global. Do not add project-specific paths beyond creating a local docs/bugs structure for reports.

// turbo-all

1) Preparation and context capture (non-destructive)
   - Create report directories if missing (Windows-safe):
     - PowerShell
       - powershell -NoProfile -ExecutionPolicy Bypass -Command "New-Item -ItemType Directory -Path 'docs/bugs/_dev' -Force | Out-Null"
   - Capture environment context for reproducibility:
     - git rev-parse --abbrev-ref HEAD
     - git rev-parse --short HEAD
     - git status --porcelain
     - node -v && npm -v (or pnpm -v / bun -v)
     - Print OS, shell, and any relevant runtime versions

2) Reproduce quickly and collect artifacts (automated where available)
   - Prefer running the project’s test suite (e.g., Playwright) with diagnostics enabled:
     - npm run test:e2e || npm test
     - Ensure screenshots, videos, traces, HAR, and console/network logs are enabled on failure.
   - If no tests exist, attempt to reproduce via minimal steps and capture:
     - Terminal logs (stdout/stderr)
     - Application/server logs
     - Screenshots (CLI or framework-level)

3) Initial evidence review
   - Parse errors and stack traces; extract the top failing specs, steps, or endpoints.
   - Identify recently changed files or dependencies likely correlated with the failure (e.g., via git diff HEAD~N).
   - If MCP servers are available (e.g., log analyzers, code search, runtime inspectors), use them to augment evidence.

4) Form 3 hypotheses with confidence scores
   - Propose at least three plausible root causes.
   - Assign each a probability (e.g., 0–100%).
   - List what evidence supports or contradicts each hypothesis and the quickest way to validate/falsify.

5) Web research and prior art
   - Search official docs, GitHub issues, and community posts for the exact error signatures and affected packages.
   - Cross-check fixes and known regressions. Update hypothesis probabilities using new info.

6) Decide and implement fixes (quick vs. robust)
   - Quick fix: a minimal change to restore functionality with low risk and minimal scope.
   - Robust fix: a comprehensive solution addressing the root cause with tests and docs.
   - Prefer adding regression tests (unit/integration/E2E) and telemetry/logging to prevent recurrence.

7) Verify and iterate
   - Re-run the test suite with diagnostics enabled.
   - Confirm the failure is resolved across relevant environments/browsers.
   - If flakiness remains, improve selectors, timeouts, retries, or isolation until stable.

8) Generate a report (Markdown) under docs/bugs/_dev
   - File name: docs/bugs/_dev/YYYY-MM-DD-`short-id-or-slug`.md
   - Frontmatter metadata (example)

```yaml
---
title: BUG_TITLE
id: SHORT_ID
date: ISO_TIMESTAMP
severity: low|medium|high|critical
status: open|fixed|wontfix
projectVersion: SEMVER_OR_COMMIT
branch: GIT_BRANCH
commit: GIT_SHORT_SHA
relatedDocs: [LINKS_TO_TASKS_OR_SPECS]
affectedFiles: [PATHS]
reproducible: yes|no
environments: ["win", "mac", "linux"]
tools:
  tests: ["playwright", "jest"]
  artifacts: ["screenshots", "videos", "traces", "HAR", "logs"]
---
```

   - Report sections (concise, clear language):
  - Summary: brief description for non-technical readers.
  - Reproduction Steps: exact steps and environment to reproduce.
  - Evidence: key logs, stack traces, failing test names, artifacts locations.
  - Root Cause Analysis: the 3 hypotheses with probabilities, and the final diagnosis.
  - Solutions:
    - Quick Fix: steps, code areas, trade-offs.
    - Robust Fix: steps, code areas, tests to add, risks.
  - Follow-ups: tickets/links, monitoring/telemetry additions, documentation updates.

9) Archive artifacts (optional but recommended)
   - Copy or link test artifacts (e.g., Playwright test-results, traces, videos) into a subfolder adjacent to the report, or store a manifest with their paths.
   - Ensure sensitive data is scrubbed before archiving.

Notes on MCP-driven debugging (optional)

- If an MCP browser/debug server is available, Cascade may use it to augment investigation (e.g., console capture, DOM snapshots, network logs).
- Headless-only environments may limit extension-based MCP servers; prefer test-runner-native artifacts (trace, HAR, screenshots) for CI.
