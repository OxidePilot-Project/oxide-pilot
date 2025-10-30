param(
  [ValidateSet("move","delete","none")]
  [string]$ProfrawAction = "move",
  [string]$ProfrawDir = "dev-artifacts/coverage",
  [switch]$UseCognee,
  [switch]$StartSidecar
)

$ErrorActionPreference = "Stop"

Write-Host "== Oxide Pilot Dev Launcher ==" -ForegroundColor Cyan

function Ensure-Tool($name, $cmd, $help) {
  if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
    Write-Error "Missing '$name'. $help"
  }
}

function Manage-Profraw($phase) {
  Write-Host "[$phase] Handling .profraw artifacts ($ProfrawAction) ..." -ForegroundColor DarkCyan
  if ($ProfrawAction -eq 'none') { return }

  $root = (Get-Location)
  $profDirPath = Join-Path $root $ProfrawDir
  if ($ProfrawAction -eq 'move') {
    if (-not (Test-Path $profDirPath)) {
      New-Item -ItemType Directory -Force -Path $profDirPath | Out-Null
    }
  }

  $files = Get-ChildItem -Path $root -Recurse -Include *.profraw -File -ErrorAction SilentlyContinue
  foreach ($f in $files) {
    # Skip if already inside ProfrawDir
    if ($ProfrawAction -eq 'move' -and ($f.FullName -like (Join-Path $profDirPath '*'))) { continue }

    switch ($ProfrawAction) {
      'move' {
        $dest = Join-Path $profDirPath $f.Name
        if (Test-Path $dest) {
          $ts = Get-Date -Format 'yyyyMMdd_HHmmss_ffff'
          $dest = Join-Path $profDirPath ("$($f.BaseName)_$ts.profraw")
        }
        Move-Item -Force -Path $f.FullName -Destination $dest
      }
      'delete' {
        Remove-Item -Force -Path $f.FullName
      }
    }
  }
}

# Move to repo root (script resides in scripts/)
Push-Location (Join-Path $PSScriptRoot '..')
try {
  # 1) Ensure base tools
  Ensure-Tool "Rust (cargo)" "cargo" "Install Rust from https://rustup.rs/"
  Ensure-Tool "Tauri CLI" "cargo-tauri" "Install via: cargo install tauri-cli"

  # 2) Prepare Tauri .env
  $envPath = Join-Path "src-tauri" ".env"
  if (-not (Test-Path $envPath)) {
    $example = Join-Path "src-tauri" ".env.example"
    if (Test-Path $example) { Copy-Item $example $envPath -Force }
  }
  if ($UseCognee) {
    if (Test-Path $envPath) {
      (Get-Content $envPath) | ForEach-Object { $_ -replace "^OXIDE_COGNEE_ENABLE=.*$","OXIDE_COGNEE_ENABLE=true" } | Set-Content $envPath
    }
  }

  # 3) Optionally start Cognee sidecar
  if ($StartSidecar) {
    if (-not $UseCognee) {
      Write-Warning "-StartSidecar used without -UseCognee. Enabling OXIDE_COGNEE_ENABLE=true"
      $UseCognee = $true
      if (Test-Path $envPath) {
        (Get-Content $envPath) | ForEach-Object { $_ -replace "^OXIDE_COGNEE_ENABLE=.*$","OXIDE_COGNEE_ENABLE=true" } | Set-Content $envPath
      }
    }
    pwsh -File (Join-Path "scripts" "setup-cognee-sidecar.ps1") -Run
  }

  # 4) Pre-clean .profraw
  Manage-Profraw "pre"

  # 5) Frontend build (static)
  $frontendDir = Join-Path "." "src-frontend"
  if (Test-Path (Join-Path $frontendDir "package.json")) {
    Ensure-Tool "Node.js (npm)" "npm" "Install Node.js from https://nodejs.org/"
    npm --prefix $frontendDir ci
    npm --prefix $frontendDir run build
  } else {
    Write-Host "No frontend found at src-frontend/. Skipping build." -ForegroundColor DarkYellow
  }

  # 6) Launch Tauri dev
  Push-Location "src-tauri"
  try {
    if (-not $env:OXIDE_SURREAL_ENABLE -and -not $env:OXIDE_SURREAL_DISABLE) {
      $env:OXIDE_SURREAL_ENABLE = "true"
    }
    cargo tauri dev --features surrealdb-metrics
  } finally {
    Pop-Location
  }
} finally {
  # 7) Post-clean .profraw
  Manage-Profraw "post"
  Pop-Location
}
