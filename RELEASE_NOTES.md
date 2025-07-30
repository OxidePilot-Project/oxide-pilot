# Oxide Pilot - Gemini Edition

## Release v1.0.0 - Gemini Edition

### 🎯 **Enfoque exclusivo en Google Gemini**
Esta versión está optimizada para funcionar exclusivamente con Google Gemini, simplificando la configuración y mejorando la experiencia de usuario.

### ✅ **Características Implementadas**

#### **1. Configuración Simplificada**
- **Solicitud de API Key al iniciar**: El sistema solicita automáticamente la clave API de Google Gemini al primer inicio
- **Configuración única**: No requiere configuración de múltiples proveedores de IA
- **Interfaz intuitiva**: Wizard de configuración integrado en la interfaz

#### **2. Sistema de Permisos RPA Completo**
- **Niveles de permiso**: Allow, Ask, Deny para cada tipo de acción
- **Tipos de acciones cubiertas**:
  - Mouse clicks y movimientos
  - Entrada de teclado
  - Captura de pantalla
  - Operaciones de archivo
  - Acceso a red
  - Comandos del sistema
- **Registro de auditoría**: Trazabilidad completa de todas las acciones
- **Confirmación de usuario**: Diálogos de confirmación para acciones sensibles

#### **3. Optimización de Rendimiento para Segundo Plano**
- **Límites de recursos**:
  - CPU: Máximo 5% en segundo plano
  - RAM: Máximo 256MB
- **Prioridad baja**: Proceso ejecutado con prioridad reducida
- **Detección de inactividad**: Optimización automática cuando el sistema está inactivo
- **Monitoreo en tiempo real**: Métricas de rendimiento continuas

#### **4. Seguridad Empresarial Completa**
- **Encriptación E2E**: AES-256-GCM para todas las comunicaciones sensibles
- **Control de acceso basado en roles**:
  - Admin: Control total del sistema
  - User: Monitoreo y ejecución básica
  - Readonly: Solo visualización
- **Gestión de permisos**: Sistema granular de autorización
- **Registro de seguridad**: Auditoría completa de eventos de seguridad

#### **5. Despliegue Empresarial**
- **Plantillas de política de grupo**: ADMX para configuración corporativa
- **Script de instalación automatizado**: Instalación silenciosa y configuración
- **Integración con Windows**: Servicio del sistema y políticas de grupo
- **Compatibilidad empresarial**: Firewall, PATH, y permisos corporativos

### 📦 **Archivos de Instalación**

#### **Para Usuarios Individuales**
- `install.bat`: Instalación simple para usuarios individuales
- `oxide-pilot.exe`: Ejecutable principal
- Configuración automática en `%USERPROFILE%\.oxidepilot`

#### **Para Empresas**
- `install-enterprise.bat`: Instalación corporativa con privilegios de administrador
- `group-policy-template.admx`: Plantillas para políticas de grupo
- Configuración centralizada en `%ProgramData%\OxidePilot`

### 🚀 **Cómo Instalar**

#### **Usuario Individual**
1. Ejecutar `install.bat`
2. Seguir el asistente de configuración
3. Ingresar la clave API de Google Gemini cuando se solicite

#### **Empresa**
1. Ejecutar como administrador `install-enterprise.bat`
2. Configurar políticas de grupo según necesidades corporativas
3. Distribuir mediante herramientas de despliegue corporativo

### 🔧 **Configuración de Google Gemini**

1. **Obtener API Key**:
   - Visitar: https://makersuite.google.com/app/apikey
   - Crear un nuevo proyecto o usar uno existente
   - Generar la clave API

2. **Configuración inicial**:
   - La aplicación solicitará la clave al primer inicio
   - La clave se almacena de forma segura
   - No se requiere configuración adicional

### 📋 **Requisitos del Sistema**

- **Sistema operativo**: Windows 10/11 (64-bit)
- **Memoria RAM**: 512MB mínimo, 1GB recomendado
- **Espacio en disco**: 100MB
- **Conexión a internet**: Requerida para Google Gemini API

### 🛡️ **Seguridad**

- **Encriptación**: Todos los datos sensibles están encriptados
- **Almacenamiento seguro**: Credenciales en Windows Credential Manager
- **Auditoría**: Registro completo de todas las acciones
- **Permisos**: Control granular de acceso a funciones del sistema

### 📞 **Soporte**

Para soporte empresarial o problemas de instalación, consultar:
- Documentación en `docs/`
- Logs en `%USERPROFILE%\.oxidepilot\logs\`
- Configuración en `%USERPROFILE%\.oxidepilot\config.json`

---

**Versión**: 1.0.0 - Gemini Edition  
**Fecha de release**: 28 de julio de 2025  
**Estado**: Listo para producción
