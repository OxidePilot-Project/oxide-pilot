# 🚀 SurrealDB Setup Script para Oxide Pilot
# 
# Este script instala las dependencias necesarias para compilar SurrealDB con RocksDB
# 
# Uso: .\scripts\setup-surrealdb.ps1

Write-Host "🔧 Configurando dependencias de SurrealDB..." -ForegroundColor Cyan

# Verificar si estamos en Windows
if ($IsWindows -or $env:OS -match "Windows") {
    Write-Host "📦 Sistema operativo: Windows" -ForegroundColor Green
    
    # Verificar si Chocolatey está instalado
    if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Chocolatey no encontrado. Instalando..." -ForegroundColor Yellow
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    } else {
        Write-Host "✅ Chocolatey ya está instalado" -ForegroundColor Green
    }
    
    # Instalar LLVM (incluye libclang)
    Write-Host "📥 Instalando LLVM/Clang..." -ForegroundColor Cyan
    choco install llvm -y
    
    # Configurar LIBCLANG_PATH
    $llvmPath = "C:\Program Files\LLVM\bin"
    if (Test-Path $llvmPath) {
        Write-Host "✅ LLVM instalado en: $llvmPath" -ForegroundColor Green
        
        # Agregar a variables de entorno de usuario
        [System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $llvmPath, "User")
        $env:LIBCLANG_PATH = $llvmPath
        
        Write-Host "🔐 Variable LIBCLANG_PATH configurada: $llvmPath" -ForegroundColor Green
    } else {
        Write-Host "⚠️  LLVM instalado pero no se encuentra en la ruta esperada" -ForegroundColor Yellow
        Write-Host "   Busca manualmente la carpeta 'bin' de LLVM y configura:" -ForegroundColor Yellow
        Write-Host "   `$env:LIBCLANG_PATH = 'C:\Path\To\LLVM\bin'" -ForegroundColor Yellow
    }
    
    # Verificar si CMake está instalado (necesario para RocksDB)
    if (!(Get-Command cmake -ErrorAction SilentlyContinue)) {
        Write-Host "📥 Instalando CMake..." -ForegroundColor Cyan
        choco install cmake -y
    } else {
        Write-Host "✅ CMake ya está instalado" -ForegroundColor Green
    }
    
    # Refrescar variables de entorno
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    
} else {
    Write-Host "📦 Sistema operativo: Linux/macOS" -ForegroundColor Green
    
    if ($IsMacOS) {
        Write-Host "📥 Instalando LLVM con Homebrew..." -ForegroundColor Cyan
        brew install llvm cmake
        
        # Configurar LIBCLANG_PATH para macOS
        $llvmPath = "/opt/homebrew/opt/llvm/lib" # Apple Silicon
        if (!(Test-Path $llvmPath)) {
            $llvmPath = "/usr/local/opt/llvm/lib" # Intel Mac
        }
        
        if (Test-Path $llvmPath) {
            [System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $llvmPath, "User")
            $env:LIBCLANG_PATH = $llvmPath
            Write-Host "✅ LIBCLANG_PATH=$llvmPath" -ForegroundColor Green
        }
    } else {
        # Linux
        Write-Host "📥 Instalando dependencias con apt..." -ForegroundColor Cyan
        sudo apt-get update
        sudo apt-get install -y llvm-dev libclang-dev clang cmake build-essential
        
        # Configurar LIBCLANG_PATH para Linux
        $llvmPath = "/usr/lib/llvm-14/lib" # Ajustar según versión
        [System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $llvmPath, "User")
        $env:LIBCLANG_PATH = $llvmPath
        Write-Host "✅ LIBCLANG_PATH=$llvmPath" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "✨ Configuración completada!" -ForegroundColor Green
Write-Host ""
Write-Host "📌 Próximos pasos:" -ForegroundColor Cyan
Write-Host "   1. Reinicia tu terminal para aplicar las variables de entorno" -ForegroundColor White
Write-Host "   2. Ejecuta: cargo build --workspace --features surrealdb" -ForegroundColor White
Write-Host ""
Write-Host "🔍 Verificar instalación:" -ForegroundColor Cyan
Write-Host "   clang --version" -ForegroundColor White
Write-Host "   cmake --version" -ForegroundColor White
Write-Host "   echo `$env:LIBCLANG_PATH" -ForegroundColor White
Write-Host ""
