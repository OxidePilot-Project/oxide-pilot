@echo off
echo Iniciando la aplicacion Oxide Pilot en modo de desarrollo...
echo.

REM Navegar al directorio src-tauri
cd src-tauri

REM Ejecutar la aplicacion Tauri en modo de desarrollo
REM Asegurate de que Rust y Tauri CLI esten instalados y en tu PATH.
REM Si 'cargo' no esta en tu PATH, puedes usar la ruta completa como:
REM "%USERPROFILE%\.\cargo\bin\cargo" tauri dev
"%USERPROFILE%\.cargo\bin\cargo" tauri dev

echo.
echo La aplicacion Tauri se ha cerrado.
pause
