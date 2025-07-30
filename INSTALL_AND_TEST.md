# Guía de Instalación y Pruebas de Oxide Pilot - Gemini Edition

## 🛠️ Instalación del Entorno de Desarrollo

### 1. Instalar Rust

1.1. Visita [https://rustup.rs/](https://rustup.rs/) y descarga `rustup-init.exe`

1.2. Ejecuta el instalador y sigue las instrucciones en pantalla

1.3. Reinicia tu terminal o abre una nueva ventana de comandos

1.4. Verifica la instalación:
```bash
rustc --version
cargo --version
```

### 2. Instalar Node.js y npm

2.1. Descarga Node.js desde [https://nodejs.org/](https://nodejs.org/) (versión LTS recomendada)

2.2. Ejecuta el instalador y sigue las instrucciones

2.3. Verifica la instalación:
```bash
node --version
npm --version
```

### 3. Instalar Dependencias del Proyecto

3.1. Navega al directorio del frontend:
```bash
cd src-frontend
```

3.2. Instala las dependencias de Node.js:
```bash
npm install
```

3.3. Verifica que Tauri CLI esté instalado:
```bash
npm install -g @tauri-apps/cli
```

## 🧪 Pruebas de la Implementación

### 1. Ejecutar Pruebas Unitarias

1.1. Desde la raíz del proyecto:
```bash
cargo test --workspace
```

1.2. Verifica que todas las pruebas pasen sin errores

### 2. Construir y Probar el Frontend

2.1. Navega al directorio del frontend:
```bash
cd src-frontend
```

2.2. Construye el frontend:
```bash
npm run build
```

2.3. Verifica que la construcción sea exitosa

### 3. Construir el Backend

3.1. Navega al directorio de Tauri:
```bash
cd src-tauri
```

3.2. Construye el backend:
```bash
cargo build
```

### 4. Ejecutar Pruebas de Integración

4.1. Desde la raíz del proyecto, ejecuta el script de pruebas:
```bash
tests\test_gemini_integration.bat
```

4.2. Sigue las instrucciones del script para proporcionar tu clave API de Gemini

## 🔧 Configuración para Desarrollo

### 1. Configurar Variables de Entorno

Crea un archivo `.env` en la raíz del proyecto con tu clave API de Gemini:
```
GEMINI_API_KEY=tu_clave_api_aqui
```

### 2. Ejecutar en Modo Desarrollo

2.1. Desde el directorio `src-frontend`:
```bash
npm run tauri dev
```

2.2. La aplicación se iniciará en modo desarrollo

## 📦 Empaquetado

### 1. Generar Ejecutable

1.1. Desde el directorio `src-frontend`:
```bash
npm run tauri build
```

1.2. El ejecutable se generará en `src-tauri/target/release/`

### 2. Crear Paquete de Instalación

2.1. Desde la raíz del proyecto:
```bash
package-release.bat
```

2.2. El paquete se generará en `release-package/`

## ✅ Verificación Final

Después de completar todos los pasos anteriores, verifica:

- [ ] Todas las pruebas unitarias pasan
- [ ] El frontend se construye sin errores
- [ ] El backend se construye sin errores
- [ ] Las pruebas de integración pasan
- [ ] La aplicación se ejecuta en modo desarrollo
- [ ] El empaquetado genera un ejecutable válido

## 🆘 Solución de Problemas

### Problemas Comunes

1. **Error: 'cargo' no se reconoce como comando**
   - Asegúrate de que Rust está instalado correctamente
   - Reinicia tu terminal
   - Verifica que las variables de entorno estén configuradas

2. **Error: 'node' no se reconoce como comando**
   - Asegúrate de que Node.js está instalado
   - Reinicia tu terminal
   - Verifica que las variables de entorno estén configuradas

3. **Error de compilación en Tauri**
   - Asegúrate de tener las herramientas de desarrollo de Windows instaladas
   - Ejecuta el instalador de Visual Studio con soporte para desarrollo desktop C++

4. **Problemas con dependencias**
   - Elimina `node_modules` y `package-lock.json` y ejecuta `npm install` nuevamente
   - Elimina `target` en `src-tauri` y reconstruye

### Requisitos del Sistema

- Windows 10/11 (64-bit)
- 8GB RAM mínimo (16GB recomendado)
- 10GB de espacio en disco disponible
- Conexión a internet para descarga de dependencias

## 📞 Soporte

Para problemas de instalación o pruebas, consulta:

- Documentación en `docs/`
- Logs en `%USERPROFILE%\.oxidepilot\logs\`
- Configuración en `%USERPROFILE%\.oxidepilot\config.json`

---

**Versión**: 1.0.0 - Gemini Edition  
**Fecha**: 28 de julio de 2025
