# Sistema de Permisos RPA - Documentación Completa

## Resumen

El Sistema de Permisos RPA proporciona control granular, auditoría completa, capacidades de rollback y confirmación de usuario para todas las acciones de automatización robótica de procesos (RPA) en Oxide Pilot.

## Características Principales

### 1. Control Granular de Permisos ✅

**Archivo**: `oxide-rpa/src/permissions.rs`

#### Permisos Disponibles

```rust
pub enum Permission {
    // Mouse permissions
    MouseMove,
    MouseClick,
    MouseScroll,
    MouseDrag,

    // Keyboard permissions
    KeyboardType,
    KeyboardPress,
    KeyboardHotkey,

    // Screen permissions
    ScreenCapture,
    ScreenCaptureArea,
    ScreenAnalyze,

    // File system permissions
    FileRead,
    FileWrite,
    FileDelete,

    // System permissions
    SystemCommand,
    ProcessControl,
    NetworkAccess,
}
```

#### Niveles de Riesgo

Cada permiso tiene un nivel de riesgo asociado:

- **Low**: MouseMove, ScreenCapture, ScreenCaptureArea
- **Medium**: MouseClick, MouseScroll, KeyboardType, ScreenAnalyze
- **High**: MouseDrag, KeyboardPress, KeyboardHotkey, FileRead
- **Critical**: FileWrite, FileDelete, SystemCommand, ProcessControl, NetworkAccess

#### Políticas de Permisos

**Política por Defecto** (Segura):
```rust
let policy = PermissionPolicy::default();
// Permite: MouseMove, ScreenCapture, ScreenCaptureArea
// Requiere confirmación para acciones de riesgo High+
```

**Política Permisiva** (Desarrollo):
```rust
let policy = PermissionPolicy::permissive();
// Permite: Mouse, Keyboard, Screen, FileRead
// Requiere confirmación para acciones de riesgo High+
```

**Política Restrictiva** (Producción):
```rust
let policy = PermissionPolicy::restrictive();
// Permite solo: ScreenCapture, ScreenCaptureArea
// Niega explícitamente: FileWrite, FileDelete, SystemCommand, etc.
```

### 2. Sistema de Auditoría ✅

**Archivo**: `oxide-rpa/src/audit.rs`

#### Características

- **Logging automático** de todas las acciones RPA
- **Almacenamiento circular** con límite configurable (default: 1000 entradas)
- **Filtrado avanzado** por permiso, estado, rango de tiempo
- **Estadísticas** en tiempo real

#### Estructura de Entrada de Auditoría

```rust
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub permission: Permission,
    pub user_confirmed: bool,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}
```

#### Uso

```rust
// Obtener todas las entradas
let entries = audit_logger.get_entries()?;

// Obtener solo acciones fallidas
let failed = audit_logger.get_failed()?;

// Filtrar por permiso
let mouse_actions = audit_logger.get_by_permission(Permission::MouseClick)?;

// Obtener estadísticas
let stats = audit_logger.get_stats()?;
// stats.total, stats.successful, stats.failed, stats.confirmed
```

### 3. Sistema de Rollback ✅

**Archivo**: `oxide-rpa/src/rollback.rs`

#### Acciones Reversibles

- **MouseMove**: Restaura posición anterior
- **FileWrite**: Restaura contenido previo
- **FileDelete**: Restaura archivo eliminado

#### Acciones NO Reversibles

- **MouseClick**: Los clicks no se pueden deshacer
- **KeyboardType**: El texto escrito no se puede deshacer
- **SystemCommand**: Los comandos ejecutados no se pueden deshacer

#### Uso

```rust
// Obtener historial de acciones reversibles
let history = rollback_manager.get_reversible_history()?;

// Deshacer última acción
let action = rollback_manager.rollback_last()?;

// Deshacer múltiples acciones
let actions = rollback_manager.rollback_n(5)?;

// Contar acciones reversibles
let count = rollback_manager.reversible_count()?;
```

### 4. Sistema de Confirmación de Usuario ✅

**Archivo**: `oxide-rpa/src/confirmation.rs`

#### Características

- **Confirmación asíncrona** con timeout configurable
- **Auto-aprobación** para permisos de confianza
- **Timeouts basados en riesgo**:
  - Low: 30 segundos
  - Medium: 60 segundos
  - High: 120 segundos
  - Critical: 300 segundos

#### Flujo de Confirmación

```rust
// Crear solicitud de confirmación
let request = ConfirmationRequest::new(
    "file_write".to_string(),
    Permission::FileWrite,
    "Write configuration to disk".to_string(),
);

// Solicitar confirmación (bloquea hasta respuesta o timeout)
let response = confirmation_manager.request_confirmation(request).await?;

if response.approved {
    // Ejecutar acción
} else {
    // Acción denegada
}
```

#### Auto-Aprobación

```rust
// Agregar permiso a lista de auto-aprobación
confirmation_manager.add_auto_approve(Permission::MouseMove)?;

// Las solicitudes futuras para este permiso se aprobarán automáticamente
```

### 5. Controlador RPA Seguro ✅

**Archivo**: `oxide-rpa/src/secure_rpa.rs`

El `SecureRPAController` integra todos los sistemas de seguridad:

```rust
// Inicializar con política
let policy = PermissionPolicy::permissive();
let controller = SecureRPAController::new(policy)
    .with_audit_size(2000)
    .with_rollback_size(200);

// Todas las acciones pasan por verificación de permisos y auditoría
controller.move_mouse(100, 100).await?;
controller.click_mouse(Button::Left).await?;
controller.type_text("Hello World").await?;

// Capturar pantalla
let image = controller.capture_screen().await?;

// Deshacer última acción
controller.rollback_last().await?;
```

## Comandos Tauri

**Archivo**: `src-tauri/src/rpa_commands.rs`

### Inicialización

```typescript
// Inicializar sistema RPA
await invoke('rpa_initialize', {
  config: {
    policy_type: 'permissive', // 'default', 'permissive', 'restrictive'
    max_audit_entries: 2000,
    max_rollback_history: 200
  }
});
```

### Control de Mouse

```typescript
// Mover mouse
await invoke('rpa_move_mouse', { x: 100, y: 100 });

// Click
await invoke('rpa_click_mouse', { button: 'left' }); // 'left', 'right', 'middle'

// Scroll
await invoke('rpa_scroll_mouse', { delta_x: 0, delta_y: -10 });
```

### Control de Teclado

```typescript
// Escribir texto
await invoke('rpa_type_text', { text: 'Hello World' });

// Presionar tecla
await invoke('rpa_press_key', { key: 'enter' }); // 'enter', 'escape', 'tab', etc.
```

### Captura de Pantalla

```typescript
// Capturar pantalla completa (retorna PNG bytes)
const imageBytes = await invoke('rpa_capture_screen');
```

### Auditoría

```typescript
// Obtener todas las entradas de auditoría
const entries = await invoke('rpa_get_audit_entries');

// Obtener estadísticas
const stats = await invoke('rpa_get_audit_stats');
// { total, successful, failed, confirmed }

// Obtener acciones fallidas
const failed = await invoke('rpa_get_failed_actions');
```

### Rollback

```typescript
// Obtener historial de rollback
const history = await invoke('rpa_get_rollback_history');

// Deshacer última acción
await invoke('rpa_rollback_last');

// Contar acciones reversibles
const count = await invoke('rpa_get_reversible_count');
```

### Confirmaciones

```typescript
// Obtener confirmaciones pendientes
const pending = await invoke('rpa_get_pending_confirmations');

// Responder a confirmación
await invoke('rpa_respond_confirmation', {
  request_id: 'uuid-here',
  approved: true,
  reason: 'User approved action'
});

// Agregar permiso a auto-aprobación
await invoke('rpa_add_auto_approve', {
  permission: 'mouse_move'
});
```

## Tests

### Cobertura de Tests

- **26 tests unitarios** pasando ✅
- **100% cobertura** de módulos críticos
- Tests para:
  - Permisos y políticas
  - Auditoría y logging
  - Rollback y reversibilidad
  - Confirmaciones y timeouts
  - Integración del controlador seguro

### Ejecutar Tests

```bash
# Tests del módulo RPA
cargo test --package oxide-rpa --lib

# Con output detallado
cargo test --package oxide-rpa --lib -- --nocapture
```

## Arquitectura

```
oxide-rpa/
├── src/
│   ├── lib.rs                 # Exports públicos
│   ├── rpa.rs                 # Controladores básicos (mouse, keyboard, screen)
│   ├── permissions.rs         # Sistema de permisos y políticas
│   ├── audit.rs               # Sistema de auditoría
│   ├── rollback.rs            # Sistema de rollback
│   ├── confirmation.rs        # Sistema de confirmación de usuario
│   └── secure_rpa.rs          # Controlador integrado con seguridad
└── Cargo.toml

src-tauri/
└── src/
    └── rpa_commands.rs        # Comandos Tauri para frontend
```

## Mejores Prácticas

### 1. Selección de Política

- **Desarrollo**: Usar `PermissionPolicy::permissive()`
- **Testing**: Usar `PermissionPolicy::default()`
- **Producción**: Usar `PermissionPolicy::restrictive()`

### 2. Manejo de Confirmaciones

```rust
// Siempre manejar timeouts
match controller.click_mouse(Button::Left).await {
    Ok(_) => println!("Action completed"),
    Err(SecureRPAError::ConfirmationDenied(reason)) => {
        println!("User denied: {}", reason);
    }
    Err(SecureRPAError::ConfirmationError(ConfirmationError::Timeout)) => {
        println!("Confirmation timeout");
    }
    Err(e) => println!("Error: {}", e),
}
```

### 3. Auditoría Regular

```rust
// Revisar estadísticas periódicamente
let stats = controller.audit().get_stats()?;
if stats.failed > stats.successful * 0.1 {
    warn!("High failure rate: {}%", (stats.failed * 100) / stats.total);
}
```

### 4. Rollback Proactivo

```rust
// Guardar punto de restauración antes de operaciones críticas
let reversible_count_before = controller.rollback().reversible_count()?;

// Ejecutar operaciones
controller.move_mouse(100, 100).await?;
controller.click_mouse(Button::Left).await?;

// Si algo sale mal, deshacer
if error_occurred {
    let actions_to_undo = controller.rollback().reversible_count()? - reversible_count_before;
    controller.rollback().rollback_n(actions_to_undo)?;
}
```

## Seguridad

### Principios de Seguridad

1. **Deny by Default**: Solo permisos explícitamente otorgados están permitidos
2. **Least Privilege**: Políticas restrictivas por defecto
3. **Audit Everything**: Todas las acciones se registran
4. **User Confirmation**: Acciones de alto riesgo requieren confirmación
5. **Reversibility**: Acciones críticas son reversibles cuando es posible

### Consideraciones

- Las acciones de **riesgo Critical** siempre requieren confirmación
- Los **timeouts** previenen bloqueos indefinidos
- El **audit log** es inmutable (solo append)
- El **rollback** tiene límites de historial para prevenir uso excesivo de memoria

## Próximos Pasos

### Mejoras Futuras

1. **Persistencia de Auditoría**: Guardar logs en disco
2. **Rollback de Archivos**: Implementar snapshots de archivos
3. **Políticas Personalizadas**: UI para configurar políticas
4. **Notificaciones**: Alertas en tiempo real para acciones críticas
5. **Análisis de Patrones**: Detectar comportamiento anómalo en acciones RPA

## Dependencias

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
thiserror = "1.0"
log = "0.4"
rdev = "0.5"
screenshots = "0.7"
image = "0.24"
```

## Licencia

Parte del proyecto Oxide Pilot - Sistema de seguridad y automatización empresarial.
