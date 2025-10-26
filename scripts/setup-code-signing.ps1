# Oxide Pilot - Setup Code Signing
# Este script automatiza la creación de certificado autofirmado para desarrollo

param(
    [Parameter(Mandatory=$false)]
    [string]$CertName = "Oxide Pilot Development",
    
    [Parameter(Mandatory=$false)]
    [string]$Organization = "Oxide Pilot Team",
    
    [Parameter(Mandatory=$false)]
    [int]$ValidYears = 3,
    
    [switch]$SkipGitHubInstructions
)

Write-Host ""
Write-Host "🔐 Oxide Pilot - Code Signing Setup" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

# Verificar que se ejecuta como administrador
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "⚠️  ADVERTENCIA: No se está ejecutando como Administrador" -ForegroundColor Yellow
    Write-Host "Algunas operaciones pueden fallar. Se recomienda ejecutar como Administrador." -ForegroundColor Yellow
    Write-Host ""
    $continue = Read-Host "¿Continuar de todos modos? (y/N)"
    if ($continue -ne 'y') {
        Write-Host "Abortado. Por favor ejecuta este script como Administrador." -ForegroundColor Yellow
        exit 0
    }
    Write-Host ""
}

# Crear directorio para certificados
$certsDir = Join-Path $PSScriptRoot "..\certs"
if (-not (Test-Path $certsDir)) {
    Write-Host "📁 Creando directorio de certificados..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $certsDir -Force | Out-Null
}

Write-Host "✅ Directorio de certificados: $certsDir" -ForegroundColor Green
Write-Host ""

# Verificar si ya existe un certificado
$existingCerts = Get-ChildItem -Path $certsDir -Filter "*.pfx" -ErrorAction SilentlyContinue

if ($existingCerts) {
    Write-Host "⚠️  Se encontraron certificados existentes:" -ForegroundColor Yellow
    $existingCerts | ForEach-Object { Write-Host "   - $($_.Name)" -ForegroundColor Gray }
    Write-Host ""
    $overwrite = Read-Host "¿Crear un nuevo certificado de todos modos? (y/N)"
    if ($overwrite -ne 'y') {
        Write-Host "Abortado. Usando certificado existente." -ForegroundColor Yellow
        exit 0
    }
    Write-Host ""
}

# Paso 1: Crear certificado autofirmado
Write-Host "🔨 Paso 1: Creando certificado autofirmado..." -ForegroundColor Cyan
Write-Host "   Nombre: $CertName" -ForegroundColor Gray
Write-Host "   Organización: $Organization" -ForegroundColor Gray
Write-Host "   Validez: $ValidYears años" -ForegroundColor Gray
Write-Host ""

try {
    $cert = New-SelfSignedCertificate `
        -Subject "CN=$CertName, O=$Organization, C=US" `
        -Type CodeSigningCert `
        -KeyUsage DigitalSignature `
        -FriendlyName "Oxide Pilot Code Signing Certificate" `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears($ValidYears) `
        -HashAlgorithm SHA256 `
        -KeyLength 2048

    Write-Host "✅ Certificado creado exitosamente!" -ForegroundColor Green
    Write-Host "   Thumbprint: $($cert.Thumbprint)" -ForegroundColor Cyan
    Write-Host "   Subject: $($cert.Subject)" -ForegroundColor Cyan
    Write-Host "   Valid Until: $($cert.NotAfter.ToString('yyyy-MM-dd'))" -ForegroundColor Cyan
    Write-Host ""
} catch {
    Write-Host "❌ Error al crear el certificado: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Paso 2: Crear contraseña segura
Write-Host "🔑 Paso 2: Configurando contraseña para el certificado..." -ForegroundColor Cyan
Write-Host ""

$passwordGenerated = $false
$password = $null

Write-Host "Opciones:" -ForegroundColor Yellow
Write-Host "  1. Generar contraseña aleatoria segura (recomendado)" -ForegroundColor Gray
Write-Host "  2. Ingresar contraseña manualmente" -ForegroundColor Gray
Write-Host ""
$passwordOption = Read-Host "Selecciona opción (1/2)"

if ($passwordOption -eq '1') {
    # Generar contraseña aleatoria de forma segura
    try {
        # Método 1: Usar System.Web si está disponible
        Add-Type -AssemblyName 'System.Web' -ErrorAction Stop
        $passwordText = [System.Web.Security.Membership]::GeneratePassword(32, 8)
    } catch {
        # Método 2: Fallback - generar manualmente con caracteres aleatorios
        $chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-="
        $passwordText = -join ((1..32) | ForEach-Object { $chars[(Get-Random -Maximum $chars.Length)] })
    }
    
    $password = ConvertTo-SecureString -String $passwordText -AsPlainText -Force
    $passwordGenerated = $true
    
    Write-Host ""
    Write-Host "✅ Contraseña generada:" -ForegroundColor Green
    Write-Host "   $passwordText" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "⚠️  IMPORTANTE: Guarda esta contraseña en un lugar seguro!" -ForegroundColor Yellow
    Write-Host "   La necesitarás para configurar GitHub Secrets." -ForegroundColor Yellow
    Write-Host ""
    
    # Guardar contraseña en archivo temporal
    $passwordFile = Join-Path $certsDir "password.txt"
    $passwordText | Out-File -FilePath $passwordFile -Encoding ASCII -NoNewline
    Write-Host "   Contraseña guardada temporalmente en: $passwordFile" -ForegroundColor Gray
    Write-Host "   ⚠️  Elimina este archivo después de configurar GitHub Secrets" -ForegroundColor Yellow
    Write-Host ""
    
    Read-Host "Presiona Enter cuando hayas guardado la contraseña"
    Write-Host ""
} else {
    $password = Read-Host "Ingresa una contraseña segura para el certificado (min 16 caracteres)" -AsSecureString
    $passwordConfirm = Read-Host "Confirma la contraseña" -AsSecureString
    
    # Convertir SecureString a texto plano para comparar
    $pwd1 = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($password))
    $pwd2 = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($passwordConfirm))
    
    if ($pwd1 -ne $pwd2) {
        Write-Host "❌ Las contraseñas no coinciden" -ForegroundColor Red
        exit 1
    }
    
    if ($pwd1.Length -lt 16) {
        Write-Host "❌ La contraseña debe tener al menos 16 caracteres" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✅ Contraseña configurada" -ForegroundColor Green
    Write-Host ""
}

# Paso 3: Exportar certificado a PFX
Write-Host "💾 Paso 3: Exportando certificado a archivo PFX..." -ForegroundColor Cyan

$pfxPath = Join-Path $certsDir "OxidePilot-CodeSigning.pfx"

try {
    Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $password -Force | Out-Null
    Write-Host "✅ Certificado exportado a: $pfxPath" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ Error al exportar el certificado: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Paso 4: Convertir PFX a Base64
Write-Host "🔄 Paso 4: Codificando certificado a Base64..." -ForegroundColor Cyan

try {
    $pfxBytes = [System.IO.File]::ReadAllBytes($pfxPath)
    $base64 = [System.Convert]::ToBase64String($pfxBytes)
    
    $base64Path = Join-Path $certsDir "certificate-base64.txt"
    $base64 | Out-File -FilePath $base64Path -Encoding ASCII -NoNewline
    
    Write-Host "✅ Certificado codificado guardado en: $base64Path" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ Error al codificar el certificado: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Paso 5: Verificar certificado
Write-Host "🔍 Paso 5: Verificando certificado..." -ForegroundColor Cyan

try {
    $verifiedCert = Get-PfxCertificate -FilePath $pfxPath -Password $password
    
    Write-Host "✅ Certificado verificado correctamente" -ForegroundColor Green
    Write-Host "   Subject: $($verifiedCert.Subject)" -ForegroundColor Cyan
    Write-Host "   Thumbprint: $($verifiedCert.Thumbprint)" -ForegroundColor Cyan
    Write-Host "   Valid From: $($verifiedCert.NotBefore.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Cyan
    Write-Host "   Valid Until: $($verifiedCert.NotAfter.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Cyan
    
    # Verificar que es para firma de código
    $isCodeSigning = $verifiedCert.EnhancedKeyUsageList.FriendlyName -contains "Code Signing"
    if ($isCodeSigning) {
        Write-Host "   Enhanced Key Usage: ✅ Code Signing" -ForegroundColor Green
    } else {
        Write-Host "   Enhanced Key Usage: ❌ NOT Code Signing" -ForegroundColor Red
        Write-Host "⚠️  ADVERTENCIA: Este certificado no es válido para firma de código" -ForegroundColor Red
    }
    Write-Host ""
} catch {
    Write-Host "❌ Error al verificar el certificado: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Mostrar resumen
Write-Host ""
Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ Configuración completada exitosamente!" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "📂 Archivos generados:" -ForegroundColor Yellow
Write-Host "   - Certificado PFX: $pfxPath" -ForegroundColor Gray
Write-Host "   - Base64 para GitHub: $base64Path" -ForegroundColor Gray
if ($passwordGenerated) {
    Write-Host "   - Contraseña temporal: $(Join-Path $certsDir 'password.txt')" -ForegroundColor Gray
}
Write-Host ""

if (-not $SkipGitHubInstructions) {
    Write-Host "📋 Próximos pasos - Configurar GitHub Secrets:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "1. Ve a tu repositorio en GitHub" -ForegroundColor White
    Write-Host "   https://github.com/YOUR_USERNAME/oxide-pilot" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "2. Click en Settings > Secrets and variables > Actions" -ForegroundColor White
    Write-Host ""
    Write-Host "3. Click en 'New repository secret' y agrega:" -ForegroundColor White
    Write-Host ""
    Write-Host "   Secret Name: SIGN_PFX_BASE64" -ForegroundColor Cyan
    Write-Host "   Secret Value: [Copia TODO el contenido de certificate-base64.txt]" -ForegroundColor Gray
    Write-Host ""
    Write-Host "4. Click en 'New repository secret' nuevamente y agrega:" -ForegroundColor White
    Write-Host ""
    Write-Host "   Secret Name: SIGN_PFX_PASSWORD" -ForegroundColor Cyan
    if ($passwordGenerated) {
        Write-Host "   Secret Value: [Copia el contenido de password.txt]" -ForegroundColor Gray
    } else {
        Write-Host "   Secret Value: [La contraseña que ingresaste]" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "5. (Opcional) Agrega timestamp server:" -ForegroundColor White
    Write-Host ""
    Write-Host "   Secret Name: SIGN_TS_URL" -ForegroundColor Cyan
    Write-Host "   Secret Value: http://timestamp.digicert.com" -ForegroundColor Gray
    Write-Host ""
    Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "⚠️  IMPORTANTE - Seguridad:" -ForegroundColor Yellow
Write-Host ""
Write-Host "   1. NO subas estos archivos a GitHub" -ForegroundColor Red
Write-Host "   2. NO compartas el certificado ni la contraseña" -ForegroundColor Red
Write-Host "   3. Guarda el PFX en un lugar seguro (cifrado)" -ForegroundColor Yellow
Write-Host "   4. Elimina password.txt después de configurar GitHub" -ForegroundColor Yellow
Write-Host ""

# Abrir el directorio de certificados
Write-Host "📁 ¿Abrir el directorio de certificados ahora? (y/N): " -ForegroundColor Cyan -NoNewline
$openDir = Read-Host

if ($openDir -eq 'y') {
    Start-Process explorer.exe -ArgumentList $certsDir
}

Write-Host ""
Write-Host "✅ Setup completado. ¡Ya puedes crear releases firmadas!" -ForegroundColor Green
Write-Host ""
Write-Host "Para más información, consulta: docs/CODE_SIGNING_GUIDE.md" -ForegroundColor Gray
Write-Host ""
