#!/usr/bin/env pwsh
# Anchor Engine Rust - Build Cleanup Script
# Removes locked target files that interfere with builds on Windows

param(
    [switch]$Verbose,
    [switch]$MoveInsteadOfDelete
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$TargetDir = Join-Path $ProjectRoot "target"

Write-Host "🧹 Anchor Engine Rust - Build Cleanup" -ForegroundColor Cyan
Write-Host "   Project: $ProjectRoot" -ForegroundColor Gray
Write-Host ""

if (-not (Test-Path $TargetDir)) {
    Write-Host "✅ No target directory found - nothing to clean" -ForegroundColor Green
    exit 0
}

# Find locked files
Write-Host "🔍 Scanning for locked files..." -ForegroundColor Yellow
$lockedFiles = @()
$failedFiles = @()

try {
    # Try to get all .o and .rcgu files
    $objectFiles = Get-ChildItem -Path $TargetDir -Recurse -File -Include "*.o", "*.rcgu.o", "*.rlib" -ErrorAction SilentlyContinue
    
    foreach ($file in $objectFiles) {
        try {
            # Try to open file for write - if it fails, it's locked
            $stream = $file.Open([System.IO.FileMode]::Open, [System.IO.FileAccess]::Write, [System.IO.FileAccess]::None)
            $stream.Close()
        }
        catch {
            $lockedFiles += $file.FullName
            if ($Verbose) {
                Write-Host "   🔒 Locked: $($file.FullName)" -ForegroundColor Red
            }
        }
    }
}
catch {
    Write-Host "❌ Error scanning: $_" -ForegroundColor Red
    exit 1
}

if ($lockedFiles.Count -eq 0) {
    Write-Host "✅ No locked files found - build directory is clean!" -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "📊 Found $($lockedFiles.Count) locked files" -ForegroundColor Yellow
Write-Host ""

# Handle locked files
if ($MoveInsteadOfDelete) {
    # Move locked files to parent directory
    $BackupDir = Join-Path (Split-Path -Parent $ProjectRoot) "anchor-rust-v0-locked-backup"
    Write-Host "📦 Moving locked files to: $BackupDir" -ForegroundColor Cyan
    
    if (-not (Test-Path $BackupDir)) {
        New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
    }
    
    foreach ($file in $lockedFiles) {
        try {
            $fileName = Split-Path -Leaf $file
            Move-Item -Path $file -Destination (Join-Path $BackupDir $fileName) -Force
            if ($Verbose) {
                Write-Host "   ✓ Moved: $fileName" -ForegroundColor Green
            }
        }
        catch {
            $failedFiles += $file
            Write-Host "   ✗ Failed to move: $(Split-Path -Leaf $file)" -ForegroundColor Red
        }
    }
}
else {
    # Delete locked files (default)
    Write-Host "🗑️  Deleting locked files..." -ForegroundColor Cyan
    
    foreach ($file in $lockedFiles) {
        try {
            # Force delete by renaming first (Windows workaround)
            $tempName = $file + ".delete"
            Rename-Item -Path $file -NewName $tempName -Force
            Remove-Item -Path $tempName -Force
            if ($Verbose) {
                Write-Host "   ✓ Deleted: $(Split-Path -Leaf $file)" -ForegroundColor Green
            }
        }
        catch {
            $failedFiles += $file
            Write-Host "   ✗ Failed to delete: $(Split-Path -Leaf $file)" -ForegroundColor Red
        }
    }
}

Write-Host ""
Write-Host "📊 Summary:" -ForegroundColor Cyan
Write-Host "   Processed: $($lockedFiles.Count) files" -ForegroundColor Gray
Write-Host "   Failed: $($failedFiles.Count) files" -ForegroundColor $(if ($failedFiles.Count -eq 0) { "Green" } else { "Red" })
Write-Host ""

if ($failedFiles.Count -gt 0) {
    Write-Host "⚠️  Some files could not be removed. Try:" -ForegroundColor Yellow
    Write-Host "   1. Close any running cargo/rust processes" -ForegroundColor Gray
    Write-Host "   2. Run: taskkill /F /IM rustc.exe" -ForegroundColor Gray
    Write-Host "   3. Run: taskkill /F /IM cargo.exe" -ForegroundColor Gray
    Write-Host "   4. Run this script again" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   Or use: .\scripts\cleanup-build.ps1 -MoveInsteadOfDelete" -ForegroundColor Cyan
    exit 1
}

Write-Host "✅ Cleanup complete! You can now build." -ForegroundColor Green
Write-Host ""
Write-Host "💡 Tip: Run 'cargo build' to rebuild" -ForegroundColor Cyan
