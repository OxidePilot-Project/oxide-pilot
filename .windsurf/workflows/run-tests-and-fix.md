---
description: tests-and-fix
---

# Run Tests and Fix - Oxide Pilot Testing Workflow

**Description**: Comprehensive testing workflow that runs unit tests, integration tests, performance tests, and automatically fixes common errors to ensure code quality before committing, merging, or deploying.

**Author**: Oxide Pilot Development Team  
**Version**: 1.0  
**Last Updated**: July 2025

---

## Workflow Steps

### 1. Environment Verification

First, verify that all required tools and dependencies are properly installed and configured:

```bash
# Check Rust installation and version
rustc --version
cargo --version

# Check Node.js and npm
node --version
npm --version

# Check Tauri CLI
cargo tauri --version

# If any tools are missing, install them automatically
```

If any critical tools are missing:

- Install Rust from <https://rustup.rs/> if not found
- Install Node.js from <https://nodejs.org/> if not found
- Install Tauri CLI with `cargo install tauri-cli` if not found

### 2. Pre-Test Setup and Cleanup

Clean any previous build artifacts and prepare the environment:

```bash
# Clean previous builds
rm -rf src-tauri/target/debug
rm -rf src-tauri/target/release
rm -rf src-frontend/dist
rm -rf target/debug
rm -rf target/release

# Clean test artifacts
cargo clean
```

### 3. Dependency Installation and Updates

Ensure all dependencies are up to date:

```bash
# Update Rust dependencies
cargo update

# Install/update frontend dependencies
cd src-frontend
npm install
npm audit fix --force
cd ..

# Check for security vulnerabilities
cargo audit
```

If `cargo audit` is not installed, install it with `cargo install cargo-audit`.

### 4. Code Quality and Linting

Run code formatters and linters to catch style issues:

```bash
# Format Rust code
cargo fmt --all

# Run Clippy for Rust linting
cargo clippy --all-targets --all-features -- -D warnings

# Format and lint frontend code with Biome
cd src-frontend
npx @biomejs/biome format --write .
npx @biomejs/biome check --write --unsafe .
cd ..
```

If any formatting or linting errors are found, automatically fix them where possible.

### 5. Run Rust Tests & Generate Coverage Report with `grcov`

This step uses a two-phase process with `grcov` to generate a robust code coverage report. The `-Cinstrument-coverage` flag is now set globally in `.cargo/config.toml`.

#### Step 5a: Generate Coverage Data

First, we run the tests, which will now automatically generate raw coverage data thanks to the global configuration.

```bash
# Clean previous coverage data and create directory
Remove-Item -Recurse -Force -Path "target/coverage", "*.profraw" -ErrorAction SilentlyContinue
New-Item -Path "target/coverage" -ItemType Directory -ErrorAction SilentlyContinue

# Set environment variables and run tests to generate .profraw files
// turbo
$env:CARGO_INCREMENTAL=0
$env:LLVM_PROFILE_FILE='target/coverage/cargo-test-%p-%m.profraw'
cargo test --all-features -- --nocapture

# Unset environment variables
Remove-Item Env:\LLVM_PROFILE_FILE -ErrorAction SilentlyContinue
Remove-Item Env:\CARGO_INCREMENTAL -ErrorAction SilentlyContinue
```

#### Step 5b: Generate HTML Report

Next, we process the raw data with `grcov` to create the final HTML report.

```bash
# Generate the HTML report from the .profraw files
// turbo
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./target/coverage/
```

The report will be generated in the `target/coverage` directory. If `grcov` is not installed, it should be installed manually with `cargo install grcov`.

### 6. Integration Tests Execution

Run integration tests to verify component interactions:

```bash
# Run integration tests
cargo test --test integration_tests
cargo test --test integration_system_tests
cargo test --test integration_audio_tests

# Run specific integration test suites
cargo test --test oxide_copilot_tests
cargo test --test oxide_core_tests
cargo test --test oxide_guardian_tests
cargo test --test oxide_memory_tests
cargo test --test oxide_rpa_tests
cargo test --test oxide_voice_tests
```

### 7. Performance Tests

Execute performance benchmarks to ensure system meets requirements:

```bash
# Run performance tests
cargo test --test performance_tests --release

# Run benchmarks if available
cargo bench
```

### 8. Frontend Tests (if applicable)

Run frontend-specific tests:

```bash
cd src-frontend
# Run Jest/Vitest tests if configured
npm test

# Run E2E tests if configured
npm run test:e2e
cd ..
```

### 9. Build Verification

Ensure the project builds successfully in both debug and release modes:

```bash
# Build in debug mode
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Build Tauri application
cargo tauri build
```

### 10. Error Analysis and Automatic Fixes

For each test failure or build error encountered:

**a. Analyze the Error Type:**

- Compilation errors (syntax, type mismatches, missing imports)
- Test failures (assertion failures, logic errors)
- Dependency conflicts
- Configuration issues
- Performance regressions

**b. Apply Automatic Fixes:**

**For Compilation Errors:**

- Add missing imports automatically
- Fix common syntax errors
- Update deprecated API usage
- Resolve type mismatches where obvious

**For Test Failures:**

- Update test expectations if the change is intentional
- Fix obvious logic errors in test code
- Update mock data to match new schemas
- Regenerate test fixtures if needed

**For Dependency Issues:**

- Update Cargo.toml versions to resolve conflicts
- Add missing dependencies
- Remove unused dependencies

**For Performance Regressions:**

- Identify bottlenecks using profiling
- Suggest optimizations
- Update performance thresholds if justified

### 11. Security Scanning

Run security vulnerability scans:

```bash
# Scan Rust dependencies for vulnerabilities
cargo audit

# Scan frontend dependencies
cd src-frontend
npm audit
cd ..

# Run additional security tools if available
cargo clippy -- -W clippy::all -W clippy::pedantic
```

### 12. Documentation Updates

Ensure documentation is up to date:

```bash
# Generate and check documentation
cargo doc --workspace --no-deps

# Check for broken documentation links
cargo doc --workspace --no-deps --open
```

### 13. Final Verification and Reporting

**a. Run Complete Test Suite One More Time:**

```bash
# Final comprehensive test run
cargo test --workspace --all-features --release
```

**b. Generate Test Report:**

Create a comprehensive report including:

- Test coverage percentage
- Performance benchmark results
- Security scan results
- Build status for all targets
- List of fixes applied automatically
- Remaining issues requiring manual attention

**c. Git Status Check:**

```bash
# Check if any files were modified during fixes
git status
git diff --name-only
```

### 14. Conditional Actions Based on Results

**If All Tests Pass:**

- Display success message with test statistics
- Optionally commit automatic fixes with descriptive message
- Suggest next steps (merge, deploy, etc.)

**If Tests Fail After Fixes:**

- Display detailed error report
- List specific failing tests with error messages
- Provide suggestions for manual fixes
- Create GitHub issues for complex problems (if configured)
- Prevent commit/merge until issues are resolved

**If Critical Security Issues Found:**

- Block deployment immediately
- Create high-priority alerts
- Provide remediation steps
- Update security documentation

---

## Configuration Options

The workflow can be customized with these environment variables:

- `OXIDE_TEST_COVERAGE_THRESHOLD`: Minimum test coverage required (default: 80%)
- `OXIDE_PERFORMANCE_TIMEOUT`: Maximum time for performance tests (default: 300s)
- `OXIDE_AUTO_FIX_ENABLED`: Enable/disable automatic fixes (default: true)
- `OXIDE_SECURITY_SCAN_ENABLED`: Enable/disable security scanning (default: true)
- `OXIDE_GENERATE_REPORTS`: Generate detailed HTML reports (default: true)

## Integration with CI/CD

This workflow integrates seamlessly with:

- GitHub Actions
- GitLab CI
- Azure DevOps
- Jenkins
- Local pre-commit hooks

## Troubleshooting

**Common Issues and Solutions:**

1. **Memory Issues During Tests:**
   - Reduce parallel test execution: `cargo test -- --test-threads=1`
   - Increase system memory allocation

2. **Timeout Issues:**
   - Increase test timeout values
   - Run tests in release mode for better performance

3. **Dependency Conflicts:**
   - Clear cargo cache: `cargo clean`
   - Update all dependencies: `cargo update`

4. **Permission Issues:**
   - Ensure proper file permissions
   - Run with appropriate privileges if needed

---

## Success Metrics

This workflow ensures:

- ✅ 100% compilation success
- ✅ Minimum 80% test coverage
- ✅ Zero critical security vulnerabilities
- ✅ Performance within defined thresholds
- ✅ Code style compliance

- ✅ Documentation completeness

---

*This workflow follows Oxide Pilot project rules and integrates with the existing testing infrastructure. It can be invoked using `/run-tests-and-fix` in Windsurf Cascade.*
