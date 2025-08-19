@echo off
setlocal enableextensions enabledelayedexpansion

:: Oxide Pilot - Unified Dev Launcher (Windows BAT)
:: - Prompts to clear frontend caches (dist, .svelte-kit, node_modules\.vite)
:: - Prompts for a custom port OR auto-picks a random free port
:: - Patches Tauri config to use that port (beforeDevCommand/devPath), then restores it after exit

:: Resolve repo root relative to this script (scripts\)
set "SCRIPT_DIR=%~dp0"
set "ROOT=%SCRIPT_DIR%.."
pushd "%ROOT%" >nul 2>&1

echo ===============================================
echo  Oxide Pilot - Development Launcher
echo ===============================================
echo.

:: 1) Optional cache clean
set "CLEAR_PROMPT=Clear frontend caches (dist, .svelte-kit, node_modules\.vite) before start? [Y/N]: "
set /p CLEAR=%CLEAR_PROMPT%

if /I "%CLEAR%"=="Y" goto DO_CLEAN
if /I "%CLEAR%"=="YES" goto DO_CLEAN
goto ASK_PORT

:DO_CLEAN
echo.
echo [*] Cleaning caches under "%ROOT%\src-frontend" ...
if exist "src-frontend\dist" (
  rmdir /S /Q "src-frontend\dist"
  echo     - removed dist
) else (
  echo     - dist not found (ok)
)

if exist "src-frontend\.svelte-kit" (
  rmdir /S /Q "src-frontend\.svelte-kit"
  echo     - removed .svelte-kit
) else (
  echo     - .svelte-kit not found (ok)
)

if exist "src-frontend\node_modules\.vite" (
  rmdir /S /Q "src-frontend\node_modules\.vite"
  echo     - removed node_modules\.vite
) else (
  echo     - node_modules\.vite not found (ok)
)
echo [*] Cache clean completed.
echo.

:ASK_PORT
set "PORT_INPUT="
echo Choose dev server port.
echo   - Enter a specific port number (e.g. 5317),
echo   - Or press ENTER to use the dedicated default (5317).
set /p PORT_INPUT=Port (blank = 5317): 

set "PORT=%PORT_INPUT%"
if not defined PORT set "PORT=5317"

:: simple numeric validation
echo %PORT%| findstr /R "^[0-9][0-9]*$" >nul || (
  echo [!] Invalid port. Using default 5317.
  set "PORT=5317"
)
goto PREP_DEV

:PREP_DEV
echo [*] Using dev port: %PORT%

:: 2) Patch Tauri config with selected port (absolute beforeDevCommand) and backup original
set "TAURI_DIR=src-tauri"
set "CONF=%TAURI_DIR%\tauri.conf.json"
set "CONF_BAK=%TAURI_DIR%\.tauri.conf.json.dev.bak"

if exist "%CONF_BAK%" del /Q "%CONF_BAK%" >nul 2>&1
copy /Y "%CONF%" "%CONF_BAK%" >nul 2>&1

powershell -NoProfile -Command ^
  "$p=[int]$env:PORT;" ^
  ";$root=Resolve-Path '.';" ^
  ";$prefix=Join-Path $root 'src-frontend';" ^
  ";$j=Get-Content -Raw '%CONF%' | ConvertFrom-Json;" ^
  ";$j.build.devPath=\"http://localhost:$p\";" ^
  ";$j.build.beforeDevCommand=\"npm --prefix $prefix run dev -- --port $p\";" ^
  ";$out=$j | ConvertTo-Json -Depth 10;" ^
  ";[System.IO.File]::WriteAllText('%CONF%',$out,(New-Object System.Text.UTF8Encoding($false)))"

if errorlevel 1 (
  echo [!] Failed to patch tauri.conf.json. Restoring backup and aborting.
  if exist "%CONF_BAK%" copy /Y "%CONF_BAK%" "%CONF%" >nul 2>&1
  goto END
)

:: 3) Start dev (Tauri will start Vite via beforeDevCommand and connect to chosen port)
echo.
echo [*] Starting development environment...
echo     - Vite dev server: http://localhost:%PORT%
echo     - Tauri dev app will connect to that URL
echo.

pushd "%TAURI_DIR%" >nul 2>&1
cargo tauri dev
set "EXITCODE=%ERRORLEVEL%"
popd >nul 2>&1

:: 4) Restore original config
if exist "%CONF_BAK%" (
  copy /Y "%CONF_BAK%" "%CONF%" >nul 2>&1
  del /Q "%CONF_BAK%" >nul 2>&1
)

echo.
echo [*] Tauri exited with code %EXITCODE%.
echo [*] Restored original tauri.conf.json.
echo.
pause

:END
popd >nul 2>&1
endlocal
