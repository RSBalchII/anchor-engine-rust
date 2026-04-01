@echo off
REM Anchor Engine MCP Server - Start Script
REM Starts the Model Context Protocol server

echo 🚀 Anchor Engine MCP Server - Starting...
echo.

REM Find and kill any existing MCP server process
echo 🧹 Checking for existing MCP server processes...
tasklist /FI "IMAGENAME eq anchor-mcp.exe" 2>NUL | find /I /N "anchor-mcp.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo   🔒 Found existing MCP server process
    taskkill /F /IM "anchor-mcp.exe" >nul 2>&1
    if errorlevel 1 (
        echo   ⚠️  Could not kill MCP server process
    ) else (
        echo   ✅ Killed existing MCP server process
    )
)

echo.
echo 📦 Starting MCP Server...
echo.

REM Start the MCP server
cargo run --bin anchor-mcp --

if errorlevel 1 (
    echo.
    echo ❌ MCP Server crashed!
    echo.
    exit /b 1
)

echo.
echo ✅ MCP Server stopped cleanly
echo.