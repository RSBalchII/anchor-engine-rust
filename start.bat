@echo off
REM Anchor Engine Rust - Start Script
REM Automatically kills any process on port 3160 before starting

echo 🚀 Anchor Engine Rust - Starting...
echo.

REM Find and kill process on port 3160
echo 🧹 Checking for processes on port 3160...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3160 ^| findstr LISTENING') do (
    echo   🔒 Found process PID %%a on port 3160
    taskkill /F /PID %%a >nul 2>&1
    if errorlevel 1 (
        echo   ⚠️  Could not kill PID %%a
    ) else (
        echo   ✅ Killed PID %%a
    )
)

echo.
echo 📦 Starting Anchor Engine...
echo.

REM Start the engine (specify binary to avoid ambiguity)
cargo run --bin anchor-engine -- --port 3160

if errorlevel 1 (
    echo.
    echo ❌ Engine crashed!
    echo.
    exit /b 1
)

echo.
echo ✅ Engine stopped cleanly
echo.
