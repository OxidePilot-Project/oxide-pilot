@echo off
setlocal enabledelayedexpansion

echo Oxide Pilot - Package Release Script
echo =====================================
echo.
echo This script will package Oxide Pilot for release with Gemini-only support
echo.

REM Check if we're in the correct directory
if not exist "src-tauri" (
    echo ERROR: Please run this script from the oxide-pilot root directory
    pause
    exit /b 1
)

REM Create release directory
set RELEASE_DIR=release-package
if exist "%RELEASE_DIR%" rmdir /S /Q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

REM Build frontend
echo.
echo [1/5] Building frontend...
cd src-frontend
if exist "dist" rmdir /S /Q "dist"
call npm run build
cd ..

REM Build Tauri application
echo.
echo [2/5] Building Tauri application...
cd src-tauri
echo.
echo Building release binary...
call cargo build --release
cd ..

REM Check if build was successful
if not exist "src-tauri\target\release\oxide-pilot.exe" (
    echo ERROR: Tauri build failed
    pause
    exit /b 1
)

REM Create package structure
echo.
echo [3/5] Creating package structure...
mkdir "%RELEASE_DIR%\bin"
mkdir "%RELEASE_DIR%\config"
mkdir "%RELEASE_DIR%\docs"
mkdir "%RELEASE_DIR%\enterprise"

REM Copy binaries
echo.
echo [4/5] Copying binaries and assets...
copy "src-tauri\target\release\oxide-pilot.exe" "%RELEASE_DIR%\bin\"
copy "README.md" "%RELEASE_DIR%\docs\"
copy "LICENSE" "%RELEASE_DIR%\docs\" 2>nul || echo LICENSE not found, skipping...

REM Copy enterprise files
copy "enterprise\install-enterprise.bat" "%RELEASE_DIR%\enterprise\"
copy "enterprise\group-policy-template.admx" "%RELEASE_DIR%\enterprise\"
if exist "enterprise\group-policy-template.adml" copy "enterprise\group-policy-template.adml" "%RELEASE_DIR%\enterprise\"

REM Create installer script
echo.
echo [5/5] Creating installer...
(
echo @echo off
setlocal enabledelayedexpansion

echo Oxide Pilot - Gemini Edition Installation
echo ==========================================

echo.
echo Installing Oxide Pilot with Google Gemini support...
echo.

REM Create directories
set INSTALL_DIR=%USERPROFILE%\.oxidepilot
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy executable
copy "%~dp0bin\oxide-pilot.exe" "%INSTALL_DIR%\oxide-pilot.exe"

REM Create desktop shortcut
set SCRIPT=%TEMP%\create_shortcut.vbs
echo Set oWS = WScript.CreateObject("WScript.Shell") ^> %SCRIPT%
echo sLinkFile = oWS.SpecialFolders("Desktop") ^& "\Oxide Pilot.lnk" ^>^> %SCRIPT%
echo Set oLink = oWS.CreateShortcut(sLinkFile) ^>^> %SCRIPT%
echo oLink.TargetPath = "%INSTALL_DIR%\oxide-pilot.exe" ^>^> %SCRIPT%
echo oLink.WorkingDirectory = "%INSTALL_DIR%" ^>^> %SCRIPT%
echo oLink.Description = "Oxide Pilot - Gemini Edition" ^>^> %SCRIPT%
echo oLink.Save ^>^> %SCRIPT%
cscript %SCRIPT%
del %SCRIPT%

echo.
echo Installation completed!
echo.
echo Location: %INSTALL_DIR%
echo.
echo Please run Oxide Pilot from your desktop to complete setup.
echo You'll be prompted to enter your Google Gemini API key.
echo.
pause
) > "%RELEASE_DIR%\install.bat"

REM Create ZIP package
echo.
echo Creating ZIP package...
cd "%RELEASE_DIR%"
if exist "oxide-pilot-gemini-edition.zip" del "oxide-pilot-gemini-edition.zip"
powershell Compress-Archive -Path * -DestinationPath "oxide-pilot-gemini-edition.zip"

echo.
echo =====================================
echo PACKAGE CREATION COMPLETED!
echo =====================================
echo.
echo Package created: %RELEASE_DIR%\oxide-pilot-gemini-edition.zip
echo.
echo Contents:
echo - Oxide Pilot executable (Gemini Edition)
echo - Enterprise installation scripts
echo - Group Policy templates
echo - Documentation
echo.
echo Ready for distribution!
echo.

pause
