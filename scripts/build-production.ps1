# Production Build Script for Oxide Pilot
# This script builds the application with all optimizations enabled

Write-Host "🚀 Starting Oxide Pilot Production Build..." -ForegroundColor Cyan
Write-Host ""

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ Error: Must run from project root" -ForegroundColor Red
    exit 1
}

# Step 1: Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "target") {
    Remove-Item -Path "target/release" -Recurse -Force -ErrorAction SilentlyContinue
}
if (Test-Path "src-frontend/dist") {
    Remove-Item -Path "src-frontend/dist" -Recurse -Force -ErrorAction SilentlyContinue
}
Write-Host "✅ Clean complete" -ForegroundColor Green
Write-Host ""

# Step 2: Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Yellow
$testResults = @()

Write-Host "  Testing oxide-rpa..." -ForegroundColor Gray
cargo test -p oxide-rpa --quiet
if ($LASTEXITCODE -eq 0) {
    $testResults += "✅ oxide-rpa: PASS"
} else {
    $testResults += "❌ oxide-rpa: FAIL"
}

Write-Host "  Testing oxide-memory..." -ForegroundColor Gray
cargo test -p oxide-memory --features surrealdb --lib --quiet
if ($LASTEXITCODE -eq 0) {
    $testResults += "✅ oxide-memory: PASS"
} else {
    $testResults += "❌ oxide-memory: FAIL"
}

Write-Host "  Testing oxide-guardian..." -ForegroundColor Gray
cargo test -p oxide-guardian --features surrealdb-metrics --quiet
if ($LASTEXITCODE -eq 0) {
    $testResults += "✅ oxide-guardian: PASS"
} else {
    $testResults += "❌ oxide-guardian: FAIL"
}

Write-Host ""
Write-Host "Test Results:" -ForegroundColor Cyan
foreach ($result in $testResults) {
    Write-Host "  $result"
}
Write-Host ""

# Check if any tests failed
if ($testResults -match "FAIL") {
    Write-Host "❌ Tests failed. Aborting build." -ForegroundColor Red
    exit 1
}

# Step 3: Lint with Clippy
Write-Host "🔍 Running Clippy..." -ForegroundColor Yellow
cargo clippy --manifest-path src-tauri/Cargo.toml --features surrealdb-metrics --release -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy found issues. Aborting build." -ForegroundColor Red
    exit 1
}
Write-Host "✅ Clippy passed" -ForegroundColor Green
Write-Host ""

# Step 4: Build frontend
Write-Host "🎨 Building frontend..." -ForegroundColor Yellow
Set-Location src-frontend
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Frontend build failed" -ForegroundColor Red
    Set-Location ..
    exit 1
}
Set-Location ..
Write-Host "✅ Frontend built" -ForegroundColor Green
Write-Host ""

# Step 5: Build backend (release mode)
Write-Host "⚙️  Building backend (release mode)..." -ForegroundColor Yellow
Write-Host "  This may take several minutes..." -ForegroundColor Gray
cargo build --manifest-path src-tauri/Cargo.toml --release --features surrealdb-metrics
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Backend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Backend built" -ForegroundColor Green
Write-Host ""

# Step 6: Get binary size
$binaryPath = "src-tauri/target/release/oxide-pilot.exe"
if (Test-Path $binaryPath) {
    $size = (Get-Item $binaryPath).Length / 1MB
    Write-Host "📦 Binary size: $([math]::Round($size, 2)) MB" -ForegroundColor Cyan
}
Write-Host ""

# Step 7: Summary
Write-Host "🎉 Production build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Build artifacts:" -ForegroundColor Cyan
Write-Host "  Backend: src-tauri/target/release/oxide-pilot.exe" -ForegroundColor Gray
Write-Host "  Frontend: src-frontend/dist/" -ForegroundColor Gray
Write-Host ""
Write-Host "To run the production build:" -ForegroundColor Yellow
Write-Host "  .\src-tauri\target\release\oxide-pilot.exe" -ForegroundColor White
Write-Host ""
