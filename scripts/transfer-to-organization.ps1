# Script para transferir el repositorio a la organización OxidePilot-Project

param(
    [string]$Token = $env:GITHUB_TOKEN,
    [string]$Repo = "oxide-pilot",
    [string]$CurrentOwner = "iberi22",
    [string]$NewOwner = "OxidePilot-Project"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                                                                            ║" -ForegroundColor Cyan
Write-Host "║           📦 TRANSFERIR REPOSITORIO A ORGANIZACIÓN                         ║" -ForegroundColor Cyan
Write-Host "║                                                                            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

if (-not $Token) {
    Write-Host "❌ Error: No se encontró GITHUB_TOKEN" -ForegroundColor Red
    Write-Host "   Ejecuta: `$env:GITHUB_TOKEN = 'tu_token'" -ForegroundColor Yellow
    exit 1
}

Write-Host "📋 Configuración:" -ForegroundColor Cyan
Write-Host "   Repositorio actual: $CurrentOwner/$Repo" -ForegroundColor White
Write-Host "   Nuevo propietario: $NewOwner" -ForegroundColor White
Write-Host ""

# Verificar que el repositorio existe
Write-Host "🔍 Verificando repositorio..." -ForegroundColor Yellow

$headers = @{
    "Authorization" = "Bearer $Token"
    "Accept" = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
}

try {
    $repoInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$CurrentOwner/$Repo" -Headers $headers -Method Get
    Write-Host "✓ Repositorio encontrado: $($repoInfo.full_name)" -ForegroundColor Green
} catch {
    Write-Host "❌ Error: No se pudo encontrar el repositorio $CurrentOwner/$Repo" -ForegroundColor Red
    Write-Host "   $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Confirmar transferencia
Write-Host "⚠️  IMPORTANTE: Esta acción transferirá el repositorio a la organización." -ForegroundColor Yellow
Write-Host "   - Perderás acceso de propietario si no eres admin de la organización" -ForegroundColor Yellow
Write-Host "   - Los secretos y configuraciones se mantendrán" -ForegroundColor Yellow
Write-Host "   - Los colaboradores se mantendrán" -ForegroundColor Yellow
Write-Host ""

$confirm = Read-Host "¿Continuar con la transferencia? (y/N)"

if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Host "❌ Transferencia cancelada" -ForegroundColor Red
    exit 0
}

Write-Host ""
Write-Host "🚀 Transfiriendo repositorio..." -ForegroundColor Cyan
Write-Host ""

# Transferir repositorio
$transferBody = @{
    new_owner = $NewOwner
} | ConvertTo-Json

try {
    $transferResult = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/$CurrentOwner/$Repo/transfer" `
        -Headers $headers `
        -Method Post `
        -Body $transferBody `
        -ContentType "application/json"
    
    Write-Host "✅ Repositorio transferido exitosamente!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📦 Nueva ubicación: https://github.com/$NewOwner/$Repo" -ForegroundColor Cyan
    Write-Host ""
    
} catch {
    Write-Host "❌ Error durante la transferencia: $_" -ForegroundColor Red
    
    if ($_.Exception.Response.StatusCode -eq 403) {
        Write-Host ""
        Write-Host "⚠️  Posibles causas:" -ForegroundColor Yellow
        Write-Host "   1. No tienes permisos para transferir a esta organización" -ForegroundColor White
        Write-Host "   2. El token no tiene permisos suficientes (necesita: repo, admin:org)" -ForegroundColor White
        Write-Host "   3. La organización no permite transferencias" -ForegroundColor White
        Write-Host ""
        Write-Host "💡 Solución alternativa:" -ForegroundColor Cyan
        Write-Host "   1. Ve a: https://github.com/$CurrentOwner/$Repo/settings" -ForegroundColor White
        Write-Host "   2. Scroll hasta el final: 'Danger Zone'" -ForegroundColor White
        Write-Host "   3. Click en 'Transfer'" -ForegroundColor White
        Write-Host "   4. Sigue las instrucciones en pantalla" -ForegroundColor White
    }
    
    exit 1
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# Actualizar remoto local
Write-Host ""
Write-Host "🔄 Actualizando remoto local..." -ForegroundColor Cyan

try {
    $gitRemote = git remote -v 2>&1
    
    if ($gitRemote -match "iberi22/oxide-pilot") {
        git remote set-url origin "https://github.com/$NewOwner/$Repo.git"
        Write-Host "✓ Remoto actualizado a: https://github.com/$NewOwner/$Repo.git" -ForegroundColor Green
    } else {
        Write-Host "⚠️  No se detectó remoto de git en este directorio" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️  No se pudo actualizar el remoto automáticamente" -ForegroundColor Yellow
    Write-Host "   Ejecuta manualmente:" -ForegroundColor Yellow
    Write-Host "   git remote set-url origin https://github.com/$NewOwner/$Repo.git" -ForegroundColor Gray
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
Write-Host "✅ TRANSFERENCIA COMPLETADA" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Próximos pasos:" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Verificar el repositorio en:" -ForegroundColor White
Write-Host "   https://github.com/$NewOwner/$Repo" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Verificar secretos (se mantienen):" -ForegroundColor White
Write-Host "   https://github.com/$NewOwner/$Repo/settings/secrets/actions" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Hacer pull para verificar:" -ForegroundColor White
Write-Host "   git pull origin main" -ForegroundColor Gray
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""
