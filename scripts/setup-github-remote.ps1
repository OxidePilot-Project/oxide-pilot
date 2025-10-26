# Script para configurar repositorio remoto de GitHub
# Oxide Pilot - GitHub Setup

param(
    [Parameter(Mandatory=$false)]
    [string]$RepoOwner = "OxidePilot-Project",
    
    [Parameter(Mandatory=$false)]
    [string]$RepoName = "oxide-pilot",
    
    [switch]$Force
)

Write-Host ""
Write-Host "🚀 Configuración de GitHub Remote" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

# Verificar que estamos en un repositorio git
if (-not (Test-Path .git)) {
    Write-Host "❌ Error: No es un repositorio Git" -ForegroundColor Red
    Write-Host "   Inicializando repositorio..." -ForegroundColor Yellow
    git init
    Write-Host "✅ Repositorio Git inicializado" -ForegroundColor Green
    Write-Host ""
}

# Construir URL del repositorio
$repoUrl = "https://github.com/$RepoOwner/$RepoName.git"

Write-Host "📍 Repositorio destino:" -ForegroundColor Yellow
Write-Host "   Owner: $RepoOwner" -ForegroundColor Cyan
Write-Host "   Repo: $RepoName" -ForegroundColor Cyan
Write-Host "   URL: $repoUrl" -ForegroundColor Cyan
Write-Host ""

# Verificar remotes existentes
$existingRemotes = git remote -v 2>$null

if ($existingRemotes) {
    Write-Host "📋 Remotes actuales:" -ForegroundColor Yellow
    Write-Host $existingRemotes -ForegroundColor Gray
    Write-Host ""
    
    # Verificar si 'origin' ya existe
    $originExists = git remote get-url origin 2>$null
    
    if ($originExists) {
        Write-Host "⚠️  Remote 'origin' ya existe: $originExists" -ForegroundColor Yellow
        
        if ($Force) {
            Write-Host "🔄 Modo --Force activado, reemplazando remote..." -ForegroundColor Yellow
            git remote remove origin
            git remote add origin $repoUrl
            Write-Host "✅ Remote 'origin' actualizado" -ForegroundColor Green
        } else {
            Write-Host ""
            $replace = Read-Host "¿Reemplazar con el nuevo remote? (y/N)"
            
            if ($replace -eq 'y') {
                git remote remove origin
                git remote add origin $repoUrl
                Write-Host "✅ Remote 'origin' actualizado" -ForegroundColor Green
            } else {
                Write-Host "❌ Operación cancelada" -ForegroundColor Yellow
                Write-Host ""
                Write-Host "Puedes agregar el remote con otro nombre:" -ForegroundColor Gray
                Write-Host "   git remote add github $repoUrl" -ForegroundColor Cyan
                exit 0
            }
        }
    } else {
        git remote add origin $repoUrl
        Write-Host "✅ Remote 'origin' agregado" -ForegroundColor Green
    }
} else {
    # No hay remotes, agregar origin
    git remote add origin $repoUrl
    Write-Host "✅ Remote 'origin' agregado" -ForegroundColor Green
}

Write-Host ""

# Verificar estado del repositorio
Write-Host "📊 Estado del repositorio:" -ForegroundColor Yellow
$status = git status --porcelain
$branch = git rev-parse --abbrev-ref HEAD 2>$null

if (-not $branch) {
    Write-Host "   Sin commits aún" -ForegroundColor Gray
    $needsInitialCommit = $true
} else {
    Write-Host "   Rama actual: $branch" -ForegroundColor Cyan
    
    if ($status) {
        Write-Host "   Cambios sin commitear: $($status.Count) archivo(s)" -ForegroundColor Yellow
    } else {
        Write-Host "   ✅ Working directory limpio" -ForegroundColor Green
    }
}

Write-Host ""

# Sugerir siguiente paso
Write-Host "📝 Próximos pasos:" -ForegroundColor Yellow
Write-Host ""

if ($needsInitialCommit) {
    Write-Host "1. Hacer commit inicial:" -ForegroundColor White
    Write-Host "   git add ." -ForegroundColor Cyan
    Write-Host "   git commit -m 'Initial commit: Oxide Pilot v1.0'" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "2. Renombrar rama a 'main' (si es necesario):" -ForegroundColor White
    Write-Host "   git branch -M main" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "3. Push al repositorio:" -ForegroundColor White
    Write-Host "   git push -u origin main" -ForegroundColor Cyan
} elseif ($status) {
    Write-Host "1. Commitear cambios pendientes:" -ForegroundColor White
    Write-Host "   git add ." -ForegroundColor Cyan
    Write-Host "   git commit -m 'Tu mensaje de commit'" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "2. Push al repositorio:" -ForegroundColor White
    Write-Host "   git push -u origin $branch" -ForegroundColor Cyan
} else {
    Write-Host "1. Push al repositorio:" -ForegroundColor White
    Write-Host "   git push -u origin $branch" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "🔐 Autenticación de GitHub:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Si es la primera vez que haces push, GitHub te pedirá autenticación." -ForegroundColor Gray
Write-Host "Opciones:" -ForegroundColor White
Write-Host ""
Write-Host "  1. Personal Access Token (Recomendado):" -ForegroundColor Cyan
Write-Host "     - Ve a: https://github.com/settings/tokens" -ForegroundColor Gray
Write-Host "     - Click 'Generate new token (classic)'" -ForegroundColor Gray
Write-Host "     - Selecciona scope: 'repo' (acceso completo)" -ForegroundColor Gray
Write-Host "     - Copia el token generado" -ForegroundColor Gray
Write-Host "     - Úsalo como contraseña cuando Git lo pida" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. SSH Key:" -ForegroundColor Cyan
Write-Host "     - Cambia remote a SSH: git remote set-url origin git@github.com:$RepoOwner/$RepoName.git" -ForegroundColor Gray
Write-Host "     - Configura SSH key en GitHub" -ForegroundColor Gray
Write-Host ""
Write-Host "  3. GitHub CLI (gh):" -ForegroundColor Cyan
Write-Host "     - Instala: winget install GitHub.cli" -ForegroundColor Gray
Write-Host "     - Autentica: gh auth login" -ForegroundColor Gray
Write-Host ""

# Ofrecer hacer push automáticamente
if (-not $needsInitialCommit -and -not $status) {
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
    $autoPush = Read-Host "¿Hacer push ahora? (y/N)"
    
    if ($autoPush -eq 'y') {
        Write-Host ""
        Write-Host "🚀 Haciendo push a GitHub..." -ForegroundColor Cyan
        
        # Intentar push
        git push -u origin $branch
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "✅ Push completado exitosamente!" -ForegroundColor Green
            Write-Host ""
            Write-Host "🔗 Repositorio: https://github.com/$RepoOwner/$RepoName" -ForegroundColor Cyan
            Write-Host ""
        } else {
            Write-Host ""
            Write-Host "❌ Error en el push. Verifica:" -ForegroundColor Red
            Write-Host "   - Que el repositorio existe en GitHub" -ForegroundColor Yellow
            Write-Host "   - Que tienes permisos de escritura" -ForegroundColor Yellow
            Write-Host "   - Que la autenticación es correcta" -ForegroundColor Yellow
            Write-Host ""
        }
    }
}

Write-Host ""
Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "📚 Recursos útiles:" -ForegroundColor Yellow
Write-Host "   - Repositorio: https://github.com/$RepoOwner/$RepoName" -ForegroundColor Cyan
Write-Host "   - Settings: https://github.com/$RepoOwner/$RepoName/settings" -ForegroundColor Cyan
Write-Host "   - Actions: https://github.com/$RepoOwner/$RepoName/actions" -ForegroundColor Cyan
Write-Host "   - Releases: https://github.com/$RepoOwner/$RepoName/releases" -ForegroundColor Cyan
Write-Host ""
