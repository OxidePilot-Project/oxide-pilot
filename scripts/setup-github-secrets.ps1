# Script para configurar GitHub Secrets usando la API de GitHub
# Este script requiere que tengas un Personal Access Token de GitHub con permisos de repo

param(
    [string]$Owner = "iberi22",
    [string]$Repo = "oxide-pilot",
    [string]$Token = $env:GITHUB_TOKEN
)

$ErrorActionPreference = "Stop"

Write-Host "🔐 Configurando GitHub Secrets para $Owner/$Repo" -ForegroundColor Cyan
Write-Host "=" * 60

# Verificar que tenemos el token
if (-not $Token) {
    Write-Host "❌ Error: No se encontró GITHUB_TOKEN en las variables de entorno" -ForegroundColor Red
    Write-Host ""
    Write-Host "Opciones para configurar el token:" -ForegroundColor Yellow
    Write-Host "1. Ve a https://github.com/settings/tokens" -ForegroundColor Yellow
    Write-Host "2. Genera un nuevo token con permisos: repo, workflow" -ForegroundColor Yellow
    Write-Host "3. Ejecuta: `$env:GITHUB_TOKEN='tu_token_aqui'" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

# Leer los archivos de certificado
$certPath = Join-Path $PSScriptRoot ".." "certs"
$base64Path = Join-Path $certPath "certificate-base64.txt"
$passwordPath = Join-Path $certPath "password.txt"

if (-not (Test-Path $base64Path)) {
    Write-Host "❌ Error: No se encontró $base64Path" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $passwordPath)) {
    Write-Host "❌ Error: No se encontró $passwordPath" -ForegroundColor Red
    exit 1
}

Write-Host "📖 Leyendo archivos de certificado..." -ForegroundColor Yellow
$certBase64 = Get-Content $base64Path -Raw
$certPassword = Get-Content $passwordPath -Raw

# Función para obtener la clave pública del repositorio
function Get-RepoPublicKey {
    param($Owner, $Repo, $Token)
    
    $uri = "https://api.github.com/repos/$Owner/$Repo/actions/secrets/public-key"
    $headers = @{
        "Authorization" = "Bearer $Token"
        "Accept" = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
    }
    
    try {
        $response = Invoke-RestMethod -Uri $uri -Headers $headers -Method Get
        return $response
    } catch {
        Write-Host "❌ Error al obtener la clave pública: $_" -ForegroundColor Red
        throw
    }
}

# Función para encriptar un secreto usando la clave pública
function Encrypt-Secret {
    param($Secret, $PublicKey)
    
    # Convertir la clave pública de base64
    $publicKeyBytes = [System.Convert]::FromBase64String($PublicKey)
    
    # Convertir el secreto a bytes
    $secretBytes = [System.Text.Encoding]::UTF8.GetBytes($Secret)
    
    # Usar libsodium para encriptar (esto requiere Sodium.Core NuGet package)
    # Para simplificar, usaremos un método alternativo con .NET
    
    # Nota: GitHub requiere encriptación con libsodium sealed_box
    # Como PowerShell no tiene esto nativamente, usaremos un script Python inline
    
    $pythonScript = @"
import base64
import sys
from nacl import encoding, public

def encrypt_secret(public_key: str, secret_value: str) -> str:
    public_key_bytes = base64.b64decode(public_key)
    public_key_obj = public.PublicKey(public_key_bytes)
    sealed_box = public.SealedBox(public_key_obj)
    encrypted = sealed_box.encrypt(secret_value.encode('utf-8'))
    return base64.b64encode(encrypted).decode('utf-8')

if __name__ == '__main__':
    public_key = sys.argv[1]
    secret = sys.argv[2]
    print(encrypt_secret(public_key, secret))
"@
    
    # Guardar script temporal
    $tempPy = [System.IO.Path]::GetTempFileName() + ".py"
    Set-Content -Path $tempPy -Value $pythonScript
    
    try {
        # Ejecutar Python para encriptar
        $encrypted = python $tempPy $PublicKey $Secret
        return $encrypted
    } finally {
        Remove-Item $tempPy -ErrorAction SilentlyContinue
    }
}

# Función para crear o actualizar un secreto
function Set-GitHubSecret {
    param($Owner, $Repo, $Token, $SecretName, $SecretValue, $KeyId, $PublicKey)
    
    Write-Host "🔑 Configurando secreto: $SecretName" -ForegroundColor Yellow
    
    # Encriptar el secreto
    try {
        $encryptedValue = Encrypt-Secret -Secret $SecretValue -PublicKey $PublicKey
    } catch {
        Write-Host "⚠️  No se pudo encriptar con Python, intentando método alternativo..." -ForegroundColor Yellow
        Write-Host "   Por favor, configura manualmente en GitHub:" -ForegroundColor Yellow
        Write-Host "   https://github.com/$Owner/$Repo/settings/secrets/actions" -ForegroundColor Cyan
        return $false
    }
    
    $uri = "https://api.github.com/repos/$Owner/$Repo/actions/secrets/$SecretName"
    $headers = @{
        "Authorization" = "Bearer $Token"
        "Accept" = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
        "Content-Type" = "application/json"
    }
    
    $body = @{
        encrypted_value = $encryptedValue
        key_id = $KeyId
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri $uri -Headers $headers -Method Put -Body $body | Out-Null
        Write-Host "   ✅ Secreto configurado exitosamente" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "   ❌ Error al configurar secreto: $_" -ForegroundColor Red
        return $false
    }
}

# Obtener la clave pública del repositorio
Write-Host ""
Write-Host "🔐 Obteniendo clave pública del repositorio..." -ForegroundColor Yellow
try {
    $publicKeyInfo = Get-RepoPublicKey -Owner $Owner -Repo $Repo -Token $Token
    Write-Host "   ✅ Clave pública obtenida" -ForegroundColor Green
} catch {
    Write-Host "❌ No se pudo obtener la clave pública. Verifica que:" -ForegroundColor Red
    Write-Host "   1. El token tiene permisos correctos (repo, workflow)" -ForegroundColor Yellow
    Write-Host "   2. El repositorio existe: https://github.com/$Owner/$Repo" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "📝 Configurando secretos..." -ForegroundColor Cyan
Write-Host ""

# Verificar si Python y PyNaCl están disponibles
try {
    python -c "import nacl" 2>$null
    $hasPyNaCl = $true
} catch {
    $hasPyNaCl = $false
    Write-Host "⚠️  PyNaCl no está instalado. Instalando..." -ForegroundColor Yellow
    pip install PyNaCl 2>$null
    
    # Verificar nuevamente
    try {
        python -c "import nacl" 2>$null
        $hasPyNaCl = $true
        Write-Host "   ✅ PyNaCl instalado exitosamente" -ForegroundColor Green
    } catch {
        $hasPyNaCl = $false
    }
}

if (-not $hasPyNaCl) {
    Write-Host ""
    Write-Host "⚠️  No se pudo instalar PyNaCl automáticamente" -ForegroundColor Yellow
    Write-Host "   Configura los secretos manualmente en:" -ForegroundColor Yellow
    Write-Host "   https://github.com/$Owner/$Repo/settings/secrets/actions" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   Secretos a configurar:" -ForegroundColor Yellow
    Write-Host "   1. SIGN_PFX_BASE64 = (contenido de certificate-base64.txt)" -ForegroundColor White
    Write-Host "   2. SIGN_PFX_PASSWORD = $certPassword" -ForegroundColor White
    Write-Host "   3. SIGN_TS_URL = http://timestamp.digicert.com" -ForegroundColor White
    exit 0
}

# Configurar los secretos
$secrets = @(
    @{ Name = "SIGN_PFX_BASE64"; Value = $certBase64.Trim() },
    @{ Name = "SIGN_PFX_PASSWORD"; Value = $certPassword.Trim() },
    @{ Name = "SIGN_TS_URL"; Value = "http://timestamp.digicert.com" }
)

$successCount = 0
foreach ($secret in $secrets) {
    $result = Set-GitHubSecret -Owner $Owner -Repo $Repo -Token $Token `
        -SecretName $secret.Name -SecretValue $secret.Value `
        -KeyId $publicKeyInfo.key_id -PublicKey $publicKeyInfo.key
    
    if ($result) {
        $successCount++
    }
}

Write-Host ""
Write-Host "=" * 60
if ($successCount -eq $secrets.Count) {
    Write-Host "✅ Todos los secretos se configuraron exitosamente!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🎉 Configuración completa. Ahora puedes:" -ForegroundColor Cyan
    Write-Host "   1. Crear releases firmadas automáticamente" -ForegroundColor White
    Write-Host "   2. Ver tus secretos en: https://github.com/$Owner/$Repo/settings/secrets/actions" -ForegroundColor White
    Write-Host ""
    Write-Host "⚠️  IMPORTANTE: Elimina el archivo password.txt" -ForegroundColor Yellow
    Write-Host "   Remove-Item '$passwordPath'" -ForegroundColor Gray
} else {
    Write-Host "⚠️  Se configuraron $successCount de $($secrets.Count) secretos" -ForegroundColor Yellow
    Write-Host "   Verifica manualmente: https://github.com/$Owner/$Repo/settings/secrets/actions" -ForegroundColor Yellow
}
Write-Host ""
