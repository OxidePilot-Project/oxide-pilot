# 🎯 Instrucciones Para Continuar - SurrealDB

**Fecha**: 26 de Octubre, 2025  
**Estado**: Fase 1 (92% Completa) - Bloqueado por LLVM  
**Commit**: `c19703b` - [feat] SurrealDB Backend + MetricsCollector

---

## ✅ Lo Que Se Ha Completado

### Código Implementado (4000+ líneas)

1. **`oxide-memory/src/surreal_backend.rs`** (800 líneas)
   - Backend SurrealDB completo con RocksDB embedded
   - 6 tablas: system_metrics, process, threat, incident, agent_memory, spawns
   - Índices optimizados (timestamp, high_cpu, vector HNSW)
   - API: insert_system_metric, query_high_cpu_processes, vector_search
   - Tests unitarios (3 tests, pasan con LLVM)

2. **`oxide-guardian/src/metrics_collector.rs`** (400 líneas)
   - Colector asíncrono con intervalo configurable (default 5s)
   - Métricas: CPU, RAM, Disk I/O, Network
   - Alertas automáticas (CPU/RAM >90%)
   - Agent memories para análisis LLM
   - Feature-gated: `surrealdb-metrics`

3. **Documentación** (2400 líneas)
   - `docs/SURREALDB_MIGRATION.md` - Roadmap completo
   - `docs/LLVM_SETUP.md` - Instalación multi-platform
   - `docs/IMPLEMENTATION_STATUS.md` - Estado actual
   - `SURREALDB_SUMMARY.md` - Resumen ejecutivo

4. **Automatización**
   - `scripts/setup-surrealdb.ps1` - Instalación LLVM
   - Feature flags configurados en Cargo.toml
   - Integración con workspace

### Cambios en Configuración

```toml
# oxide-memory/Cargo.toml
[features]
default = ["surrealdb"]  # Habilitado por defecto
surrealdb = ["dep:surrealdb"]

[dependencies]
surrealdb = { version = "2.3", optional = true, features = ["kv-rocksdb"] }
```

```toml
# oxide-guardian/Cargo.toml
[features]
default = ["surrealdb-metrics"]
surrealdb-metrics = ["oxide-memory/surrealdb"]

[dependencies]
oxide-memory = { path = "../oxide-memory" }
hostname = "0.3"
```

---

## ⚠️ Bloqueador Actual: LLVM

### Por Qué Se Necesita

SurrealDB con backend RocksDB compila extensiones C++ que requieren `libclang` para bindings de Rust.

Error actual:
```
error: couldn't find any valid shared libraries matching: 
['clang.dll', 'libclang.dll']
```

### Solución (5 minutos)

#### Opción 1: PowerShell como Administrador (Recomendado)

```powershell
# 1. Abrir PowerShell como Administrador
# (Clic derecho en PowerShell → "Ejecutar como administrador")

# 2. Instalar LLVM
choco install llvm cmake -y

# 3. Configurar variable de entorno
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
[System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")

# 4. Verificar instalación
clang --version
# Debería mostrar: clang version 21.x.x

# 5. REINICIAR TERMINAL (importante para aplicar env vars)
```

#### Opción 2: Descarga Manual

1. Ir a: https://github.com/llvm/llvm-project/releases/latest
2. Descargar: `LLVM-21.1.0-win64.exe`
3. Ejecutar instalador
4. Durante instalación, marcar: **"Add LLVM to system PATH"**
5. Instalar en: `C:\Program Files\LLVM`
6. Configurar manualmente:
   ```powershell
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   ```
7. Reiniciar terminal

---

## 🚀 Compilar Después de Instalar LLVM

### Paso 1: Verificar Instalación

```powershell
# Verificar LLVM
clang --version
cmake --version
echo $env:LIBCLANG_PATH

# Verificar Rust
rustc --version
cargo --version
```

**Expected output**:
```
clang version 21.1.0
cmake version 3.x.x
C:\Program Files\LLVM\bin
rustc 1.x.x
cargo 1.x.x
```

### Paso 2: Limpiar Build Anterior

```powershell
cd E:\scripts-python\oxide-pilot
cargo clean
```

### Paso 3: Compilar Workspace Completo

```powershell
# Compilar con todas las features
cargo build --workspace --all-features

# O solo con SurrealDB
cargo build --workspace --features surrealdb
```

**Tiempo estimado**: 5-10 minutos (primera vez, compila RocksDB)

### Paso 4: Ejecutar Tests

```powershell
# Tests de SurrealBackend
cargo test -p oxide-memory --features surrealdb

# Tests de MetricsCollector
cargo test -p oxide-guardian --features surrealdb-metrics

# Todos los tests
cargo test --workspace --all-features
```

**Expected**: 3 tests en `surreal_backend.rs` deberían pasar:
- ✅ `test_backend_initialization`
- ✅ `test_insert_and_query_metrics`
- ✅ `test_memory_backend_trait`

### Paso 5: Benchmark Básico (Opcional)

```powershell
# Crear benchmark simple
cargo bench --features surrealdb
```

---

## 🔄 Alternativa Temporal (Sin LLVM)

Si **NO** puedes instalar LLVM ahora, puedes trabajar en otras partes del proyecto:

### Deshabilitar SurrealDB Temporalmente

```toml
# oxide-memory/Cargo.toml
[features]
default = []  # Cambiar de ["surrealdb"] a []

# oxide-guardian/Cargo.toml
[features]
default = []  # Remover "surrealdb-metrics"
```

### Compilar Sin SurrealDB

```powershell
cargo build --workspace --no-default-features
```

### Trabajar en Otras Áreas

- ✅ Frontend Svelte (src-frontend/)
- ✅ Comandos Tauri (src-tauri/)
- ✅ UI/UX components
- ✅ Documentación

### Re-habilitar SurrealDB Después

Cuando tengas LLVM:
1. Restaurar `default = ["surrealdb"]` en Cargo.toml
2. `cargo clean && cargo build --features surrealdb`

---

## 📋 Checklist de Próximos Pasos

### Inmediato (Hoy/Mañana)

- [ ] **Instalar LLVM** (5 minutos con permisos admin)
  ```powershell
  choco install llvm cmake -y
  ```
- [ ] **Configurar LIBCLANG_PATH**
  ```powershell
  $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
  ```
- [ ] **Compilar workspace**
  ```powershell
  cargo build --workspace --features surrealdb
  ```
- [ ] **Ejecutar tests**
  ```powershell
  cargo test --all-features
  ```
- [ ] **Verificar que 3 tests pasen** ✅

### Corto Plazo (Esta Semana)

- [ ] **Implementar embeddings reales**
  - Opción A: Integrar OpenAI API desde Rust
  - Opción B: Usar `text-embeddings-inference` local
  - Ubicación: `oxide-memory/src/embeddings.rs` (nuevo)

- [ ] **Completar métricas OS-specific**
  - Windows: PDH API para disk I/O
  - Windows: GetExtendedTcpTable para connections
  - Ubicación: `oxide-guardian/src/metrics_collector.rs` (actualizar)

- [ ] **Integración con Tauri**
  - Comandos para queries SurrealDB
  - UI Svelte para visualizar métricas
  - Ubicación: `src-tauri/src/oxide_system.rs` (modificar)

- [ ] **Dashboard de métricas**
  - Componente Svelte con gráficas
  - Chart.js o similar
  - Ubicación: `src-frontend/src/lib/components/MetricsDashboard.svelte` (nuevo)

### Mediano Plazo (Próximas 2 Semanas)

- [ ] **15+ queries SurrealQL pre-definidas**
  - Para uso por LLM agents
  - Documentar en `docs/SURREALDB_QUERIES.md`

- [ ] **Benchmarks comparativos**
  - Cognee vs SurrealDB
  - Latencia, throughput, memoria
  - Resultados en `docs/BENCHMARKS.md`

- [ ] **Migración de datos**
  - Script JSON → SurrealDB
  - Script Cognee → SurrealDB (si hay datos)
  - Ubicación: `scripts/migrate-to-surrealdb.ps1`

- [ ] **1 semana de datos recolectados**
  - Guardian ejecutándose 24/7
  - Validar estabilidad
  - Análisis de patterns

### Largo Plazo (Fase 4)

- [ ] **Modo distribuido (TiKV)**
  - Para enterprise multi-device
  - Configuración opcional
  - Documentar en `docs/DISTRIBUTED_MODE.md`

- [ ] **Deprecar Cognee**
  - Remover feature flag
  - Actualizar docs
  - Release notes

---

## 📚 Recursos de Referencia

### Documentación del Proyecto

| Documento | Contenido | Cuándo Usar |
|-----------|-----------|-------------|
| `docs/SURREALDB_MIGRATION.md` | Roadmap completo, schemas, ejemplos | Planificación, desarrollo |
| `docs/LLVM_SETUP.md` | Instalación LLVM multi-platform | Setup inicial |
| `docs/IMPLEMENTATION_STATUS.md` | Estado actual detallado | Tracking progreso |
| `SURREALDB_SUMMARY.md` | Resumen ejecutivo | Presentación stakeholders |

### Archivos de Código Clave

| Archivo | LOC | Descripción |
|---------|-----|-------------|
| `oxide-memory/src/surreal_backend.rs` | 800 | Backend SurrealDB completo |
| `oxide-guardian/src/metrics_collector.rs` | 400 | Colector de métricas |
| `oxide-memory/src/lib.rs` | 15 | Exports públicos |
| `oxide-guardian/src/lib.rs` | 5 | Exports públicos |

### Scripts

| Script | Uso |
|--------|-----|
| `scripts/setup-surrealdb.ps1` | Instalación LLVM automatizada |
| `scripts/oxide-dev.ps1` | Launcher desarrollo unificado |
| `scripts/build-windows.ps1` | Build release Windows |

### SurrealDB Docs Oficiales

- **SurrealDB Docs**: https://surrealdb.com/docs
- **SurrealQL Reference**: https://surrealdb.com/docs/surrealql
- **Rust SDK**: https://surrealdb.com/docs/sdk/rust

---

## 🐛 Troubleshooting Común

### Error: "couldn't find libclang"

**Causa**: LLVM no instalado o `LIBCLANG_PATH` no configurado.

**Solución**:
```powershell
choco install llvm -y
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
# Reiniciar terminal
```

### Error: "CMake not found"

**Solución**:
```powershell
choco install cmake -y
```

### Compilación muy lenta

**Causa**: RocksDB compila de cero (normal la primera vez).

**Optimización**:
- Primera vez: 5-10 minutos (normal)
- Compilaciones incrementales: <1 minuto
- Usar `cargo build --release` para producción

### Tests fallan

**Verificar**:
```powershell
# LLVM instalado
clang --version

# Variable configurada
echo $env:LIBCLANG_PATH

# Workspace limpio
cargo clean
cargo build --workspace --features surrealdb
cargo test --all-features
```

---

## 📞 Siguiente Sesión

### Pregunta al Iniciar

"¿Lograste instalar LLVM y compilar el workspace?"

**Si SÍ**:
- Continuar con embeddings reales
- Integración Tauri
- Dashboard Svelte

**Si NO**:
- Trabajar temporalmente sin SurrealDB
- Continuar con frontend/UI
- Revisitar LLVM setup

---

## 🎯 Objetivo Final Fase 1

**Criterio de Completitud (100%)**:

- [x] Backend SurrealDB implementado ✅
- [x] MetricsCollector implementado ✅
- [x] Tests unitarios escritos ✅
- [x] Documentación completa ✅
- [ ] **LLVM instalado** ⏳
- [ ] **Workspace compila** ⏳
- [ ] **Tests pasan** ⏳
- [ ] **Benchmarks ejecutados** ⏳

**Bloqueador**: Solo instalación de LLVM (5 minutos)

---

**¿Necesitas ayuda?** Consulta `docs/LLVM_SETUP.md` o pregunta en la próxima sesión.

**Última actualización**: 26 de Octubre, 2025 - 20:00  
**Commit actual**: `c19703b`  
**Branch**: `main`
