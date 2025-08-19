# Oxide Pilot - Scripts de Lanzamiento

## Scripts Disponibles

### 🚀 Scripts de Lanzamiento Principal

#### `start-oxide-pilot.bat` (Recomendado)
Script principal mejorado que:
- Limpia procesos existentes
- Instala dependencias del frontend
- Inicia el servidor de desarrollo
- Verifica que el frontend esté listo
- Lanza la aplicación Tauri
- Limpia procesos al cerrar

#### `launch-oxide-pilot-complete.bat`
Script completo con verificaciones exhaustivas:
- Verificación de directorio correcto
- Instalación/actualización de dependencias
- Verificación de que el frontend esté funcionando
- Manejo de errores mejorado

#### `dev-quick.bat`
Script rápido para desarrollo:
- Verificación mínima de dependencias
- Lanzamiento rápido para desarrollo

### 🔧 Scripts de Diagnóstico

#### `diagnose-oxide.bat`
Verifica que todos los requisitos del sistema estén instalados:
- Node.js y npm
- Rust y Cargo
- Tauri CLI
- Estructura del proyecto
- Dependencias instaladas
- Puertos disponibles

#### `test-auth.bat`
Prueba específica del sistema de autenticación:
- Verifica Tauri CLI
- Instala dependencias si es necesario
- Lanza la app para probar autenticación
- Guía para verificar que no hay errores de IPC

#### `fix-syntax-errors.bat`
Repara errores de sintaxis comunes:
- Detiene servidores de desarrollo
- Informa sobre errores corregidos
- Reinicia el servidor frontend
- Útil después de cambios automáticos del IDE

### 🧹 Scripts de Mantenimiento

#### `cleanup-project.bat`
Limpia archivos temporales y basura del proyecto:
- Elimina archivos .profraw (profiling de Rust)
- Remueve archivos duplicados
- Limpia caché de build
- Organiza el proyecto

#### `restart-app.bat`
Reinicio rápido de la aplicación:
- Detiene procesos existentes
- Relanza la aplicación
- Útil durante desarrollo

## 🛠️ Solución de Problemas

### Error: "TAURI_IPC is not a function"
Este error indica que el backend de Tauri no está conectado correctamente.

**Solución:**
1. Ejecuta `diagnose-oxide.bat` para verificar requisitos
2. Usa `start-oxide-pilot.bat` que incluye verificaciones
3. Asegúrate de que el frontend esté corriendo en puerto 5173

### Error: "System not initialized"
El sistema Oxide no se ha inicializado correctamente.

**Solución:**
1. Verifica que todas las dependencias estén instaladas
2. Revisa los logs de la consola para errores específicos
3. Usa el script de diagnóstico

### Error: "Unexpected token" en Svelte
Error de sintaxis en componentes Svelte, usualmente por bloques try-catch duplicados.

**Solución:**
1. Ejecuta `fix-syntax-errors.bat`
2. O revisa manualmente el archivo GoogleAuthSetup.svelte
3. Busca bloques catch duplicados o mal formados

### Frontend no se conecta
El servidor de desarrollo del frontend no está funcionando.

**Solución:**
1. Verifica que el puerto 5173 esté libre
2. Ejecuta `cd src-frontend && npm install`
3. Usa `launch-oxide-pilot-complete.bat` que verifica la conexión

## 📋 Orden Recomendado de Uso

1. **Primera vez:** `diagnose-oxide.bat`
2. **Desarrollo normal:** `start-oxide-pilot.bat`
3. **Problemas de autenticación:** `test-auth.bat`
4. **Desarrollo rápido:** `dev-quick.bat`

## 🔍 Verificación de Funcionamiento

Cuando la aplicación funcione correctamente, deberías ver:
- ✅ Pantalla de "Google Gemini API Configuration"
- ✅ Dos pestañas: "API Key" y "OAuth 2.0"
- ✅ Sin errores de "TAURI_IPC" en la consola
- ✅ Botones funcionales para guardar credenciales

## 📞 Funciones Backend Disponibles

Las siguientes funciones están disponibles en el backend:
- `set_google_client_credentials` - Guardar credenciales OAuth
- `authenticate_google_command` - Iniciar flujo de autenticación
- `startup_check` - Verificar estado del sistema
- `initialize_auth_manager` - Inicializar gestor de autenticación

## 🎯 Próximos Pasos

Una vez que la autenticación funcione:
1. Configurar API key de Google Gemini
2. Probar la conexión con la API
3. Implementar funcionalidades de chat
4. Configurar otros módulos (voice, memory, etc.)