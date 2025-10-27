# Configuración de Variables de Entorno - Oxide Pilot

Este documento explica cómo configurar las variables de entorno necesarias para que Oxide Pilot funcione correctamente, basado en las mejores prácticas de OAuth2 Device Flow.

## Archivo de Configuración

Crea un archivo `.env` en la carpeta `src-tauri/` con las siguientes variables:

```bash
# =============================================================================
# GOOGLE GEMINI API CONFIGURATION
# =============================================================================
# Obtén tu API key desde: https://aistudio.google.com/apikey
GEMINI_API_KEY=tu_clave_api_de_google_gemini_aqui

# =============================================================================
# QWEN OAUTH2 DEVICE FLOW CONFIGURATION
# =============================================================================
# Basado en OAuth2 Device Authorization Grant (RFC 8628)
# Basado en ejemplos de: https://github.com/QwenLM/qwen-code

# Device Authorization Endpoint
# Este endpoint se usa para iniciar el flujo de autorización del dispositivo
QWEN_DEVICE_AUTH_URL=https://tu-proveedor-qwen.com/oauth2/device/code

# Token Endpoint
# Este endpoint se usa para sondear el token de acceso
QWEN_DEVICE_TOKEN_URL=https://tu-proveedor-qwen.com/oauth2/token

# Credenciales OAuth2 del Cliente
# Registra tu aplicación con el proveedor OAuth de Qwen para obtener estos
QWEN_CLIENT_ID=tu_client_id_de_qwen
QWEN_CLIENT_SECRET=tu_client_secret_de_qwen

# Alcances OAuth2
# Lista separada por comas de los alcances que tu aplicación necesita
QWEN_SCOPE=openid,profile,email

# =============================================================================
# LOCAL LLM CONFIGURATION (Opcional)
# =============================================================================
# Para ejecutar modelos de IA localmente usando LM Studio o similar
LOCAL_LLM_BASE_URL=http://localhost:11434
LOCAL_LLM_MODEL=llama2
LOCAL_LLM_TIMEOUT=30000

# =============================================================================
# SYSTEM CONFIGURATION
# =============================================================================
# Nivel de logging: trace, debug, info, warn, error
OXIDE_LOG_LEVEL=info

# Directorio de datos para almacenar datos de la aplicación
OXIDE_DATA_DIR=./data

# =============================================================================
# CONFIGURACIÓN DE DESARROLLO
# =============================================================================
# Establece en 'true' para habilitar características de modo desarrollo
OXIDE_DEV_MODE=false

# Endpoints OAuth2 simulados para pruebas (solo desarrollo)
# QWEN_DEVICE_AUTH_URL=https://demo.oauth2.dev/device/code
# QWEN_DEVICE_TOKEN_URL=https://demo.oauth2.dev/token
# QWEN_CLIENT_ID=demo_client_id
# QWEN_CLIENT_SECRET=demo_client_secret
```

## Configuración por Proveedor

### Google Gemini
1. Ve a [Google AI Studio](https://aistudio.google.com/apikey)
2. Crea una nueva API key
3. Copia la clave y pégala en `GEMINI_API_KEY`

### Qwen AI
1. Registra tu aplicación en el proveedor de Qwen
2. Obtén tu `CLIENT_ID` y `CLIENT_SECRET`
3. Configura las URLs de los endpoints OAuth2
4. Añade las variables al archivo `.env`

### Modelos Locales (LM Studio)
1. Instala y ejecuta LM Studio
2. Configura el puerto (por defecto 11434)
3. Ajusta `LOCAL_LLM_BASE_URL` si es necesario

## Solución de Problemas

### Error: "Missing env var: QWEN_DEVICE_AUTH_URL"
Este error indica que las variables de entorno de Qwen no están configuradas. Para solucionarlo:

1. Crea el archivo `src-tauri/.env`
2. Añade las variables de Qwen como se muestra arriba
3. Reinicia la aplicación

### Error: "Desktop application required"
Este error aparece cuando intentas usar funciones que requieren Tauri en el navegador. Usa la aplicación de escritorio en su lugar.

## Seguridad

⚠️ **Importante**:
- Nunca subas el archivo `.env` al repositorio
- Mantén tus claves API seguras
- Usa variables de entorno en producción

## Configuración Automática

### Scripts de Configuración

Para facilitar la configuración, hemos incluido scripts que crean automáticamente el archivo `.env`:

#### Windows (PowerShell)
```powershell
# Ejecutar en modo interactivo
.\scripts\setup-env.ps1 -Interactive

# O con parámetros específicos
.\scripts\setup-env.ps1 -GeminiApiKey "tu_clave_aqui" -QwenClientId "tu_client_id"
```

#### Linux/macOS (Bash)
```bash
# Crear archivo .env básico
./scripts/setup-env.sh

# Luego editar manualmente el archivo generado
nano src-tauri/.env
```

### Configuración Manual

Si prefieres configurar manualmente:

1. Copia el contenido del ejemplo de arriba
2. Crea el archivo `src-tauri/.env`
3. Reemplaza los valores de ejemplo con tus credenciales reales

## Verificación

Para verificar que la configuración funciona:

1. Inicia la aplicación
2. Ve a la pestaña de configuración
3. Verifica que no aparezcan errores de variables de entorno
4. Prueba la autenticación con cada proveedor

## Referencias

- [OAuth2 Device Authorization Grant (RFC 8628)](https://tools.ietf.org/html/rfc8628)
- [Qwen OAuth2 Implementation Examples](https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/qwen/qwenOAuth2.test.ts)
- [Google AI Studio API Keys](https://aistudio.google.com/apikey)
