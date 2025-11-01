param(
  [switch]$UseCognee,
  [string]$Version = "0.1.0",
  [string]$OutputDir = "portable-output"
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$tauriConfigPath = Join-Path $projectRoot "src-tauri\tauri.conf.json"
if (-not (Test-Path $tauriConfigPath)) {
  throw "Unable to find tauri.conf.json at $tauriConfigPath"
}

$tauriConfig = Get-Content $tauriConfigPath -Raw | ConvertFrom-Json
$productName = $tauriConfig.package.productName
if (-not $PSBoundParameters.ContainsKey("Version") -and $tauriConfig.package.version) {
  $Version = $tauriConfig.package.version
}

Write-Host "== Oxide Pilot Portable Executable Creator ==" -ForegroundColor Cyan
Write-Host "Version: $Version" -ForegroundColor Gray
Write-Host "Product: $productName" -ForegroundColor Gray
Write-Host ""

Push-Location $projectRoot

try {
  function Ensure-Tool {
    param(
      [Parameter(Mandatory = $true)][string]$Name,
      [Parameter(Mandatory = $true)][string]$Command,
      [string]$Help = ""
    )

    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
      $message = "Missing '$Name'."
      if ($Help) {
        $message += " $Help"
      }
      throw $message
    }
  }

  Write-Host "Checking prerequisites..." -ForegroundColor Yellow

  $sevenZipCandidates = @()
  if ($env:ProgramFiles)    { $sevenZipCandidates += Join-Path $env:ProgramFiles    "7-Zip\7z.exe" }
  if ($env:ProgramFilesx86) { $sevenZipCandidates += Join-Path $env:ProgramFilesx86 "7-Zip\7z.exe" }
  if ($env:LOCALAPPDATA)    { $sevenZipCandidates += Join-Path $env:LOCALAPPDATA    "7-Zip\7z.exe" }

  $sevenZipPath = $sevenZipCandidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

  if (-not $sevenZipPath) {
    throw "7-Zip not found. Please install 7-Zip from https://www.7-zip.org/"
  }

  Write-Host "[OK] 7-Zip found at: $sevenZipPath" -ForegroundColor Green

  Ensure-Tool -Name "Rust (cargo)" -Command "cargo" -Help "Install Rust from https://rustup.rs/"
  Ensure-Tool -Name "Tauri CLI" -Command "cargo-tauri" -Help "Install with: cargo install tauri-cli"

  if ($UseCognee) {
    $envPath = Join-Path "src-tauri" ".env"
    if (-not (Test-Path $envPath)) {
      Copy-Item (Join-Path "src-tauri" ".env.example") $envPath -Force
    }

    (Get-Content $envPath) | ForEach-Object {
      $_ -replace "^OXIDE_COGNEE_ENABLE=.*$", "OXIDE_COGNEE_ENABLE=true"
    } | Set-Content -Path $envPath -Encoding UTF8

    Write-Host "[OK] Cognee memory system enabled" -ForegroundColor Green
  }

  $logoPath = "logo.png"
  if (Test-Path $logoPath) {
    Write-Host "Generating icons from logo.png..." -ForegroundColor Yellow
    if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
      Write-Warning "Python not found. Skipping icon generation."
    } else {
      $iconsDir = Join-Path $projectRoot "src-tauri\icons"
      if (-not (Test-Path $iconsDir)) {
        New-Item -ItemType Directory -Path $iconsDir | Out-Null
      }

      python "src-tauri/create_icon.py"
      if ($LASTEXITCODE -ne 0) {
        Write-Warning "Failed to generate icons: Icon generation script exited with code $LASTEXITCODE."
      } else {
        Write-Host "[OK] Icons generated successfully" -ForegroundColor Green
      }
    }
  } else {
    Write-Host "[WARN] logo.png not found. Using existing icons if available." -ForegroundColor Yellow
  }

  Write-Host "Building frontend..." -ForegroundColor Yellow
  Push-Location "src-frontend"
  try {
    npm ci
    if ($LASTEXITCODE -ne 0) {
      throw "npm ci failed with exit code $LASTEXITCODE."
    }

    npm run build
    if ($LASTEXITCODE -ne 0) {
      throw "npm run build failed with exit code $LASTEXITCODE."
    }

    Write-Host "[OK] Frontend built successfully" -ForegroundColor Green
  } catch {
    throw "Frontend build failed: $_"
  } finally {
    Pop-Location
  }

  Write-Host "Building Tauri app (portable bundle)..." -ForegroundColor Yellow
  Push-Location "src-tauri"
  try {
    $env:TAURI_BUNDLE_TARGETS = "app"
    cargo tauri build --bundles app
    if ($LASTEXITCODE -ne 0) {
      throw "cargo tauri build failed with exit code $LASTEXITCODE."
    }

    Write-Host "[OK] Tauri app built successfully" -ForegroundColor Green
  } catch {
    throw "Tauri build failed: $_"
  } finally {
    Pop-Location
    Remove-Item Env:\TAURI_BUNDLE_TARGETS -ErrorAction SilentlyContinue
  }

  Write-Host "Creating portable executable..." -ForegroundColor Yellow

  $targetReleaseDir = Join-Path $projectRoot "target\release"
  if (-not (Test-Path $targetReleaseDir)) {
    throw "Cargo target directory not found at: $targetReleaseDir"
  }

  $exeName = "$productName.exe"
  $binaryPath = Join-Path $targetReleaseDir $exeName
  if (-not (Test-Path $binaryPath)) {
    throw "Built binary not found at: $binaryPath"
  }

  $resourcesDir = Join-Path $targetReleaseDir "resources"
  $stagingRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("oxide-portable-" + [System.Guid]::NewGuid().ToString("N"))
  New-Item -ItemType Directory -Path $stagingRoot | Out-Null

  try {
    $appDir = Join-Path $stagingRoot $productName
    New-Item -ItemType Directory -Path $appDir | Out-Null

    Copy-Item -Path $binaryPath -Destination $appDir -Force
    if (Test-Path $resourcesDir) {
      Copy-Item -Path $resourcesDir -Destination $appDir -Recurse -Force
    }

    if (-not (Test-Path $OutputDir)) {
      New-Item -ItemType Directory -Path $OutputDir | Out-Null
    }

    $portableName = ($productName -replace '[^A-Za-z0-9]+', '-').Trim('-')
    if (-not $portableName) {
      $portableName = "oxide-pilot"
    }

    $archiveName = "$portableName-Portable-$Version.7z"
    $sfxName = "$portableName-Portable-$Version.exe"
    $archivePath = Join-Path $OutputDir $archiveName
    $sfxPath = Join-Path $OutputDir $sfxName

    Write-Host "Creating 7z archive..." -ForegroundColor Gray
    $sevenZipArgs = @(
      "a",
      "-t7z",
      "-mx=9",
      "-mmt=on",
      "-r",
      $archivePath,
      "$appDir\*"
    )

    & $sevenZipPath $sevenZipArgs
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $archivePath)) {
      throw "Failed to create 7z archive"
    }

    $configContent = @"
;!@Install@!UTF-8!
Title="$productName Portable $Version"
BeginPrompt="Do you want to run $productName Portable?"
RunProgram="\"$productName\$exeName\""
;!@InstallEnd@!
"@

    $configPath = Join-Path $OutputDir "sfx-config.txt"
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($configPath, $configContent, $utf8NoBom)

    $sfxModule = Join-Path (Split-Path $sevenZipPath) "7z.sfx"
    if (-not (Test-Path $sfxModule)) {
      throw "7z.sfx module not found alongside 7-Zip installation."
    }

    Write-Host "Creating self-extracting executable..." -ForegroundColor Gray
    $copyCommand = "copy /b `"$sfxModule`"+`"$configPath`"+`"$archivePath`" `"$sfxPath`" /Y"
    cmd.exe /c $copyCommand | Out-Null

    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $sfxPath)) {
      throw "Failed to create SFX executable"
    }

    Remove-Item $archivePath -ErrorAction SilentlyContinue
    Remove-Item $configPath -ErrorAction SilentlyContinue

    $sfxSize = (Get-Item $sfxPath).Length / 1MB
    $sfxSizeFormatted = "{0:N2} MB" -f $sfxSize

    Write-Host ""
    Write-Host "[DONE] Portable executable created successfully!" -ForegroundColor Green
    Write-Host "Location: $sfxPath" -ForegroundColor Cyan
    Write-Host "Size: $sfxSizeFormatted" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To use:" -ForegroundColor Yellow
    Write-Host "  1. Copy the .exe file to any Windows computer" -ForegroundColor Gray
    Write-Host "  2. Run the .exe file" -ForegroundColor Gray
    Write-Host "  3. The application will extract and run automatically" -ForegroundColor Gray
    Write-Host ""
    Write-Host "The portable version includes:" -ForegroundColor Yellow
    Write-Host "  - No installation required" -ForegroundColor Gray
    Write-Host "  - All dependencies bundled" -ForegroundColor Gray
    Write-Host "  - Can run from any directory" -ForegroundColor Gray
    Write-Host "  - Leaves no traces in Windows registry" -ForegroundColor Gray
  } finally {
    if (Test-Path $stagingRoot) {
      Remove-Item $stagingRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
  }
} finally {
  Pop-Location
}
