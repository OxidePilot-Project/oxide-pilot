@echo off
setlocal ENABLEDELAYEDEXPANSION

rem Oxide Pilot project cleanup script
rem - Removes build/test caches and all *.profraw coverage files
rem - Optional deep clean (Rust target and npm cache)
rem - Optional global clean (Cargo registry & git cache)
rem - Optional removal of pattern-craft folder
rem - Optional removal of superdesign folder

rem Resolve project root as parent of this script directory
set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%.." >nul 2>&1
set "PROJ=%CD%"
popd >nul 2>&1

echo.
echo Oxide Cleanup
echo Project root: "%PROJ%"
rem Detect unified Cargo target directory (CARGO_TARGET_DIR or default to %PROJ%\target)
set "TARGET_DIR=%CARGO_TARGET_DIR%"
if not defined TARGET_DIR set "TARGET_DIR=%PROJ%\target"
echo Cargo target: "%TARGET_DIR%"
echo.

:askMode
choice /C QD /N /M "Select clean mode: [Q]uick (recommended) or [D]eep: "
if errorlevel 2 ( set "DEEP=1" ) else ( set "DEEP=0" )

echo.
choice /C YN /N /M "Also clear GLOBAL Cargo registry/git caches? [Y]es/[N]o: "
if errorlevel 2 ( set "CLR_CARGO_GLOBAL=0" ) else ( set "CLR_CARGO_GLOBAL=1" )

echo.
if exist "%PROJ%\pattern-craft" (
  choice /C YN /N /M "Remove '%PROJ%\pattern-craft' folder? [Y]es/[N]o: "
  if errorlevel 2 ( set "RM_PATTERN=0" ) else ( set "RM_PATTERN=1" )
) else (
  set "RM_PATTERN=0"
)

echo.
if exist "%PROJ%\superdesign" (
  choice /C YN /N /M "Remove '%PROJ%\superdesign' folder? [Y]es/[N]o: "
  if errorlevel 2 ( set "RM_SUPERDESIGN=0" ) else ( set "RM_SUPERDESIGN=1" )
) else (
  set "RM_SUPERDESIGN=0"
)

echo.
choice /C YN /N /M "Preview ignored deletions with 'git clean -ndX'? [Y]es/[N]o: "
if errorlevel 2 (
  echo Skipping git clean preview.
) else (
  echo --- git clean -ndX (preview) ---
  git clean -ndX
  echo -------------------------------
  choice /C YN /N /M "Proceed to delete ignored files now with 'git clean -fdX' (excluding node_modules)? [Y]es/[N]o: "
  if errorlevel 2 (
    echo Skipping deletion of ignored files.
  ) else (
    rem Exclude node_modules to avoid Windows lock prompts; user can delete manually as needed
    git clean -fdX -e src-frontend/node_modules
  )
)

echo.
echo Cleaning coverage artifacts (*.profraw)...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-ChildItem -LiteralPath '%PROJ%' -Recurse -Filter '*.profraw' -Force -ErrorAction SilentlyContinue ^| Remove-Item -Force -ErrorAction SilentlyContinue"

echo Removing frontend caches...
if exist "%PROJ%\src-frontend\.svelte-kit" rmdir /S /Q "%PROJ%\src-frontend\.svelte-kit"
if exist "%PROJ%\src-frontend\dist" rmdir /S /Q "%PROJ%\src-frontend\dist"
if exist "%PROJ%\src-frontend\node_modules\.vite" rmdir /S /Q "%PROJ%\src-frontend\node_modules\.vite"

rem Always trim Rust incremental artifacts; deeper removal if DEEP=1
echo Removing Rust incremental artifacts...
if exist "%TARGET_DIR%\debug\incremental" rmdir /S /Q "%TARGET_DIR%\debug\incremental"

rem Remove hidden workspace caches if present
if exist "%PROJ%\.target-workspace" rmdir /S /Q "%PROJ%\.target-workspace"
if exist "%PROJ%\src-tauri\.target-workspace" rmdir /S /Q "%PROJ%\src-tauri\.target-workspace"

echo.
choice /C YN /N /M "Run 'cargo clean' in src-tauri (safe even if workspace root is broken)? [Y]es/[N]o: "
if errorlevel 2 (
  echo Skipping cargo clean in src-tauri.
) else (
  pushd "%PROJ%\src-tauri" >nul 2>&1
  cargo clean
  popd >nul 2>&1
)

if "%DEEP%"=="1" (
  echo Deep clean enabled: removing Rust target and npm cache...
  if exist "%TARGET_DIR%" rmdir /S /Q "%TARGET_DIR%"
  powershell -NoProfile -ExecutionPolicy Bypass -Command "if (Test-Path \"$env:LOCALAPPDATA\npm-cache\") { Get-ChildItem -LiteralPath \"$env:LOCALAPPDATA\npm-cache\" -Force -ErrorAction SilentlyContinue ^| Remove-Item -Recurse -Force -ErrorAction SilentlyContinue }"
)

if "%CLR_CARGO_GLOBAL%"=="1" (
  echo Clearing GLOBAL Cargo caches (registry, git)...
  powershell -NoProfile -ExecutionPolicy Bypass -Command "if (Test-Path \"$env:USERPROFILE\.cargo\registry\") { Remove-Item -LiteralPath \"$env:USERPROFILE\.cargo\registry\" -Recurse -Force -ErrorAction SilentlyContinue }"
  powershell -NoProfile -ExecutionPolicy Bypass -Command "if (Test-Path \"$env:USERPROFILE\.cargo\git\") { Remove-Item -LiteralPath \"$env:USERPROFILE\.cargo\git\" -Recurse -Force -ErrorAction SilentlyContinue }"
)

if "%RM_PATTERN%"=="1" (
  echo Removing folder: "%PROJ%\pattern-craft" ...
  rmdir /S /Q "%PROJ%\pattern-craft"
)

if "%RM_SUPERDESIGN%"=="1" (
  echo Removing folder: "%PROJ%\superdesign" ...
  rmdir /S /Q "%PROJ%\superdesign"
)

echo.
echo Cleanup complete.
echo You may now re-run: cargo tauri dev

echo.
pause
endlocal
