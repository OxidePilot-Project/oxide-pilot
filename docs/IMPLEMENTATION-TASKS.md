# Plan de Implementación Final (100%)

Este documento segmenta las tareas restantes para alcanzar el 100% de readiness. Cada sección incluye objetivo, acciones y estado.

## 1. Endurecimiento de Seguridad Tauri (Hecho  ✅)
- Objetivo: Minimizar superficie de APIs y definir metadatos de bundle.
- Acciones:
  - Allowlist mínima en `src-tauri/tauri.conf.json` (app, window, dialog, fs limitada, http, path, notification, shell.open).
  - Metadatos (publisher, copyright).
- Estado: Aplicado.

## 2. UI Básica de Verificación (Hecho  ✅)
- Objetivo: Validar conexión UI↔Backend y estado de autenticación.
- Acciones:
  - `src-frontend/index.html` con botones para `startup_check` y `send_message_to_gemini`.
  - `tauri.conf.json` apunta a `../src-frontend/dist` (build automático).
- Estado: Aplicado.

## 3. Build Windows con Firma Opcional (Hecho  ✅)
- Objetivo: Generar MSI/NSIS y permitir firma de código.
- Acciones:
  - Script `scripts/build-windows.ps1` construye frontend + bundle; firma si hay vars: SIGNTOOL, SIGN_CERT, SIGN_PASS, SIGN_TS_URL.
  - Documentación en `README-WINDOWS-BUILD.md`.
- Estado: Aplicado (requiere tu certificado para firma real).

## 4. Flujo Dev con Sidecar (Hecho  ✅)
- Objetivo: Facilitar dev con memoria Cognee opcional.
- Acciones:
  - `scripts/dev-up.ps1` con flags `-UseCognee` y `-StartSidecar`.
  - `scripts/setup-cognee-sidecar.ps1` prepara venv, instala deps, crea `.env` e inicia uvicorn.
- Estado: Aplicado.

## 5. CI Windows (Hecho  ✅)
- Objetivo: Asegurar builds y tests en Windows.
- Acciones:
  - Workflow `/.github/workflows/windows-ci.yml`: tests Rust, build frontend, bundle Tauri, artifacts.
- Estado: Aplicado.

## 6. Iconos / Branding (Configurado  ✅, a falta del icono corporativo)
- Objetivo: Identidad visual consistente en instaladores/ventanas.
- Acciones realizadas:
  - Pipeline de generación de iconos: `scripts/build-windows.ps1` ejecuta `src-tauri/create_icon.py` si existe `src-tauri/icon.png`.
  - CI genera iconos si `src-tauri/icon.png` está presente.
  - Documentado en `src-tauri/icons/README.md`.
- Próximo paso:
  - Proveer `src-tauri/icon.png` (1024x1024).
- Objetivo: Identidad visual consistente en instaladores/ventanas.
- Acciones:
  - Preparar iconos en `src-tauri/icons/` (PNG 1024x1024 recomendado). Opcional: usar `create_icon.py` para generar variantes.
  - Actualizar `tauri.conf.json` para incluir iconos si se decide (no obligatorio para funcionalidad).
- Pasos siguientes:
  - Proveer `icon.png` corporativo o autorizar un placeholder.
  - Integración en bundle.

## 7. Firma de Código Real (Listo para integrar  ✅)
- Objetivo: Evitar advertencias de SmartScreen y mejorar confianza.
- Acciones realizadas:
  - Script `scripts/build-windows.ps1` firma EXE/MSI automáticamente si define env: SIGNTOOL, SIGN_CERT, SIGN_PASS, SIGN_TS_URL.
  - Documentación en `README-WINDOWS-BUILD.md`.
- Próximo paso:
  - Proveer PFX y contraseña (local o como secretos de CI).
- Objetivo: Evitar advertencias de SmartScreen y mejorar confianza.
- Acciones:
  - Proveer certificado PFX + contraseña.
  - Configurar entorno CI/Local con las variables y validar instaladores firmados.
- Pasos siguientes:
  - Entregar PFX o definir estrategia alternativa (sign service).

## 8. UI Svelte Ampliada (Opcional  🎯)
- Objetivo: Experiencia de usuario completa.
- Acciones sugeridas:
  - Vistas: Dashboard, Configuración (claves/API, toggles), Estado del sistema, Memoria.
  - Integrar `@tauri-apps/api` y componentes.
- Estimación: 1–2 sprints ligeros.

## 9. Sidecar Cognee: Packaging (Opcional  🎯)
- Objetivo: Facilitar uso por usuarios finales sin Python preinstalado.
- Opciones:
  - (A) Mantenerlo opcional con guía.
  - (B) Script de post-instalación que prepara venv.
  - (C) Distribución con Python embebido (más compleja).
- Recomendación: Mantener opcional para primera release; evaluar telemetry de adopción.

## 10. Publicación Artefactos (Opcional  🎯)
- Objetivo: Distribución automatizada.
- Acciones:
  - Añadir job CI de release (GitHub Actions) que suba a Releases los bundles.
  - Firma previa.

---

## Checklist de Cierre
- [x] Tauri endurecido
- [x] UI mínima de verificación
- [x] Build Win con firma opcional
- [x] Dev Up + Sidecar helper
- [x] CI Windows (tests+bundle)
- [ ] Iconos/branding integrados
- [ ] Firma de código real validada
- [ ] (Opcional) UI completa
- [ ] (Opcional) Estrategia de packaging de sidecar

Cuando tengamos iconos y certificado, el readiness sube al 100%.
