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
echo Cleaning coverage artifacts (*.profraw)...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-ChildItem -LiteralPath '%PROJ%' -Recurse -Filter '*.profraw' -Force -ErrorAction SilentlyContinue ^| Remove-Item -Force -ErrorAction SilentlyContinue"

echo Removing frontend caches...
if exist "%PROJ%\src-frontend\.svelte-kit" rmdir /S /Q "%PROJ%\src-frontend\.svelte-kit"
if exist "%PROJ%\src-frontend\dist" rmdir /S /Q "%PROJ%\src-frontend\dist"
if exist "%PROJ%\src-frontend\node_modules\.vite" rmdir /S /Q "%PROJ%\src-frontend\node_modules\.vite"

rem Always trim Rust incremental artifacts; deeper removal if DEEP=1
echo Removing Rust incremental artifacts...
if exist "%PROJ%\src-tauri\target\debug\incremental" rmdir /S /Q "%PROJ%\src-tauri\target\debug\incremental"

if "%DEEP%"=="1" (
  echo Deep clean enabled: removing Rust target and npm cache...
  if exist "%PROJ%\src-tauri\target" rmdir /S /Q "%PROJ%\src-tauri\target"
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
