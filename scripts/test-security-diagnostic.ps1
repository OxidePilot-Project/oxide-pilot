# Test script for Security Diagnostic integration
# This script verifies that the security diagnostic is properly integrated

Write-Host "=== Testing Security Diagnostic Integration ===" -ForegroundColor Cyan
Write-Host ""

# 1. Check if the Rust module exists
Write-Host "[1/5] Checking Rust module..." -ForegroundColor Green
$rustModulePath = "src-tauri\src\security_diagnostic.rs"
if (Test-Path $rustModulePath) {
    Write-Host "  ✓ Security diagnostic Rust module found" -ForegroundColor Green
} else {
    Write-Host "  ✗ Security diagnostic Rust module NOT found" -ForegroundColor Red
    exit 1
}

# 2. Check if the module is imported in main.rs
Write-Host "[2/5] Checking main.rs integration..." -ForegroundColor Green
$mainRsContent = Get-Content "src-tauri\src\main.rs" -Raw
if ($mainRsContent -match "mod security_diagnostic;") {
    Write-Host "  ✓ Module declared in main.rs" -ForegroundColor Green
} else {
    Write-Host "  ✗ Module NOT declared in main.rs" -ForegroundColor Red
    exit 1
}

# 3. Check if Tauri commands are registered
Write-Host "[3/5] Checking Tauri command registration..." -ForegroundColor Green
if ($mainRsContent -match "security_diagnostic::run_security_diagnostic") {
    Write-Host "  ✓ run_security_diagnostic command registered" -ForegroundColor Green
} else {
    Write-Host "  ✗ run_security_diagnostic NOT registered" -ForegroundColor Red
    exit 1
}

if ($mainRsContent -match "security_diagnostic::get_last_security_scan") {
    Write-Host "  ✓ get_last_security_scan command registered" -ForegroundColor Green
} else {
    Write-Host "  ✗ get_last_security_scan NOT registered" -ForegroundColor Red
    exit 1
}

if ($mainRsContent -match "security_diagnostic::get_system_health") {
    Write-Host "  ✓ get_system_health command registered" -ForegroundColor Green
} else {
    Write-Host "  ✗ get_system_health NOT registered" -ForegroundColor Red
    exit 1
}

# 4. Check if frontend component exists
Write-Host "[4/5] Checking frontend component..." -ForegroundColor Green
$frontendComponentPath = "src-frontend\src\lib\components\SecurityDiagnostic.svelte"
if (Test-Path $frontendComponentPath) {
    Write-Host "  ✓ SecurityDiagnostic.svelte component found" -ForegroundColor Green
} else {
    Write-Host "  ✗ SecurityDiagnostic.svelte component NOT found" -ForegroundColor Red
    exit 1
}

# 5. Check if component is integrated in SecurityCenter
Write-Host "[5/5] Checking SecurityCenter integration..." -ForegroundColor Green
$securityCenterContent = Get-Content "src-frontend\src\lib\components\SecurityCenter.svelte" -Raw
if ($securityCenterContent -match "import SecurityDiagnostic") {
    Write-Host "  ✓ SecurityDiagnostic imported in SecurityCenter" -ForegroundColor Green
} else {
    Write-Host "  ✗ SecurityDiagnostic NOT imported in SecurityCenter" -ForegroundColor Red
    exit 1
}

if ($securityCenterContent -match "<SecurityDiagnostic") {
    Write-Host "  ✓ SecurityDiagnostic component used in SecurityCenter" -ForegroundColor Green
} else {
    Write-Host "  ✗ SecurityDiagnostic component NOT used in SecurityCenter" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== ALL TESTS PASSED ===" -ForegroundColor Green
Write-Host ""
Write-Host "Security Diagnostic is fully integrated!" -ForegroundColor Cyan
Write-Host ""
Write-Host "Available commands:" -ForegroundColor Yellow
Write-Host "  - run_security_diagnostic: Run a complete security scan" -ForegroundColor White
Write-Host "  - get_last_security_scan: Get the last scan report" -ForegroundColor White
Write-Host "  - get_system_health: Get current system health metrics" -ForegroundColor White
Write-Host ""
Write-Host "Frontend component:" -ForegroundColor Yellow
Write-Host "  - SecurityDiagnostic.svelte: UI for security scanning" -ForegroundColor White
Write-Host "  - Integrated in SecurityCenter.svelte" -ForegroundColor White
Write-Host ""
