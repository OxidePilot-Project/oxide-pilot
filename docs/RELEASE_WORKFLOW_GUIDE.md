# 🚀 Release Workflow Guide - Oxide Pilot

**Last Updated**: October 28, 2025

---

## 📋 Overview

Este documento describe cómo usar el workflow automatizado de GitHub Actions para crear releases de Oxide Pilot con builds para Windows, macOS y Linux.

---

## 🎯 Workflow: `release-production.yml`

### Características

✅ **Multi-plataforma**: Windows, macOS (Universal), Linux
✅ **Validación pre-release**: Formatting, linting, security audit
✅ **Code signing**: Soporte para firmar binarios
✅ **Checksums**: SHA256 y SHA512 para verificación
✅ **Release notes**: Generación automática
✅ **Artifacts**: Subida automática a GitHub Releases

---

## 🚀 Cómo Crear un Release

### Método 1: Usando Tags (Recomendado)

```bash
# 1. Asegurarse de que todo está commiteado
git status

# 2. Crear y push del tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial public release"
git push origin v0.1.0

# 3. El workflow se ejecutará automáticamente
# Ir a: https://github.com/OxidePilot-Project/oxide-pilot/actions
```

### Método 2: Manual Dispatch

```bash
# 1. Ir a GitHub Actions
# https://github.com/OxidePilot-Project/oxide-pilot/actions/workflows/release-production.yml

# 2. Click en "Run workflow"

# 3. Llenar los campos:
#    - Version: v0.1.0
#    - Pre-release: false (para release estable)

# 4. Click en "Run workflow"
```

---

## 📦 Proceso del Workflow

### 1. Validación (validate)

```yaml
- Check formatting (cargo fmt)
- Run Clippy (0 warnings)
- Security audit (cargo audit)
- Extract version from tag
```

**Duración**: ~2-3 minutos

### 2. Build Windows (build-windows)

```yaml
- Setup environment (Rust, Node, Python)
- Install Tauri CLI
- Install NSIS & WiX
- Build frontend
- Build Tauri app (MSI + NSIS)
- Code signing (opcional)
- Generate checksums
- Upload artifacts
```

**Duración**: ~15-20 minutos
**Outputs**: `.msi`, `.exe`, checksums

### 3. Build macOS (build-macos)

```yaml
- Setup environment
- Build universal binary (Intel + Apple Silicon)
- Code signing (opcional)
- Notarization (opcional)
- Generate checksums
- Upload artifacts
```

**Duración**: ~20-25 minutos
**Outputs**: `.dmg`, checksums

### 4. Build Linux (build-linux)

```yaml
- Install dependencies
- Build Tauri app
- Create DEB package
- Create AppImage
- Generate checksums
- Upload artifacts
```

**Duración**: ~10-15 minutos
**Outputs**: `.deb`, `.AppImage`, checksums

### 5. Create Release (create-release)

```yaml
- Download all artifacts
- Generate release notes
- Create GitHub Release
- Upload all installers
```

**Duración**: ~2-3 minutos

### 6. Notify (notify)

```yaml
- Send success notification
- Log release URL
```

**Duración**: <1 minuto

---

## 🔐 Secrets Requeridos

### Obligatorios

- `GITHUB_TOKEN` - Automático, provisto por GitHub

### Opcionales (Code Signing)

#### Windows

```yaml
SIGN_PFX_BASE64: Base64 del certificado .pfx
SIGN_PFX_PASSWORD: Contraseña del certificado
```

**Cómo configurar**:
```bash
# Convertir .pfx a base64
certutil -encode codesign.pfx codesign.txt
# Copiar el contenido (sin headers) a SIGN_PFX_BASE64
```

#### macOS

```yaml
APPLE_CERTIFICATE: Certificado de Apple
APPLE_CERTIFICATE_PASSWORD: Contraseña del certificado
APPLE_SIGNING_IDENTITY: Identity del certificado
APPLE_ID: Apple ID para notarization
APPLE_PASSWORD: App-specific password
APPLE_TEAM_ID: Team ID de Apple Developer
```

#### Tauri Auto-update

```yaml
TAURI_PRIVATE_KEY: Clave privada para updates
TAURI_KEY_PASSWORD: Contraseña de la clave
```

**Cómo generar**:
```bash
# Generar par de claves
cargo tauri signer generate -w ~/.tauri/myapp.key

# Copiar la clave privada a TAURI_PRIVATE_KEY
# Copiar la contraseña a TAURI_KEY_PASSWORD
```

---

## 📝 Configurar Secrets en GitHub

```bash
# 1. Ir a Settings del repositorio
https://github.com/OxidePilot-Project/oxide-pilot/settings/secrets/actions

# 2. Click en "New repository secret"

# 3. Agregar cada secret:
#    Name: SIGN_PFX_BASE64
#    Value: [contenido base64]

# 4. Click en "Add secret"

# 5. Repetir para cada secret necesario
```

---

## 🎯 Versioning

### Formato de Versiones

Seguimos **Semantic Versioning** (SemVer):

```
v<MAJOR>.<MINOR>.<PATCH>

Ejemplos:
- v0.1.0 - Initial release
- v0.1.1 - Patch release (bug fixes)
- v0.2.0 - Minor release (new features)
- v1.0.0 - Major release (breaking changes)
```

### Cuándo Incrementar

- **MAJOR**: Cambios incompatibles en la API
- **MINOR**: Nueva funcionalidad compatible
- **PATCH**: Bug fixes compatibles

---

## 📊 Monitoreo del Workflow

### Ver Progreso

```bash
# 1. Ir a Actions
https://github.com/OxidePilot-Project/oxide-pilot/actions

# 2. Click en el workflow "Production Release"

# 3. Ver el progreso en tiempo real
```

### Logs

Cada job tiene logs detallados:
- Click en el job (ej: "Build Windows")
- Expandir los steps para ver detalles
- Descargar logs si es necesario

### Artifacts

Durante el build, los artifacts están disponibles:
- Ir al workflow run
- Scroll down a "Artifacts"
- Descargar para testing antes del release

---

## ✅ Checklist Pre-Release

### Código

- [ ] Todos los tests passing
- [ ] 0 Clippy warnings
- [ ] Código formateado
- [ ] Security audit limpio
- [ ] Documentación actualizada

### Versioning

- [ ] Version bumped en `Cargo.toml`
- [ ] `CHANGELOG.md` actualizado
- [ ] Release notes preparadas

### Testing

- [ ] Build local exitoso
- [ ] Tests E2E passing
- [ ] Performance benchmarks ejecutados
- [ ] Installers probados localmente

### Secrets

- [ ] `GITHUB_TOKEN` disponible (automático)
- [ ] Code signing secrets configurados (opcional)
- [ ] Tauri update keys configurados (opcional)

---

## 🐛 Troubleshooting

### "Validation failed"

**Problema**: Clippy warnings o formatting issues

**Solución**:
```bash
cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged --workspace --features surrealdb-metrics
git add .
git commit -m "fix: resolve linting issues"
git push
```

### "Build failed on Windows"

**Problema**: Dependencias faltantes o errores de compilación

**Solución**:
1. Revisar logs del workflow
2. Reproducir localmente: `cargo build --release --features surrealdb-metrics`
3. Corregir errores
4. Push y re-trigger workflow

### "Code signing failed"

**Problema**: Secrets incorrectos o certificado expirado

**Solución**:
1. Verificar que los secrets estén configurados
2. Verificar que el certificado no haya expirado
3. Re-generar certificado si es necesario
4. Actualizar secrets en GitHub

### "Release creation failed"

**Problema**: Tag ya existe o permisos insuficientes

**Solución**:
```bash
# Eliminar tag local y remoto
git tag -d v0.1.0
git push origin :refs/tags/v0.1.0

# Crear nuevo tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## 📈 Mejores Prácticas

### 1. Testing Antes del Release

```bash
# Ejecutar validación completa
.\scripts\validate-release.ps1

# Build local
cargo build --release --features surrealdb-metrics

# Tests E2E
.\scripts\run-e2e-tests.ps1
```

### 2. Release Notes

Mantener `CHANGELOG.md` actualizado:
```markdown
## [0.1.0] - 2025-11-01

### Added
- Initial release
- RPA automation engine
- Guardian monitoring system

### Fixed
- Bug fixes

### Changed
- Performance improvements
```

### 3. Versioning Consistente

Actualizar versión en todos los archivos:
- `Cargo.toml` (workspace)
- `src-tauri/Cargo.toml`
- `package.json`
- `CHANGELOG.md`

### 4. Pre-release Testing

Usar pre-releases para testing:
```bash
# Crear pre-release
git tag -a v0.1.0-beta.1 -m "Beta release"
git push origin v0.1.0-beta.1

# Marcar como pre-release en workflow
# O usar workflow_dispatch con prerelease: true
```

---

## 🔄 Workflow de Release Completo

```bash
# 1. Preparación
git checkout main
git pull origin main

# 2. Bump version
# Editar Cargo.toml, package.json, etc.

# 3. Update CHANGELOG
# Agregar cambios en CHANGELOG.md

# 4. Commit cambios
git add .
git commit -m "chore: bump version to 0.1.0"
git push origin main

# 5. Validación local
.\scripts\validate-release.ps1

# 6. Crear tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial public release"
git push origin v0.1.0

# 7. Monitorear workflow
# https://github.com/OxidePilot-Project/oxide-pilot/actions

# 8. Verificar release
# https://github.com/OxidePilot-Project/oxide-pilot/releases

# 9. Anunciar release
# Twitter, Reddit, Dev.to, etc.
```

---

## 📚 Referencias

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Tauri Build Documentation](https://tauri.app/v1/guides/building/)
- [Semantic Versioning](https://semver.org/)
- [Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-windows)

---

## 🎯 Próximos Pasos

Después de crear el release:

1. **Verificar installers**: Descargar y probar en cada plataforma
2. **Actualizar documentación**: Links de descarga, etc.
3. **Anunciar release**: Redes sociales, blog, etc.
4. **Monitorear issues**: Responder a problemas reportados
5. **Planear siguiente release**: Roadmap, features, etc.

---

**Maintained by**: Oxide Pilot Team
**Last Updated**: October 28, 2025

