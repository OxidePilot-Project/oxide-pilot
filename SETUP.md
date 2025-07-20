# Guía de Instalación y Configuración de Oxide Pilot

Esta guía te ayudará a configurar el entorno de desarrollo completo para Oxide Pilot, incluyendo todas las dependencias necesarias para trabajar con Rust, Tauri y Svelte.

## Requisitos del Sistema

- **Sistema Operativo**: Windows 10/11, macOS 10.15+, o Linux (Ubuntu/Debian recomendado)
- **RAM**: Mínimo 8GB (16GB recomendado)
- **Espacio en Disco**: Al menos 10GB libres
- **Conexión a Internet**: Necesaria para descargar dependencias

## 1. Instalación de Rust y Cargo

Rust es el lenguaje principal para el backend de Oxide Pilot. Sigue estos pasos para instalarlo:

### Windows

1. Descarga el instalador de Rustup desde [https://rustup.rs/](https://rustup.rs/)
2. Ejecuta el archivo descargado y sigue las instrucciones
3. Alternativamente, abre PowerShell y ejecuta:
   ```powershell
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
4. Reinicia tu terminal o PowerShell
5. Verifica la instalación:
   ```powershell
   rustc --version
   cargo --version
   ```

### macOS / Linux

1. Abre una terminal y ejecuta:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Sigue las instrucciones del instalador (opción 1 para instalación estándar)
3. Carga Rust en tu shell actual:
   ```bash
   source $HOME/.cargo/env
   ```
4. Verifica la instalación:
   ```bash
   rustc --version
   cargo --version
   ```

## 2. Instalación de Node.js y npm

Node.js y npm son necesarios para el frontend de Svelte:

### Windows

1. Descarga el instalador LTS desde [https://nodejs.org/](https://nodejs.org/)
2. Ejecuta el instalador y sigue las instrucciones
3. Verifica la instalación:
   ```powershell
   node --version
   npm --version
   ```

### macOS

1. Usando Homebrew:
   ```bash
   brew install node
   ```
2. Verifica la instalación:
   ```bash
   node --version
   npm --version
   ```

### Linux (Ubuntu/Debian)

1. Actualiza los repositorios:
   ```bash
   sudo apt update
   ```
2. Instala Node.js y npm:
   ```bash
   sudo apt install nodejs npm
   ```
3. Para una versión más reciente, considera usar NodeSource:
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt install -y nodejs
   ```
4. Verifica la instalación:
   ```bash
   node --version
   npm --version
   ```

## 3. Dependencias del Sistema para Tauri

Tauri requiere algunas dependencias específicas del sistema:

### Windows

1. Instala Visual Studio Build Tools:
   - Descarga desde [Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Durante la instalación, selecciona "Desarrollo de escritorio con C++"
   - Esto instalará las herramientas necesarias para compilar código nativo

2. WebView2:
   - Generalmente ya está instalado en Windows 10/11
   - Si no, descárgalo desde [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### macOS

1. Instala Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

2. Instala dependencias adicionales con Homebrew:
   ```bash
   brew install cmake
   ```

### Linux (Ubuntu/Debian)

1. Instala las dependencias necesarias:
   ```bash
   sudo apt update
   sudo apt install -y \
     libwebkit2gtk-4.0-dev \
     build-essential \
     curl \
     wget \
     libssl-dev \
     libgtk-3-dev \
     libayatana-appindicator3-dev \
     librsvg2-dev
   ```

## 4. Instalación de Tauri CLI

La CLI de Tauri es necesaria para desarrollar y compilar la aplicación:

```bash
cargo install tauri-cli
```

Verifica la instalación:
```bash
cargo tauri --version
```

## 5. Configuración del Proyecto Oxide Pilot

Ahora que tienes todas las dependencias instaladas, puedes configurar el proyecto:

### Estructura de Directorios Recomendada

```
oxide-pilot/
├── src/                  # Código Rust del backend
│   ├── guardian/         # Agente Guardián
│   ├── copilot/          # Agente Copiloto
│   ├── ai/               # Orquestador de IA
│   ├── memory/           # Gestión de memoria
│   ├── rpa/              # Control RPA
│   └── utils/            # Utilidades compartidas
├── src-tauri/            # Configuración de Tauri
├── src-frontend/         # Frontend Svelte
└── tests/                # Pruebas unitarias e integración
```

### Inicializar el Proyecto

1. Crea el directorio del proyecto:
   ```bash
   mkdir -p oxide-pilot
   cd oxide-pilot
   ```

2. Inicializa un proyecto Tauri con Svelte:
   ```bash
   cargo tauri init
   ```

3. Responde a las preguntas del asistente:
   - ¿Qué gestor de paquetes? `npm`
   - ¿Ruta al directorio frontend? `src-frontend`
   - ¿Comando de desarrollo? `npm run dev`
   - ¿Comando de compilación? `npm run build`
   - ¿URL de desarrollo? `http://localhost:5173`

4. Configura el frontend Svelte:
   ```bash
   cd src-frontend
   npm install
   npm install @tauri-apps/api carbon-components-svelte
   cd ..
   ```

## 6. Dependencias de Rust Específicas

Edita el archivo `src-tauri/Cargo.toml` para añadir las dependencias necesarias:

```toml
[dependencies]
tauri = { version = "1.4", features = ["shell-open", "system-tray", "notification", "dialog", "fs-all", "path-all", "window-all", "process-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.28", features = ["full"] }
thiserror = "1.0"
log = "0.4"
env_logger = "0.10"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.3", features = ["v4", "serde"] }
sysinfo = "0.29"

# Para APIs específicas de Windows
[target.'cfg(windows)'.dependencies]
windows = { version = "0.48", features = ["Win32_System_ProcessStatus"] }
```

## 7. Ejecutar el Proyecto

Una vez configurado todo, puedes ejecutar el proyecto:

```bash
# En el directorio raíz del proyecto
cargo tauri dev
```

Esto iniciará el servidor de desarrollo y abrirá la aplicación Tauri con el frontend de Svelte.

## 8. Compilar para Producción

Cuando estés listo para crear una versión de producción:

```bash
cargo tauri build
```

Esto generará un instalador en el directorio `src-tauri/target/release/bundle`.

## Solución de Problemas Comunes

### Error: "cargo command not found"
- Asegúrate de que Rust está correctamente instalado
- Verifica que la variable PATH incluye `~/.cargo/bin`
- Reinicia tu terminal o sesión

### Error: "tauri command not found"
- Asegúrate de haber instalado tauri-cli: `cargo install tauri-cli`
- Verifica que la variable PATH incluye `~/.cargo/bin`

### Error de compilación en Windows
- Verifica que Visual Studio Build Tools está correctamente instalado
- Asegúrate de haber seleccionado "Desarrollo de escritorio con C++"

### Error de WebView2 en Windows
- Instala manualmente el runtime de WebView2 desde el sitio de Microsoft

### Errores de dependencias en Linux
- Asegúrate de haber instalado todas las dependencias del sistema listadas
- En algunas distribuciones, los nombres de los paquetes pueden variar ligeramente

## Recursos Adicionales

- [Documentación de Rust](https://www.rust-lang.org/learn)
- [Documentación de Tauri](https://tauri.app/v1/guides/)
- [Documentación de Svelte](https://svelte.dev/docs)
- [Repositorio de Cognee](https://github.com/topoteretes/cognee)