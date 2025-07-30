@echo off
setlocal

echo.
echo ================================================
echo            OXIDE PILOT - TEST SUITE            
echo ================================================
echo.

echo [ Environment Setup and Verification ]
echo.

:: Check if Rust is available
echo Checking Rust installation...
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Rust not found! Please install Rust first.
    echo Download from: https://rustup.rs/
    pause
    exit /b 1
) else (
    echo ✅ Rust found
    for /f "tokens=*" %%i in ('rustc --version') do set rust_version=%%i
    echo %rust_version%
)

echo.
:: Check if Node.js is available
echo Checking Node.js installation...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Node.js not found! Please install Node.js first.
    echo Download from: https://nodejs.org/
    pause
    exit /b 1
) else (
    echo ✅ Node.js found
    for /f "tokens=*" %%i in ('node --version') do set node_version=%%i
    echo %node_version%
    for /f "tokens=*" %%i in ('npm --version') do set npm_version=%%i
    echo npm %npm_version%
)

echo.
:: Check Tauri CLI
echo Checking Tauri CLI...
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ⚠️  Tauri CLI not found!
    echo Installing Tauri CLI...
    cargo install tauri-cli
    if %errorlevel% neq 0 (
        echo ❌ Failed to install Tauri CLI
        pause
        exit /b 1
    ) else (
        echo ✅ Tauri CLI installed successfully
    )
) else (
    echo ✅ Tauri CLI found
    for /f "tokens=*" %%i in ('cargo tauri --version') do set tauri_version=%%i
    echo %tauri_version%
)

echo.
echo ================================================
echo.

:: Pause to allow user to see environment checks
echo Environment checks complete. Press any key to continue with build and tests...
pause >nul

goto :build

:build
echo.
echo [ Building Project Components ]
echo.

:: Clean previous builds
echo Cleaning previous builds...
if exist "src-tauri\target\release" rmdir /s /q "src-tauri\target\release" >nul 2>&1
if exist "src-frontend\dist" rmdir /s /q "src-frontend\dist" >nul 2>&1

:: Install frontend dependencies
echo Installing frontend dependencies...
cd src-frontend
call npm install
if %errorlevel% neq 0 (
    echo ❌ Frontend dependency installation failed!
    cd ..
    pause
    exit /b 1
)

echo.
:: Build frontend
echo Building frontend...
call npm run build
if %errorlevel% neq 0 (
    echo ❌ Frontend build failed!
    cd ..
    pause
    exit /b 1
)
cd ..

echo.
:: Build Rust backend
echo Building Rust backend...
cargo build --release
if %errorlevel% neq 0 (
    echo ❌ Rust build failed!
    pause
    exit /b 1
)

echo.
echo ================================================
echo.

echo Build complete. Press any key to continue with tests...
pause >nul

goto :tests

:tests
echo.
echo [ Running Test Suites ]
echo.

:: Run unit tests
echo 📋 Running Unit Tests...
cargo test --lib --all
if %errorlevel% neq 0 (
    echo ❌ Unit tests failed!
    pause
    exit /b 1
)

echo.
:: Run integration tests
echo 🔗 Running Integration Tests...
cargo test --test integration_system_tests
if %errorlevel% neq 0 (
    echo ❌ Integration system tests failed!
    pause
    exit /b 1
)

cargo test --test integration_audio_tests
if %errorlevel% neq 0 (
    echo ❌ Integration audio tests failed!
    pause
    exit /b 1
)

echo.
:: Run performance tests
echo ⚡ Running Performance Tests...
cargo test --test performance_tests
if %errorlevel% neq 0 (
    echo ❌ Performance tests failed!
    pause
    exit /b 1
)

echo.
:: Run function tests
echo 🎯 Running Function Tests...
cargo test -p oxide-copilot test_function_registry
if %errorlevel% neq 0 (
    echo ❌ Function registry tests failed!
    pause
    exit /b 1
)

cargo test -p oxide-guardian test_threat_detection
if %errorlevel% neq 0 (
    echo ❌ Threat detection tests failed!
    pause
    exit /b 1
)

cargo test -p oxide-memory test_memory_operations
if %errorlevel% neq 0 (
    echo ❌ Memory operations tests failed!
    pause
    exit /b 1
)

echo.
:: Run audio tests (may fail without hardware)
echo 🔊 Running Audio Tests (may fail without hardware)...
cargo test --test integration_audio_tests --features hardware_tests || echo "⚠️ Audio hardware tests skipped (no hardware detected)"

echo.
:: Generate test report
echo 📊 Generating Test Report...
cargo test --all -- --nocapture > test_results.txt 2>&1

echo.
echo ================================================
echo.

echo ✅ All tests completed!
echo 📄 Results saved to test_results.txt
echo.

echo Press any key to exit...
pause >nul

goto :eof
