@echo off
REM Anchor Engine Rust - Quick Build Script
REM Cleans locked files and builds automatically

echo Anchor Engine Rust - Build Script
echo =====================================
echo.

REM Check if cleanup script exists
if exist "scripts\cleanup-build.ps1" (
    echo [CLEANUP] Running cleanup...
    powershell -ExecutionPolicy Bypass -File scripts\cleanup-build.ps1
    if errorlevel 1 (
        echo.
        echo [WARN] Cleanup had issues, but continuing with build...
        echo.
    )
) else (
    echo [WARN] Cleanup script not found, skipping...
    echo.
)

echo [BUILD] Building anchor-engine...
echo.
cargo build --bin anchor-engine

if errorlevel 1 (
    echo.
    echo [ERROR] Build failed!
    echo.
    echo Try running cleanup manually:
    echo   powershell -File scripts\cleanup-build.ps1
    echo.
    exit /b 1
)

echo.
echo [OK] Build successful!
echo.
echo To run:
echo   cargo run --bin anchor-engine -- --port 3160
echo.
