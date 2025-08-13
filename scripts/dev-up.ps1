param(
  [switch]$UseCognee,
  [switch]$StartSidecar
)

$ErrorActionPreference = "Stop"

Write-Host "== Oxide Pilot Dev Up ==" -ForegroundColor Cyan

# 1) Ensure tools
function Ensure-Tool($name, $cmd, $help) {
  if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
    Write-Error "Falta '$name'. $help"
  }
}

Ensure-Tool "Rust (cargo)" "cargo" "Instala Rust desde https://rustup.rs/"
Ensure-Tool "Tauri CLI" "cargo-tauri" "Instala con: cargo install tauri-cli"

# 2) Prepare .env for Tauri
$envPath = Join-Path "src-tauri" ".env"
if (-not (Test-Path $envPath)) { Copy-Item (Join-Path "src-tauri" ".env.example") $envPath -Force }
if ($UseCognee) {
  (Get-Content $envPath) | ForEach-Object { $_ -replace "^OXIDE_COGNEE_ENABLE=.*$","OXIDE_COGNEE_ENABLE=true" } | Set-Content $envPath
}

# 3) Optionally start sidecar
if ($StartSidecar) {
  if ($UseCognee -eq $false) { Write-Warning "-StartSidecar usado sin -UseCognee. Activando OXIDE_COGNEE_ENABLE=true"; $UseCognee = $true }
  pwsh -File scripts/setup-cognee-sidecar.ps1 -Run
}

# 4) Build frontend once for static dev
$frontendDir = Join-Path "." "src-frontend"
if (Test-Path (Join-Path $frontendDir "package.json")) {
  Ensure-Tool "Node.js (npm)" "npm" "Instala Node.js desde https://nodejs.org/"
  npm --prefix $frontendDir ci
  npm --prefix $frontendDir run build
}

# 5) Run Tauri dev (servirá ../src-frontend/dist)
Push-Location "src-tauri"
try {
  cargo tauri dev
} finally {
  Pop-Location
}
