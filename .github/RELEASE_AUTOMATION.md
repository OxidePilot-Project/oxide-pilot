# Release Automation System

Este documento describe el sistema automatizado de releases de Oxide Pilot, inspirado en [termux/termux-packages](https://github.com/termux/termux-packages/releases).

## Funcionamiento

### Triggers Automáticos

El sistema de releases se activa automáticamente en dos escenarios:

1. **Push a la rama `main`**
   - Genera una release automática con etiqueta `bootstrap-TIMESTAMP-COMMIT`
   - Marcada como **pre-release**
   - Permite descargar builds de desarrollo

2. **Tags de versión (`v*` o `bootstrap-*`)**
   - Tags `v*.*.*` generan releases **estables**
   - Tags `bootstrap-*` generan releases de desarrollo
   - Incluye notas de release automáticas

### Generación de Etiquetas

Las etiquetas se generan automáticamente:

```powershell
# Formato automático para push a main
bootstrap-YYYYMMDD-HHmmss-{commit-short}
# Ejemplo: bootstrap-20251026-143022-7bab01b

# Formato manual con tag
v1.0.0                    # Release estable
bootstrap-custom-build    # Build personalizada
```

### Proceso de Build

1. **Checkout del código** con historial completo
2. **Setup del entorno**: Rust, Node.js, Python
3. **Instalación de herramientas**: Tauri CLI, NSIS, WiX
4. **Generación de iconos** (si existe `src-tauri/icon.png`)
5. **Build del frontend** Svelte
6. **Bundle de Tauri** (instaladores NSIS y MSI)
7. **Firma de código** (opcional, si hay secretos configurados)
8. **Generación de checksums** (MD5, SHA256, SHA512)
9. **Creación de GitHub Release** con artefactos

## Artefactos Generados

Cada release incluye:

### Instaladores
- `oxide-pilot-{version}_x64-setup.exe` - Instalador NSIS (Windows)
- `oxide-pilot-{version}_x64.msi` - Instalador MSI (Windows)

### Checksums
- `CHECKSUMS-md5.txt` - Hashes MD5 de todos los binarios
- `CHECKSUMS-sha256.txt` - Hashes SHA256 de todos los binarios
- `CHECKSUMS-sha512.txt` - Hashes SHA512 de todos los binarios

### Metadatos
- Notas de release automáticas desde commits
- Link al changelog completo
- Información de versión y commit

## Verificación de Integridad

Los usuarios pueden verificar la integridad de los archivos descargados:

```powershell
# Windows PowerShell - Verificar SHA256
$expectedHash = (Get-Content CHECKSUMS-sha256.txt | Select-String "oxide-pilot").Line.Split()[0]
$actualHash = (Get-FileHash oxide-pilot-setup.exe -Algorithm SHA256).Hash.ToLower()
if ($expectedHash -eq $actualHash) { 
    Write-Host "✓ Checksum verificado correctamente" -ForegroundColor Green 
} else { 
    Write-Host "✗ Checksum NO coincide - Archivo corrupto o manipulado" -ForegroundColor Red 
}
```

```bash
# Linux/macOS - Verificar SHA256
sha256sum -c CHECKSUMS-sha256.txt
# ✓ oxide-pilot-setup.exe: OK
```

## Firma de Código (Opcional)

El workflow soporta firma de código automática si se configuran los siguientes secretos en GitHub:

- `SIGN_PFX_BASE64` - Certificado PFX codificado en Base64
- `SIGN_PFX_PASSWORD` - Contraseña del certificado
- `SIGN_TS_URL` - URL del servidor de timestamp (opcional)

### Configuración de Secretos

```bash
# Codificar certificado a Base64
base64 -w 0 certificate.pfx > certificate.txt

# En GitHub: Settings > Secrets and variables > Actions
# New repository secret:
#   - Name: SIGN_PFX_BASE64
#   - Value: [contenido de certificate.txt]
```

## Crear una Release Manual

### Release Estable (Versionada)

```bash
# 1. Actualizar versión en src-tauri/Cargo.toml
version = "1.0.0"

# 2. Commit y push
git add src-tauri/Cargo.toml
git commit -m "chore: bump version to 1.0.0"
git push origin main

# 3. Crear y push tag
git tag v1.0.0
git push origin v1.0.0

# 4. GitHub Actions automáticamente:
#    - Construye el proyecto
#    - Genera instaladores
#    - Crea release con tag v1.0.0
#    - Marca como release estable
```

### Release de Desarrollo (Bootstrap)

```bash
# Simplemente push a main - automático
git add .
git commit -m "feat: nueva funcionalidad experimental"
git push origin main

# GitHub Actions automáticamente:
#    - Genera tag bootstrap-TIMESTAMP-COMMIT
#    - Construye el proyecto
#    - Crea pre-release para testing
```

### Release Personalizada

```bash
# Crear tag personalizado
git tag bootstrap-feature-xyz
git push origin bootstrap-feature-xyz

# Se genera release con ese nombre
```

## Estructura de una Release

Cada release en GitHub contiene:

```
Oxide Pilot 0.1.0 (bootstrap-20251026-143022-7bab01b)
├── 📦 Assets
│   ├── oxide-pilot-0.1.0_x64-setup.exe    (28.5 MB)
│   ├── oxide-pilot-0.1.0_x64.msi          (28.4 MB)
│   ├── CHECKSUMS-md5.txt                   (212 bytes)
│   ├── CHECKSUMS-sha256.txt                (340 bytes)
│   └── CHECKSUMS-sha512.txt                (596 bytes)
├── 📝 Release Notes
│   ├── Version info
│   ├── Installation instructions
│   ├── Verification steps
│   └── Changelog link
└── 🏷️ Metadata
    ├── Tag: bootstrap-20251026-143022-7bab01b
    ├── Commit: 7bab01b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8
    └── Pre-release: true (if from main)
```

## Ventajas del Sistema

1. **Automatización Total**: Sin intervención manual para builds de desarrollo
2. **Trazabilidad**: Cada release vinculada a un commit específico
3. **Verificación**: Múltiples checksums para integridad
4. **Seguridad**: Firma de código opcional
5. **Distribución**: Descarga directa desde GitHub
6. **Versionado Claro**: Distinción entre releases estables y de desarrollo

## Comparación con Termux

| Característica | Termux | Oxide Pilot |
|----------------|--------|-------------|
| Triggers automáticos | ✓ Tags | ✓ Tags + Push a main |
| Pre-releases | ✓ | ✓ |
| Checksums | MD5, SHA256, SHA512 | MD5, SHA256, SHA512 |
| Firma de código | ✗ | ✓ (opcional) |
| Notas automáticas | ✓ | ✓ |
| Multi-plataforma | ✓ Android APKs | ✓ Windows (futuro: macOS, Linux) |

## Roadmap

- [ ] Soporte para macOS (DMG, APP)
- [ ] Soporte para Linux (AppImage, DEB, RPM)
- [ ] Builds para arquitecturas ARM64
- [ ] Delta updates (solo cambios incrementales)
- [ ] Auto-update desde la aplicación
- [ ] Releases firmadas con GPG
- [ ] Mirror CDN para descargas rápidas

## Troubleshooting

### El workflow falla en la firma de código

**Problema**: Error al firmar binarios
**Solución**: Verifica que los secretos estén configurados correctamente y que el certificado sea válido.

### Checksums no coinciden

**Problema**: Los checksums descargados no coinciden
**Solución**: Descarga nuevamente el archivo o prueba desde otra red. Podría ser corrupción durante la descarga.

### No se genera la release automáticamente

**Problema**: Push a main no crea release
**Solución**: Verifica que GitHub Actions esté habilitado y que el workflow tenga permisos de escritura en `contents`.

## Referencias

- [GitHub Actions - softprops/action-gh-release](https://github.com/softprops/action-gh-release)
- [Tauri - Building](https://tauri.app/v1/guides/building/)
- [Termux Packages Releases](https://github.com/termux/termux-packages/releases)
- [Semantic Versioning](https://semver.org/)
