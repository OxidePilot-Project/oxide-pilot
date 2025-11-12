# Workflow Validation Script
# Verifica que los workflows estén configurados correctamente

Write-Host "=== VERIFICACIÓN DE WORKFLOWS ===" -ForegroundColor Cyan
Write-Host ""

$workflowsPath = ".github\workflows"
$errors = @()
$warnings = @()

# 1. Verificar que los archivos de workflow existan
Write-Host "[1/5] Verificando archivos de workflow..." -ForegroundColor Green
$requiredWorkflows = @(
    "ci.yml",
    "validate-core.yml",
    "validate.yml"
)

foreach ($workflow in $requiredWorkflows) {
    $path = Join-Path $workflowsPath $workflow
    if (Test-Path $path) {
        Write-Host "  ✓ $workflow" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $workflow no encontrado" -ForegroundColor Red
        $errors += "$workflow no encontrado"
    }
}

# Verificar workflows desactivados
if (Test-Path (Join-Path $workflowsPath "windows-ci.yml.disabled")) {
    Write-Host "  ℹ windows-ci.yml.disabled (correcto - redundante)" -ForegroundColor Yellow
}

# 2. Verificar features en workflows
Write-Host "`n[2/5] Verificando features de Cargo..." -ForegroundColor Green
$workflowFiles = Get-ChildItem -Path $workflowsPath -Filter "*.yml" | Where-Object { $_.Name -notlike "*.disabled" }

foreach ($file in $workflowFiles) {
    $content = Get-Content $file.FullName -Raw
    
    # Verificar que no se use --no-default-features con surrealdb-metrics solo
    if ($content -match "--no-default-features.*surrealdb-metrics" -and $content -notmatch "custom-protocol") {
        Write-Host "  ⚠ $($file.Name): Usa --no-default-features sin custom-protocol" -ForegroundColor Yellow
        $warnings += "$($file.Name): Revisa las features"
    }
    
    # Verificar que se use surrealdb-metrics
    if ($content -match "cargo (build|test|clippy)" -and $content -notmatch "surrealdb-metrics") {
        Write-Host "  ⚠ $($file.Name): No especifica surrealdb-metrics" -ForegroundColor Yellow
        $warnings += "$($file.Name): Falta feature surrealdb-metrics"
    }
}

Write-Host "  ✓ Features verificadas" -ForegroundColor Green

# 3. Verificar dependencias de Ubuntu
Write-Host "`n[3/5] Verificando dependencias de Ubuntu..." -ForegroundColor Green

foreach ($file in $workflowFiles) {
    $content = Get-Content $file.FullName -Raw
    
    if ($content -match "ubuntu") {
        if ($content -match "libwebkit2gtk-4\.1-dev" -and $content -match "\|\|") {
            Write-Host "  ✓ $($file.Name): Dependencias con fallback correcto" -ForegroundColor Green
        } elseif ($content -match "libwebkit2gtk") {
            Write-Host "  ⚠ $($file.Name): Dependencias presentes (verificar fallback manualmente)" -ForegroundColor Yellow
        }
    }
}

# 4. Verificar que macOS no tenga GTK
Write-Host "`n[4/5] Verificando dependencias de macOS..." -ForegroundColor Green
foreach ($file in $workflowFiles) {
    $content = Get-Content $file.FullName -Raw
    
    if ($content -match "macos-latest") {
        if ($content -match "brew install gtk") {
            Write-Host "  ✗ $($file.Name): macOS instala GTK (incorrecto)" -ForegroundColor Red
            $errors += "$($file.Name): macOS no debe instalar GTK"
        } else {
            Write-Host "  ✓ $($file.Name): macOS sin GTK" -ForegroundColor Green
        }
    }
}

# 5. Verificar cache de Rust
Write-Host "`n[5/5] Verificando configuración de cache..." -ForegroundColor Green
foreach ($file in $workflowFiles) {
    $content = Get-Content $file.FullName -Raw
    
    if ($content -match "cargo (build|test)") {
        if ($content -match "swatinem/rust-cache@v2" -or $content -match "actions/cache@v4") {
            Write-Host "  ✓ $($file.Name): Usa cache de Rust" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ $($file.Name): No usa cache de Rust" -ForegroundColor Yellow
            $warnings += "$($file.Name): Agregar cache de Rust"
        }
    }
}

# Resumen
Write-Host "`n" + "="*60 -ForegroundColor Cyan
Write-Host "RESUMEN" -ForegroundColor Cyan
Write-Host "="*60 -ForegroundColor Cyan

if ($errors.Count -eq 0 -and $warnings.Count -eq 0) {
    Write-Host "`n✅ TODOS LOS WORKFLOWS ESTÁN CORRECTOS" -ForegroundColor Green
    Write-Host "`nLos workflows deberían ejecutarse correctamente en GitHub Actions." -ForegroundColor Green
    exit 0
} else {
    if ($errors.Count -gt 0) {
        Write-Host "`n❌ ERRORES ENCONTRADOS ($($errors.Count)):" -ForegroundColor Red
        foreach ($error in $errors) {
            Write-Host "  - $error" -ForegroundColor Red
        }
    }
    
    if ($warnings.Count -gt 0) {
        Write-Host "`n⚠️  ADVERTENCIAS ($($warnings.Count)):" -ForegroundColor Yellow
        foreach ($warning in $warnings) {
            Write-Host "  - $warning" -ForegroundColor Yellow
        }
    }
    
    if ($errors.Count -gt 0) {
        Write-Host "`nCorrige los errores antes de hacer commit." -ForegroundColor Red
        exit 1
    } else {
        Write-Host "`nAdvertencias encontradas, pero los workflows deberían funcionar." -ForegroundColor Yellow
        exit 0
    }
}
