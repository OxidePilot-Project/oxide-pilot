# Script de Configuración Completa - Oxide Pilot
# Ejecuta este script para configurar todo de una vez

param(
    [string]$GitHubToken = $env:GITHUB_TOKEN
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                                                                            ║" -ForegroundColor Cyan
Write-Host "║              🚀 OXIDE PILOT - CONFIGURACIÓN AUTOMÁTICA                     ║" -ForegroundColor Cyan
Write-Host "║                                                                            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Verificar que estamos en el directorio correcto
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ Error: Debes ejecutar este script desde el directorio raíz de oxide-pilot" -ForegroundColor Red
    Write-Host "   cd E:\scripts-python\oxide-pilot" -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Directorio verificado" -ForegroundColor Green
Write-Host ""

# Verificar GitHub Token
if (-not $GitHubToken) {
    Write-Host "⚠️  No se encontró GitHub Token" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Para continuar, necesitas un Personal Access Token de GitHub:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "1. Ve a: https://github.com/settings/tokens" -ForegroundColor White
    Write-Host "2. Click en 'Generate new token' → 'Generate new token (classic)'" -ForegroundColor White
    Write-Host "3. Selecciona permisos: repo, workflow" -ForegroundColor White
    Write-Host "4. Copia el token" -ForegroundColor White
    Write-Host ""
    
    $token = Read-Host "Pega tu GitHub Token aquí"
    
    if (-not $token) {
        Write-Host "❌ Token no proporcionado. Abortando." -ForegroundColor Red
        exit 1
    }
    
    $GitHubToken = $token
}

Write-Host "✓ GitHub Token configurado" -ForegroundColor Green
Write-Host ""

# Verificar que los certificados existen
$certPath = Join-Path $PSScriptRoot "certs"
$pfxPath = Join-Path $certPath "OxidePilot-CodeSigning.pfx"
$base64Path = Join-Path $certPath "certificate-base64.txt"
$passwordPath = Join-Path $certPath "password.txt"

if (-not (Test-Path $pfxPath)) {
    Write-Host "⚠️  No se encontró certificado PFX" -ForegroundColor Yellow
    Write-Host "   Generando nuevo certificado..." -ForegroundColor Cyan
    
    # Ejecutar script de generación de certificados
    & .\scripts\setup-code-signing.ps1
    
    if (-not $?) {
        Write-Host "❌ Error al generar certificado" -ForegroundColor Red
        exit 1
    }
}

Write-Host "✓ Certificado de firma verificado" -ForegroundColor Green
Write-Host ""

# Ejecutar script de configuración de secretos
Write-Host "📤 Subiendo secretos a GitHub..." -ForegroundColor Cyan
Write-Host ""

$env:GITHUB_TOKEN = $GitHubToken

try {
    & .\scripts\setup-github-secrets.ps1
    
    if (-not $?) {
        Write-Host "⚠️  Hubo un problema al configurar los secretos" -ForegroundColor Yellow
        Write-Host "   Puedes configurarlos manualmente en:" -ForegroundColor Yellow
        Write-Host "   https://github.com/iberi22/oxide-pilot/settings/secrets/actions" -ForegroundColor Cyan
    }
} catch {
    Write-Host "⚠️  Error al ejecutar configuración de secretos: $_" -ForegroundColor Yellow
    Write-Host "   Puedes configurarlos manualmente siguiendo SETUP_COMPLETADO.md" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# Verificar secretos en GitHub
Write-Host ""
Write-Host "🔍 Verificando secretos en GitHub..." -ForegroundColor Cyan

$headers = @{
    "Authorization" = "Bearer $GitHubToken"
    "Accept" = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
}

try {
    $secrets = Invoke-RestMethod -Uri "https://api.github.com/repos/iberi22/oxide-pilot/actions/secrets" -Headers $headers -Method Get
    
    Write-Host ""
    Write-Host "Secretos configurados:" -ForegroundColor Green
    foreach ($secret in $secrets.secrets) {
        Write-Host "  ✓ $($secret.name)" -ForegroundColor Green
    }
    
    $expectedSecrets = @("SIGN_PFX_BASE64", "SIGN_PFX_PASSWORD", "SIGN_TS_URL")
    $missingSecrets = $expectedSecrets | Where-Object { $_ -notin $secrets.secrets.name }
    
    if ($missingSecrets.Count -gt 0) {
        Write-Host ""
        Write-Host "⚠️  Secretos faltantes:" -ForegroundColor Yellow
        foreach ($missing in $missingSecrets) {
            Write-Host "  ⚠️  $missing" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "⚠️  No se pudo verificar los secretos: $_" -ForegroundColor Yellow
    Write-Host "   Verifica manualmente en: https://github.com/iberi22/oxide-pilot/settings/secrets/actions" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# Limpiar archivo de contraseña
Write-Host ""
Write-Host "🧹 Limpiando archivos sensibles..." -ForegroundColor Cyan

if (Test-Path $passwordPath) {
    $password = Get-Content $passwordPath -Raw
    Write-Host ""
    Write-Host "⚠️  IMPORTANTE: Guarda esta contraseña en un lugar seguro" -ForegroundColor Yellow
    Write-Host "   Contraseña del certificado: $password" -ForegroundColor White
    Write-Host ""
    
    $remove = Read-Host "¿Eliminar archivo password.txt? (y/N)"
    
    if ($remove -eq "y" -or $remove -eq "Y") {
        Remove-Item $passwordPath -Force
        Write-Host "✓ Archivo eliminado" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Recuerda eliminar manualmente: $passwordPath" -ForegroundColor Yellow
    }
} else {
    Write-Host "✓ Archivo password.txt ya fue eliminado" -ForegroundColor Green
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "✅ CONFIGURACIÓN COMPLETADA" -ForegroundColor Green
Write-Host ""
Write-Host "Próximos pasos:" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Verificar secretos en GitHub:" -ForegroundColor White
Write-Host "   https://github.com/iberi22/oxide-pilot/settings/secrets/actions" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Crear tu primer release:" -ForegroundColor White
Write-Host "   git tag v0.1.0" -ForegroundColor Gray
Write-Host "   git push origin v0.1.0" -ForegroundColor Gray
Write-Host ""
Write-Host "3. O usar el script de release:" -ForegroundColor White
Write-Host "   .\scripts\create-release.ps1 -Version 0.1.0" -ForegroundColor Gray
Write-Host ""
Write-Host "4. Revisar dependabot alerts (10 vulnerabilidades detectadas):" -ForegroundColor White
Write-Host "   https://github.com/iberi22/oxide-pilot/security/dependabot" -ForegroundColor Gray
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "📚 Documentación disponible:" -ForegroundColor Cyan
Write-Host "   - INSTRUCCIONES_RAPIDAS.txt" -ForegroundColor White
Write-Host "   - SETUP_COMPLETADO.md" -ForegroundColor White
Write-Host "   - docs/GITHUB_SECRETS_SETUP.md" -ForegroundColor White
Write-Host ""
Write-Host "🎉 ¡Listo para desarrollar!" -ForegroundColor Green
Write-Host ""
