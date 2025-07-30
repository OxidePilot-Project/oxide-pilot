@echo off
setlocal enabledelayedexpansion

echo Oxide Pilot Enterprise Installation Script
echo ============================================

REM Check for admin privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This script requires administrator privileges.
    echo Please run as administrator.
    pause
    exit /b 1
)

REM Set installation directory
set INSTALL_DIR=%ProgramFiles%\OxidePilot
set CONFIG_DIR=%ProgramData%\OxidePilot
set USER_CONFIG_DIR=%USERPROFILE%\.oxidepilot

echo.
echo Installing Oxide Pilot Enterprise...
echo.

REM Create directories
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%CONFIG_DIR%" mkdir "%CONFIG_DIR%"
if not exist "%USER_CONFIG_DIR%" mkdir "%USER_CONFIG_DIR%"

REM Copy application files
echo Copying application files...
xcopy /E /I /Y "%~dp0..\src-tauri\target\release\oxide-pilot.exe" "%INSTALL_DIR%\"
xcopy /E /I /Y "%~dp0..\enterprise\*" "%CONFIG_DIR%\"

REM Create default configuration
echo Creating default configuration...
(
echo {
echo   "guardian": {
echo     "enable_monitoring": true,
echo     "enable_threat_detection": true,
echo     "monitoring_interval": 5000
echo   },
echo   "copilot": {
echo     "enable_voice": true,
echo     "enable_rpa": true,
echo     "max_context_length": 8192
echo   },
echo   "ai_providers": {
echo     "google": {
echo       "model": "gemini-1.5-pro",
echo       "max_tokens": 8192,
echo       "temperature": 0.7
echo     }
echo   },
echo   "security": {
echo     "require_admin_approval": true,
echo     "enable_audit_log": true,
echo     "max_log_size": 1000
echo   },
echo   "performance": {
echo     "background_priority": true,
echo     "max_cpu_percent": 5.0,
echo     "max_memory_mb": 256
echo   }
echo }
) > "%CONFIG_DIR%\config.json"

REM Create Windows service
echo Creating Windows service...
sc create OxidePilotEnterprise binPath= "%INSTALL_DIR%\oxide-pilot.exe --service" start= auto
sc description OxidePilotEnterprise "Oxide Pilot Enterprise AI Assistant"

REM Add to PATH
echo Adding to system PATH...
setx PATH "%PATH%;%INSTALL_DIR%" /M

REM Install Group Policy templates
echo Installing Group Policy templates...
if not exist "%SystemRoot%\PolicyDefinitions" mkdir "%SystemRoot%\PolicyDefinitions"
copy "%~dp0group-policy-template.admx" "%SystemRoot%\PolicyDefinitions\"
if not exist "%SystemRoot%\PolicyDefinitions\en-US" mkdir "%SystemRoot%\PolicyDefinitions\en-US"
copy "%~dp0group-policy-template.adml" "%SystemRoot%\PolicyDefinitions\en-US\"

REM Create firewall rules
echo Creating firewall rules...
netsh advfirewall firewall add rule name="OxidePilot" dir=in action=allow program="%INSTALL_DIR%\oxide-pilot.exe"
netsh advfirewall firewall add rule name="OxidePilot" dir=out action=allow program="%INSTALL_DIR%\oxide-pilot.exe"

REM Create desktop shortcut
echo Creating desktop shortcut...
(
echo Set oWS = WScript.CreateObject("WScript.Shell")
echo sLinkFile = oWS.SpecialFolders("Desktop") ^& "\Oxide Pilot Enterprise.lnk"
echo Set oLink = oWS.CreateShortcut(sLinkFile)
echo oLink.TargetPath = "%INSTALL_DIR%\oxide-pilot.exe"
echo oLink.WorkingDirectory = "%INSTALL_DIR%"
echo oLink.Description = "Oxide Pilot Enterprise AI Assistant"
echo oLink.Save
) > "%TEMP%\create_shortcut.vbs"
cscript "%TEMP%\create_shortcut.vbs"
del "%TEMP%\create_shortcut.vbs"

echo.
echo Installation completed successfully!
echo.
echo Configuration directory: %CONFIG_DIR%
echo User configuration: %USER_CONFIG_DIR%
echo.
echo Please restart your computer or run 'Oxide Pilot Enterprise' from the Start menu.
echo.
echo To configure Google Gemini API key, run the application and follow the setup wizard.

pause
