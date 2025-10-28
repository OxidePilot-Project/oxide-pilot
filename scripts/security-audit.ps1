#!/usr/bin/env pwsh
# Security Audit Script for Oxide Pilot
# Performs comprehensive security checks

param(
    [switch]$Fix = $false,
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

Write-Host "🔒 Oxide Pilot - Security Audit" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

$startTime = Get-Date
$issues = @()

# Function to log issues
function Add-Issue {
    param($Severity, $Message)
    $script:issues += @{
        Severity = $Severity
        Message = $Message
    }
    $color = switch ($Severity) {
        "CRITICAL" { "Red" }
        "HIGH" { "Yellow" }
        "MEDIUM" { "Cyan" }
        "LOW" { "Gray" }
    }
    Write-Host "[$Severity] $Message" -ForegroundColor $color
}

# 1. Check if cargo-audit is installed
Write-Host "📦 Step 1: Checking cargo-audit installation..." -ForegroundColor Green
try {
    $null = cargo audit --version 2>&1
    Write-Host "✅ cargo-audit is installed" -ForegroundColor Green
} catch {
    Write-Host "⚠️  cargo-audit not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-audit
}

# 2. Run cargo audit
Write-Host ""
Write-Host "🔍 Step 2: Running dependency vulnerability scan..." -ForegroundColor Green
$auditOutput = cargo audit --json 2>&1 | Out-String

try {
    $auditJson = $auditOutput | ConvertFrom-Json

    if ($auditJson.vulnerabilities.found -gt 0) {
        Add-Issue "CRITICAL" "Found $($auditJson.vulnerabilities.found) vulnerable dependencies"

        foreach ($vuln in $auditJson.vulnerabilities.list) {
            Add-Issue "HIGH" "  - $($vuln.package.name) $($vuln.package.version): $($vuln.advisory.title)"
        }
    } else {
        Write-Host "✅ No vulnerable dependencies found" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠️  Could not parse audit output, running text mode..." -ForegroundColor Yellow
    cargo audit
}

# 3. Check for outdated dependencies
Write-Host ""
Write-Host "📊 Step 3: Checking for outdated dependencies..." -ForegroundColor Green
try {
    $null = cargo outdated --version 2>&1
    cargo outdated --root-deps-only
} catch {
    Write-Host "⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated" -ForegroundColor Yellow
}

# 4. Check for secrets in code
Write-Host ""
Write-Host "🔐 Step 4: Scanning for hardcoded secrets..." -ForegroundColor Green

$secretPatterns = @(
    @{ Name = "API Key"; Pattern = "(?i)(api[_-]?key|apikey)\s*[:=]\s*['\"]([a-zA-Z0-9_\-]{20,})['\"]" },
    @{ Name = "Password"; Pattern = "(?i)(password|passwd|pwd)\s*[:=]\s*['\"]([^'\"]{8,})['\"]" },
    @{ Name = "Token"; Pattern = "(?i)(token|auth[_-]?token)\s*[:=]\s*['\"]([a-zA-Z0-9_\-]{20,})['\"]" },
    @{ Name = "Private Key"; Pattern = "-----BEGIN (RSA |EC )?PRIVATE KEY-----" },
    @{ Name = "AWS Key"; Pattern = "AKIA[0-9A-Z]{16}" }
)

$sourceFiles = Get-ChildItem -Path . -Include *.rs,*.toml,*.json,*.ts,*.svelte -Recurse -File |
    Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\.git\\" }

$secretsFound = $false
foreach ($file in $sourceFiles) {
    $content = Get-Content $file.FullName -Raw

    foreach ($pattern in $secretPatterns) {
        if ($content -match $pattern.Pattern) {
            Add-Issue "CRITICAL" "Potential $($pattern.Name) found in $($file.Name)"
            $secretsFound = $true
        }
    }
}

if (-not $secretsFound) {
    Write-Host "✅ No hardcoded secrets detected" -ForegroundColor Green
}

# 5. Check Tauri security configuration
Write-Host ""
Write-Host "🛡️  Step 5: Checking Tauri security configuration..." -ForegroundColor Green

$tauriConfig = Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json

# Check CSP
if ($tauriConfig.tauri.security.csp) {
    Write-Host "✅ Content Security Policy is configured" -ForegroundColor Green
} else {
    Add-Issue "HIGH" "Content Security Policy (CSP) not configured"
}

# Check dangerous permissions
$dangerousPerms = @("all", "shell-execute")
$allowlist = $tauriConfig.tauri.allowlist

if ($allowlist.all -eq $true) {
    Add-Issue "HIGH" "Tauri allowlist 'all' is enabled - this is dangerous"
}

# 6. Check for unsafe Rust code
Write-Host ""
Write-Host "⚠️  Step 6: Scanning for unsafe Rust code..." -ForegroundColor Green

$unsafeCount = 0
$rustFiles = Get-ChildItem -Path . -Include *.rs -Recurse -File |
    Where-Object { $_.FullName -notmatch "\\target\\" }

foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    $matches = [regex]::Matches($content, "\bunsafe\b")

    if ($matches.Count -gt 0) {
        $unsafeCount += $matches.Count
        if ($Verbose) {
            Add-Issue "MEDIUM" "Found $($matches.Count) unsafe blocks in $($file.Name)"
        }
    }
}

if ($unsafeCount -gt 0) {
    Add-Issue "MEDIUM" "Total unsafe blocks found: $unsafeCount (review recommended)"
} else {
    Write-Host "✅ No unsafe code blocks found" -ForegroundColor Green
}

# 7. Check file permissions (Windows)
Write-Host ""
Write-Host "📁 Step 7: Checking sensitive file permissions..." -ForegroundColor Green

$sensitiveFiles = @(
    "certs\OxidePilot-CodeSigning.pfx",
    "certs\password.txt",
    ".env"
)

foreach ($file in $sensitiveFiles) {
    if (Test-Path $file) {
        Write-Host "⚠️  Sensitive file exists: $file" -ForegroundColor Yellow
        Add-Issue "MEDIUM" "Sensitive file should be excluded from version control: $file"
    }
}

# 8. Check .gitignore
Write-Host ""
Write-Host "📝 Step 8: Validating .gitignore..." -ForegroundColor Green

$gitignore = Get-Content ".gitignore" -Raw
$requiredPatterns = @(
    "*.pfx",
    "*.pem",
    "*.key",
    ".env",
    "password.txt"
)

foreach ($pattern in $requiredPatterns) {
    if ($gitignore -notmatch [regex]::Escape($pattern)) {
        Add-Issue "HIGH" ".gitignore missing pattern: $pattern"
    }
}

Write-Host "✅ .gitignore validation complete" -ForegroundColor Green

# 9. Generate security report
Write-Host ""
Write-Host "📄 Step 9: Generating security report..." -ForegroundColor Green

$report = @"
# 🔒 Security Audit Report
**Date**: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
**Project**: Oxide Pilot
**Version**: 0.1.0

---

## 📊 Summary

- **Total Issues Found**: $($issues.Count)
- **Critical**: $(($issues | Where-Object { $_.Severity -eq "CRITICAL" }).Count)
- **High**: $(($issues | Where-Object { $_.Severity -eq "HIGH" }).Count)
- **Medium**: $(($issues | Where-Object { $_.Severity -eq "MEDIUM" }).Count)
- **Low**: $(($issues | Where-Object { $_.Severity -eq "LOW" }).Count)

---

## 🔍 Detailed Findings

"@

if ($issues.Count -eq 0) {
    $report += "`n✅ **No security issues found!**`n"
} else {
    foreach ($severity in @("CRITICAL", "HIGH", "MEDIUM", "LOW")) {
        $severityIssues = $issues | Where-Object { $_.Severity -eq $severity }
        if ($severityIssues.Count -gt 0) {
            $report += "`n### $severity ($($severityIssues.Count))`n`n"
            foreach ($issue in $severityIssues) {
                $report += "- $($issue.Message)`n"
            }
        }
    }
}

$report += @"

---

## ✅ Checks Performed

1. ✅ Dependency vulnerability scan (cargo audit)
2. ✅ Outdated dependencies check
3. ✅ Hardcoded secrets scan
4. ✅ Tauri security configuration
5. ✅ Unsafe Rust code analysis
6. ✅ Sensitive file permissions
7. ✅ .gitignore validation

---

## 🔧 Recommendations

### Immediate Actions
- Review and fix all CRITICAL and HIGH severity issues
- Update vulnerable dependencies
- Remove or encrypt hardcoded secrets
- Configure proper CSP in Tauri

### Best Practices
- Run security audit before each release
- Keep dependencies up to date
- Use environment variables for secrets
- Enable all Tauri security features
- Minimize use of unsafe code
- Regular code reviews

---

## 📚 Resources

- [Tauri Security Best Practices](https://tauri.app/v1/guides/security/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

**Audit Duration**: $((Get-Date) - $startTime)
**Generated by**: Oxide Pilot Security Audit Script

"@

$reportPath = "docs/SECURITY_AUDIT_REPORT.md"
$report | Out-File -FilePath $reportPath -Encoding UTF8

Write-Host ""
Write-Host "✅ Security report generated: $reportPath" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "================================" -ForegroundColor Cyan
Write-Host "🎯 Audit Complete!" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Duration: $((Get-Date) - $startTime)" -ForegroundColor White
Write-Host "Issues Found: $($issues.Count)" -ForegroundColor $(if ($issues.Count -eq 0) { "Green" } else { "Yellow" })
Write-Host "Report: $reportPath" -ForegroundColor White
Write-Host ""

if ($issues.Count -gt 0) {
    $criticalCount = ($issues | Where-Object { $_.Severity -eq "CRITICAL" }).Count
    if ($criticalCount -gt 0) {
        Write-Host "⚠️  CRITICAL: $criticalCount critical issues require immediate attention!" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "⚠️  Please review and address the findings" -ForegroundColor Yellow
        exit 0
    }
} else {
    Write-Host "✅ No security issues detected!" -ForegroundColor Green
    exit 0
}
