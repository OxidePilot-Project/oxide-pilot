param(
  [switch]$UseCognee,
  [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

Write-Host "== Oxide Pilot Windows Build ==" -ForegroundColor Cyan

# 1) Check prerequisites
function Ensure-Tool($name, $cmd, $help) {
  if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
    Write-Error "Falta '$name'. $help"
  }
}

Ensure-Tool "Rust (cargo)" "cargo" "Instala Rust desde https://rustup.rs/"
Ensure-Tool "Tauri CLI" "cargo-tauri" "Instala con: cargo install tauri-cli"

# For MSI/NSIS targets
if ($env:TAURI_BUNDLE_TARGETS -match "msi") {
  Ensure-Tool "WiX Toolset (candle/light)" "candle.exe" "Instala WiX Toolset 3.11+ y agrega 'candle' y 'light' al PATH"
}
if ($env:TAURI_BUNDLE_TARGETS -match "nsis") {
  Ensure-Tool "NSIS (makensis)" "makensis.exe" "Instala NSIS y agrega 'makensis' al PATH"
}

# 2) Optionally enable Cognee via env file
if ($UseCognee) {
  $envPath = Join-Path "src-tauri" ".env"
  if (-not (Test-Path $envPath)) {
    Copy-Item (Join-Path "src-tauri" ".env.example") $envPath -Force
  }
  (Get-Content $envPath) | ForEach-Object {
    $_ -replace "^OXIDE_COGNEE_ENABLE=.*$","OXIDE_COGNEE_ENABLE=true"
  } | Set-Content $envPath
}

# 3) Branding: generate icons if base icon exists
$iconBase = Join-Path "src-tauri" "icon.png"
if (Test-Path $iconBase) {
  Write-Host "Generando iconos desde src-tauri/icon.png ..." -ForegroundColor Yellow
  if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Warning "Python no encontrado. Omite generación de iconos."
  } else {
    Push-Location "src-tauri"
    try {
      python create_icon.py
    } catch {
      Write-Warning "Fallo al generar iconos: $_"
    } finally {
      Pop-Location
    }
  }
} else {
  Write-Host "No se encontró src-tauri/icon.png. Usando iconos existentes (si los hay)." -ForegroundColor DarkYellow
}

# 4) Build frontend (if present)
$frontendDir = Join-Path "." "src-frontend"
if (Test-Path (Join-Path $frontendDir "package.json")) {
  Ensure-Tool "Node.js (npm)" "npm" "Instala Node.js desde https://nodejs.org/"
  Write-Host "Construyendo frontend..." -ForegroundColor Yellow
  npm --prefix $frontendDir ci
  npm --prefix $frontendDir run build
}

# 5) Build release + bundle
Push-Location "src-tauri"
try {
  $env:TAURI_PRIVATE_TESTING_VERSION = $Version
  cargo tauri build
} finally {
  Pop-Location
}

# 5) Optional code signing (EXE/MSI) if env vars are provided
# Requires: $env:SIGNTOOL (path to signtool.exe), $env:SIGN_CERT (PFX path), $env:SIGN_PASS (PFX password)
# Optional timestamp server: $env:SIGN_TS_URL (e.g., http://timestamp.digicert.com)
if ($env:SIGNTOOL -and (Test-Path $env:SIGNTOOL) -and (Test-Path $env:SIGN_CERT)) {
  Write-Host "Firmando artefactos..." -ForegroundColor Yellow
  $bundleDir = Join-Path "src-tauri/target/release" "bundle"
  if (Test-Path $bundleDir) {
    Get-ChildItem -Path $bundleDir -Recurse -Include *.exe,*.msi | ForEach-Object {
      $ts = $env:SIGN_TS_URL
      if ($ts) {
        & $env:SIGNTOOL sign /f $env:SIGN_CERT /p $env:SIGN_PASS /tr $ts /td SHA256 /fd SHA256 $_.FullName
      } else {
        & $env:SIGNTOOL sign /f $env:SIGN_CERT /p $env:SIGN_PASS /fd SHA256 $_.FullName
      }
    }
  }
}

Write-Host "Build finalizado. Artefactos en src-tauri/target/release/" -ForegroundColor Green
