# 🔧 Instalación Manual de LLVM para SurrealDB

## ⚠️ Requisitos

SurrealDB con backend RocksDB requiere **LLVM/Clang** para compilar. Hay dos opciones:

### Opción 1: Instalar LLVM (Recomendado para Producción)

#### Windows - Método Chocolatey (Requiere PowerShell como Administrador)

```powershell
# Abrir PowerShell como Administrador
choco install llvm cmake -y

# Configurar variable de entorno
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")

# Reiniciar terminal y verificar
clang --version
echo $env:LIBCLANG_PATH
```

#### Windows - Método Manual

1. **Descargar LLVM**: https://github.com/llvm/llvm-project/releases/latest
   - Buscar archivo `LLVM-<version>-win64.exe`
   - Ejemplo: `LLVM-21.1.0-win64.exe`

2. **Instalar**:
   - Ejecutar el instalador
   - Durante instalación, marcar "Add LLVM to system PATH"
   - Instalar en: `C:\Program Files\LLVM`

3. **Configurar PowerShell**:
   ```powershell
   # Agregar a perfil de PowerShell (~\Documents\PowerShell\profile.ps1)
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   ```

4. **Verificar**:
   ```powershell
   clang --version
   # Debería mostrar: clang version 21.1.0
   ```

#### macOS

```bash
brew install llvm cmake

# Configurar PATH (agregar a ~/.zshrc o ~/.bashrc)
export LIBCLANG_PATH="/opt/homebrew/opt/llvm/lib"  # Apple Silicon
# O para Intel Macs:
# export LIBCLANG_PATH="/usr/local/opt/llvm/lib"

# Recargar
source ~/.zshrc

# Verificar
clang --version
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y llvm-dev libclang-dev clang cmake build-essential

# Configurar (agregar a ~/.bashrc)
export LIBCLANG_PATH="/usr/lib/llvm-14/lib"

# Recargar
source ~/.bashrc

# Verificar
clang --version
```

### Opción 2: Usar Memory Backend (Desarrollo Temporal)

Si no puedes instalar LLVM ahora, puedes deshabilitar temporalmente SurrealDB:

#### 1. Modificar `oxide-memory/Cargo.toml`

```toml
[features]
default = []  # Cambiar de ["surrealdb"] a []
cognee = []
surrealdb = ["dep:surrealdb"]
```

#### 2. Modificar `oxide-guardian/Cargo.toml`

```toml
[features]
default = []  # Remover "surrealdb-metrics"
yara-detection = ["yara"]
jemalloc = []
surrealdb-metrics = ["oxide-memory/surrealdb"]
```

#### 3. Compilar sin SurrealDB

```powershell
cargo build --workspace --no-default-features
```

#### 4. Habilitar SurrealDB después

```powershell
# Cuando tengas LLVM instalado:
cargo build --workspace --features surrealdb
```

---

## 🚀 Compilación Completa (Con LLVM Instalado)

### 1. Verificar Dependencias

```powershell
# Verificar Rust
rustc --version
cargo --version

# Verificar LLVM
clang --version
cmake --version
echo $env:LIBCLANG_PATH  # Windows
# echo $LIBCLANG_PATH     # Linux/macOS
```

### 2. Compilar Workspace

```powershell
# Limpia build anterior
cargo clean

# Compilar todo con SurrealDB
cargo build --workspace --all-features

# O solo con features necesarias
cargo build --workspace --features "surrealdb,surrealdb-metrics"
```

### 3. Ejecutar Tests

```powershell
# Tests del backend SurrealDB
cargo test -p oxide-memory --features surrealdb

# Tests del colector de métricas
cargo test -p oxide-guardian --features surrealdb-metrics

# Todos los tests
cargo test --workspace --all-features
```

---

## 🐛 Troubleshooting

### Error: "couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']"

**Causa**: `LIBCLANG_PATH` no está configurado o LLVM no está instalado.

**Solución**:
1. Instalar LLVM (ver arriba)
2. Configurar variable de entorno:
   ```powershell
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   ```
3. Reiniciar terminal
4. Recompilar: `cargo clean && cargo build`

### Error: "CMake not found"

**Solución Windows**:
```powershell
choco install cmake -y
# O descargar de: https://cmake.org/download/
```

**Solución Linux**:
```bash
sudo apt-get install cmake
```

### Error: Permisos al instalar con Chocolatey

**Solución**:
1. Abrir PowerShell **como Administrador** (clic derecho → "Ejecutar como administrador")
2. Ejecutar de nuevo: `choco install llvm cmake -y`

### Compilación muy lenta

**Causa**: RocksDB compila de cero (primera vez puede tardar 5-10 minutos).

**Optimización**:
```powershell
# Compilar solo en modo release para producción
cargo build --release --workspace --features surrealdb

# Usar compilación incremental (por defecto en dev)
# Ya está habilitado en desarrollo
```

---

## 📊 Verificación Post-Instalación

### Test Rápido de SurrealDB

```powershell
# Crear archivo de test: test_surreal.rs
cargo new --bin test-surreal
cd test-surreal

# Agregar a Cargo.toml:
# [dependencies]
# surrealdb = { version = "2.3", features = ["kv-rocksdb"] }
# tokio = { version = "1", features = ["full"] }
# anyhow = "1.0"

# Código en src/main.rs:
# Ver ejemplo en docs/SURREALDB_MIGRATION.md

# Ejecutar
cargo run
```

Si compila y ejecuta sin errores, ✅ SurrealDB está correctamente configurado.

---

## 🎯 Next Steps Después de Instalación

1. **Compilar Oxide Pilot**:
   ```powershell
   cd E:\scripts-python\oxide-pilot
   cargo build --workspace --features surrealdb
   ```

2. **Ejecutar tests**:
   ```powershell
   cargo test --workspace --all-features
   ```

3. **Ejecutar en desarrollo**:
   ```powershell
   pwsh -File scripts/oxide-dev.ps1
   ```

4. **Habilitar métricas de Guardian**:
   - El colector de métricas se activa automáticamente con feature `surrealdb-metrics`
   - Ver configuración en `src-tauri/src/oxide_system.rs`

---

## 📚 Referencias

- **LLVM Releases**: https://github.com/llvm/llvm-project/releases
- **SurrealDB Docs**: https://surrealdb.com/docs
- **Rust Bindgen (requiere libclang)**: https://rust-lang.github.io/rust-bindgen/requirements.html
- **RocksDB Build**: https://github.com/rust-rocksdb/rust-rocksdb#requirements

---

**Última actualización**: 26 de octubre de 2025  
**Estado**: Implementación en progreso (Phase 1)
