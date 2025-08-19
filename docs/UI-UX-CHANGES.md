---
description: Cambios de UI/UX recientes y guía de uso
---

# Cambios de UI/UX (Agosto 2025)

Este documento resume los cambios más recientes en la interfaz de Oxide Pilot y cómo utilizarlos. Las mejoras se centran en exponer funcionalidades de seguridad/sesiones y rendimiento/alertas del backend Tauri de forma profesional, mantenible y responsiva.

## Resumen

- Se añadieron dos componentes Svelte nuevos con integración a comandos Tauri:
  - `src-frontend/src/lib/components/SecurityCenter.svelte`
  - `src-frontend/src/lib/components/PerformanceAlertsPanel.svelte`
- Se actualizaron las pestañas del `Dashboard` en `PatternDashboard.svelte` para incrustar estos paneles.
- Se estandarizó el uso de `tauriInvoke` SSR-safe y la detección `isTauri`.
- Se reforzó la UI para tamaños de ventana soportados: fullscreen y mediana (1280x800).
- Animaciones: la app integra `anime.js` mediante Svelte actions reutilizables (`fadeIn`, `hoverLift`, `ripple`).
- Se creó una página de “showcase” en `/ui` para probar botones con ripple y variantes.

## Componentes nuevos

### SecurityCenter.svelte
Ubicación: `src-frontend/src/lib/components/SecurityCenter.svelte`

Funciones principales:
- Crear sesiones de seguridad.
- Validar sesiones y mostrar estado.
- Comprobar permisos específicos.
- Listado de eventos de seguridad con auto-actualización.

Comandos Tauri utilizados:
- `create_security_session(user_id, permissions, ip_address?, user_agent?)`
- `validate_security_session(session_id)`
- `check_security_permission(session_id, permission)`
- `get_security_events(limit)`

Notas de UI/UX:
- Campos con validación mínima y mensajes de estado/errores.
- Manejo “disabled” fuera de Tauri (`isTauri === false`).
- Diseño responsive sin scroll horizontal, legible en 1280x800.

### PerformanceAlertsPanel.svelte
Ubicación: `src-frontend/src/lib/components/PerformanceAlertsPanel.svelte`

Funciones principales:
- Mostrar alertas de rendimiento.
- Mostrar estadísticas de errores y lista de errores recientes.
- Mostrar perfiles de operaciones recolectados.
- Alternar monitoreo de rendimiento desde la UI.
- Limpiar alertas.

Comandos Tauri utilizados:
- `get_performance_alerts()`
- `get_error_statistics()`
- `get_recent_errors(limit)`
- `get_operation_profiles()`
- `set_performance_monitoring(enabled)`
- `clear_performance_alerts()`

Notas de UI/UX:
- Botones de refresco y limpiar con feedback visual.
- Sección de estadísticas con métricas clave.
- Listas con timestamp y payloads JSON inline.

## Integración en el Dashboard

Archivo: `src-frontend/src/lib/components/PatternDashboard.svelte`
- Pestaña “Security”: reemplaza el grid estático por `<SecurityCenter />`.
- Pestaña “Performance”: añade `<PerformanceAlertsPanel />` bajo el gráfico.

## Utilidades y arquitectura de UI

- `tauriInvoke`: Utilidad centralizada SSR-safe importada desde `$lib/utils/tauri`.
- `isTauri`: Detección del runtime para degradar funciones en navegador.
- Acciones `anime.js`: `fadeIn`, `hoverLift`, `ripple` disponibles y usadas en botones (`Button.svelte`).
- Página de pruebas UI: `/ui` para probar variantes, tamaños, y colores de ripple.

## Cómo usar (rápido)

1. Inicia el frontend:
   ```powershell
   cd src-frontend
   npm install
   npm run dev:5180
   ```
2. Abre `http://localhost:5180/app` y autentica (Gemini o Qwen) si es necesario.
3. Ve a Dashboard > Security/Performance para probar las funciones nuevas.
4. (Opcional) `http://localhost:5180/ui` para probar animaciones de botones.

## Accesibilidad y Responsividad

- Fuentes legibles, color tokens y contrastes revisados.
- Sin desbordes horizontales; `AppLayout.svelte` limita el scroll a `main`.
- Controles de ventana (Tauri): fullscreen y modo mediano (1280x800). Se recomienda deshabilitar “user resizing” desde Tauri para reforzar.

## Recomendaciones (próximos opcionales)

- Toasts/Alerts: Añadir sistema de notificaciones no intrusivo.
- Skeletons/Loading: Indicadores de carga en listas y cards.
- Paginación/Filtrado: Para eventos y errores recientes.
- Persistencia: Guardar el estado del monitoreo de rendimiento.
- Gráficas: Mini sparklines de métricas recientes.
- E2E: Pruebas Playwright para flujos de Security/Performance.
- Documentación: Actualizar `src-frontend/README.md` con estas guías y scripts reales.
- Tauri: Confirmar `resizable: false` o lógica equivalente en configuración de ventana.

## Historial de cambios

- 2025-08-14: Se agregan SecurityCenter y PerformanceAlertsPanel; integración en PatternDashboard; link en README; script de limpieza actualizado para opcionalmente eliminar `superdesign`.
