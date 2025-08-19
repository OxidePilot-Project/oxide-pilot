# Construir instalador Windows (MSI/EXE) para Oxide Pilot

Este proyecto usa Tauri para empaquetar el binario de Rust como instalador Windows.

## Requisitos

- Windows 10/11 x64
- Rust toolchain (https://rustup.rs/)
- Tauri CLI: `cargo install tauri-cli`
- Para MSI: WiX Toolset 3.11+ (asegúrate que `candle.exe` y `light.exe` estén en el PATH)
- Para NSIS: NSIS (asegúrate que `makensis.exe` esté en el PATH)
- Node.js solo es necesario si integras un frontend web que requiera build (no obligatorio para este backend Tauri)

## Configuración de Tauri

El archivo `src-tauri/tauri.conf.json` ya define:
- productName, identifier
- targets: `nsis` y `msi`
- ventana principal básica

Si necesitas firmar el ejecutable, agrega tu configuración de firma de código según la documentación de Tauri.

## Pasos rápidos

1) Opcional: habilitar Cognee en el binario (usa `.env`):
   - Copia `src-tauri/.env.example` a `src-tauri/.env`
   - Ajusta `OXIDE_COGNEE_ENABLE=true` y define `OXIDE_COGNEE_URL` y `OXIDE_COGNEE_TOKEN` si usarás el sidecar

2) Ejecutar el build del instalador:

- PowerShell:

```
# Por defecto genera ambos targets según configuración de Tauri
pwsh -File scripts/build-windows.ps1
```

Los artefactos se generan en `src-tauri/target/release/` (carpeta `bundle`).

## Ajustar targets

Por defecto, Tauri intentará construir los targets configurados. Puedes controlar los objetivos con variables de entorno, por ejemplo:

- Solo MSI:
```
$env:TAURI_BUNDLE_TARGETS="msi"
```
- Solo NSIS:
```
$env:TAURI_BUNDLE_TARGETS="nsis"
```

Luego ejecuta `cargo tauri build` desde `src-tauri` o usa el script.

## Firma de código (opcional pero recomendada)

Variables de entorno esperadas por el script (si quieres firmar automáticamente tras el build):

- SIGNTOOL: ruta a signtool.exe (por ejemplo: C:\\Program Files (x86)\\Windows Kits\\10\\App Certification Kit\\signtool.exe)
- SIGN_CERT: ruta al certificado PFX
- SIGN_PASS: contraseña del PFX
- SIGN_TS_URL: URL del servidor de timestamp (ej: http://timestamp.digicert.com)

Ejemplo (PowerShell):

```
$env:SIGNTOOL = "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe"
$env:SIGN_CERT = "C:\keys\codesign.pfx"
$env:SIGN_PASS = "{{CERT_PASSWORD}}"
$env:SIGN_TS_URL = "http://timestamp.digicert.com"
pwsh -File scripts/build-windows.ps1
```

## Notas de producción

- Firma de código recomendada para evitar advertencias de SmartScreen.
- Si usas el sidecar de Cognee, evalúa empaquetarlo por separado o proveerlo como dependencia opcional (este repo actualmente lo trata como componente externo opcional).
- Actualiza el `version` en `src-tauri/tauri.conf.json` y/o usa el parámetro `-Version` del script.
