# Build Oxide Pilot Release
# This script builds a release version of Oxide Pilot with all optimizations

param(
    [string]$Target = "",
    [switch]$Sign,
    [string]$SigningKey = "",
    [switch]$Bundle,
    [switch]$Clean
)

Write-Host "🚀 Building Oxide Pilot Release..." -ForegroundColor Cyan

# Clean previous builds if requested
if ($Clean) {
    Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
    if (Test-Path "src-tauri/target") {
        Remove-Item -Recurse -Force "src-tauri/target"
    }
    if (Test-Path "src-frontend/dist") {
        Remove-Item -Recurse -Force "src-frontend/dist"
    }
}

# Build frontend
Write-Host "📦 Building frontend..." -ForegroundColor Green
Set-Location "src-frontend"

if (-not (Test-Path "node_modules")) {
    Write-Host "📥 Installing frontend dependencies..." -ForegroundColor Blue
    npm ci
}

npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Frontend build failed" -ForegroundColor Red
    exit 1
}

Set-Location ".."

# Build backend
Write-Host "⚙️  Building Rust backend..." -ForegroundColor Green
Set-Location "src-tauri"

# Set environment variables for signing if provided
if ($Sign -and $SigningKey) {
    if (Test-Path $SigningKey) {
        $env:TAURI_PRIVATE_KEY = Get-Content $SigningKey -Raw
        Write-Host "🔐 Signing key loaded" -ForegroundColor Blue
    } else {
        Write-Host "⚠️  Signing key not found: $SigningKey" -ForegroundColor Yellow
    }
}

# Build command
$BuildArgs = @("build", "--release")

if ($Target) {
    $BuildArgs += "--target", $Target
    Write-Host "🎯 Building for target: $Target" -ForegroundColor Blue
}

if ($Bundle) {
    Write-Host "📦 Creating bundles..." -ForegroundColor Blue
    tauri build @BuildArgs
} else {
    Write-Host "🔨 Building binary only..." -ForegroundColor Blue
    cargo build @BuildArgs
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Backend build failed" -ForegroundColor Red
    exit 1
}

Set-Location ".."

# Display build results
Write-Host ""
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host ""

if ($Bundle) {
    Write-Host "📦 Bundle artifacts:" -ForegroundColor Cyan
    $BundleDir = "src-tauri/target/release/bundle"
    if (Test-Path $BundleDir) {
        Get-ChildItem -Recurse $BundleDir -Include "*.exe", "*.msi", "*.dmg", "*.deb", "*.AppImage" | ForEach-Object {
            $Size = [math]::Round($_.Length / 1MB, 2)
            Write-Host "  📄 $($_.Name) ($Size MB)" -ForegroundColor White
        }
    }
} else {
    Write-Host "🔨 Binary artifacts:" -ForegroundColor Cyan
    $BinaryDir = "src-tauri/target/release"
    if (Test-Path "$BinaryDir/oxide-pilot.exe") {
        $Binary = Get-Item "$BinaryDir/oxide-pilot.exe"
        $Size = [math]::Round($Binary.Length / 1MB, 2)
        Write-Host "  📄 oxide-pilot.exe ($Size MB)" -ForegroundColor White
    } elseif (Test-Path "$BinaryDir/oxide-pilot") {
        $Binary = Get-Item "$BinaryDir/oxide-pilot"
        $Size = [math]::Round($Binary.Length / 1MB, 2)
        Write-Host "  📄 oxide-pilot ($Size MB)" -ForegroundColor White
    }
}

Write-Host ""
Write-Host "🎉 Ready for distribution!" -ForegroundColor Green