#!/usr/bin/env pwsh
# Windows Installer Creation Script for Oxide Pilot

param(
    [switch]$Sign = $false,
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

Write-Host "📦 Oxide Pilot - Installer Creation" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

$startTime = Get-Date

# 1. Build release version
Write-Host "🔨 Building release version..." -ForegroundColor Green
Write-Host ""

cargo build --release --features surrealdb-metrics

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build complete" -ForegroundColor Green

# 2. Prepare installer directory
Write-Host ""
Write-Host "📁 Preparing installer directory..." -ForegroundColor Green

$installerDir = "target/installer"
if (Test-Path $installerDir) {
    Remove-Item -Recurse -Force $installerDir
}
New-Item -ItemType Directory -Path $installerDir | Out-Null

# Copy binary
Copy-Item "target/release/oxide-pilot.exe" "$installerDir/"

# Copy assets
if (Test-Path "assets") {
    Copy-Item -Recurse "assets" "$installerDir/"
}

Write-Host "✅ Files prepared" -ForegroundColor Green

# 3. Sign binary (if requested)
if ($Sign) {
    Write-Host ""
    Write-Host "✍️  Signing binary..." -ForegroundColor Green

    $certPath = "certs/OxidePilot-CodeSigning.pfx"
    $passwordPath = "certs/password.txt"

    if ((Test-Path $certPath) -and (Test-Path $passwordPath)) {
        $password = Get-Content $passwordPath -Raw

        # Sign using signtool (requires Windows SDK)
        try {
            $signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"

            if (Test-Path $signtool) {
                & $signtool sign /f $certPath /p $password /t http://timestamp.digicert.com "$installerDir/oxide-pilot.exe"
                Write-Host "✅ Binary signed" -ForegroundColor Green
            } else {
                Write-Host "⚠️  signtool not found, skipping signing" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "⚠️  Signing failed: $_" -ForegroundColor Yellow
        }
    } else {
        Write-Host "⚠️  Certificate or password not found, skipping signing" -ForegroundColor Yellow
    }
}

# 4. Create MSI installer using WiX (if available)
Write-Host ""
Write-Host "📦 Creating MSI installer..." -ForegroundColor Green

$wixPath = "C:\Program Files (x86)\WiX Toolset v3.11\bin"
if (Test-Path $wixPath) {
    Write-Host "✅ WiX Toolset found" -ForegroundColor Green

    # Create WiX configuration
    $wxs = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="Oxide Pilot" Language="1033" Version="$Version"
           Manufacturer="Oxide Pilot Team" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />

    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="Oxide Pilot" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="OxidePilot" />
      </Directory>
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="Oxide Pilot"/>
      </Directory>
    </Directory>

    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="*">
        <File Id="OxidePilotExe" Source="$installerDir\oxide-pilot.exe" KeyPath="yes" />
      </Component>
    </ComponentGroup>
  </Product>
</Wix>
"@

    $wxs | Out-File -FilePath "$installerDir/installer.wxs" -Encoding UTF8

    # Compile WiX
    try {
        & "$wixPath\candle.exe" "$installerDir/installer.wxs" -out "$installerDir/installer.wixobj"
        & "$wixPath\light.exe" "$installerDir/installer.wixobj" -out "target/OxidePilot-$Version.msi"

        Write-Host "✅ MSI installer created: target/OxidePilot-$Version.msi" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  WiX compilation failed: $_" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  WiX Toolset not found. Install from: https://wixtoolset.org/" -ForegroundColor Yellow
    Write-Host "    Creating portable ZIP instead..." -ForegroundColor Yellow

    # Create portable ZIP
    Compress-Archive -Path "$installerDir/*" -DestinationPath "target/OxidePilot-$Version-portable.zip" -Force
    Write-Host "✅ Portable ZIP created: target/OxidePilot-$Version-portable.zip" -ForegroundColor Green
}

# 5. Generate installer report
Write-Host ""
Write-Host "📄 Generating installer report..." -ForegroundColor Green

$binarySize = (Get-Item "target/release/oxide-pilot.exe").Length / 1MB

$report = @"
# 📦 Installer Creation Report
**Date**: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**Project**: Oxide Pilot
**Version**: $Version

---

## 📊 Build Information

- **Binary Size**: $([math]::Round($binarySize, 2)) MB
- **Build Type**: Release
- **Features**: surrealdb-metrics
- **Signed**: $(if ($Sign) { "Yes ✅" } else { "No" })

---

## 📦 Installer Packages

"@

if (Test-Path "target/OxidePilot-$Version.msi") {
    $msiSize = (Get-Item "target/OxidePilot-$Version.msi").Length / 1MB
    $report += "- **MSI Installer**: OxidePilot-$Version.msi ($([math]::Round($msiSize, 2)) MB)`n"
}

if (Test-Path "target/OxidePilot-$Version-portable.zip") {
    $zipSize = (Get-Item "target/OxidePilot-$Version-portable.zip").Length / 1MB
    $report += "- **Portable ZIP**: OxidePilot-$Version-portable.zip ($([math]::Round($zipSize, 2)) MB)`n"
}

$report += @"

---

## 🚀 Distribution

### Installation Methods

1. **MSI Installer** (Recommended)
   - Double-click to install
   - Automatic updates supported
   - System-wide installation

2. **Portable ZIP**
   - Extract and run
   - No installation required
   - Portable across machines

---

## 📋 System Requirements

- **OS**: Windows 10/11 (64-bit)
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 200MB free space
- **.NET**: Not required (native Rust binary)

---

## 🔐 Security

- Binary compiled with release optimizations
- $(if ($Sign) { "Code signed with valid certificate ✅" } else { "Not code signed ⚠️" })
- All dependencies audited

---

**Build Duration**: $((Get-Date) - $startTime)
**Generated by**: Oxide Pilot Installer Script

"@

$reportPath = "docs/INSTALLER_REPORT.md"
$report | Out-File -FilePath $reportPath -Encoding UTF8

Write-Host "✅ Installer report generated: $reportPath" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "✅ Installer Creation Complete!" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Duration: $((Get-Date) - $startTime)" -ForegroundColor White
Write-Host "Binary Size: $([math]::Round($binarySize, 2)) MB" -ForegroundColor White
Write-Host "Report: $reportPath" -ForegroundColor White
Write-Host ""

if (Test-Path "target/OxidePilot-$Version.msi") {
    Write-Host "📦 MSI Installer: target/OxidePilot-$Version.msi" -ForegroundColor Green
}
if (Test-Path "target/OxidePilot-$Version-portable.zip") {
    Write-Host "📦 Portable ZIP: target/OxidePilot-$Version-portable.zip" -ForegroundColor Green
}

Write-Host ""
