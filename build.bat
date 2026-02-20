@echo off
REM Anchor Engine Rust - Quick Build Script
REM Cleans locked files and builds automatically

echo 🚀 Anchor Engine Rust - Build Script
echo =====================================
echo.

REM Check if cleanup script exists
if exist "scripts\cleanup-build.ps1" (
    echo 🧹 Cleaning locked files...
    powershell -ExecutionPolicy Bypass -File scripts\cleanup-build.ps1
    if errorlevel 1 (
        echo.
        echo ⚠️  Cleanup had issues, but continuing with build...
        echo.
    )
) else (
    echo ⚠️  Cleanup script not found, skipping...
    echo.
)

echo 📦 Building anchor-engine...
echo.
cargo build -p anchor-engine

if errorlevel 1 (
    echo.
    echo ❌ Build failed!
    echo.
    echo Try running cleanup manually:
    echo   powershell -File scripts\cleanup-build.ps1
    echo.
    exit /b 1
)

echo.
echo ✅ Build successful!
echo.
echo To run:
echo   cargo run -p anchor-engine -- --port 3160
echo.
