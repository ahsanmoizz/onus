#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build Onus from source.
.DESCRIPTION
    Builds the Onus binary (release profile) and prints the output path.
    Repository-relative paths only — no embedded credentials.
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$ProjectDir = Join-Path $RepoRoot "onus"

Write-Host "[build-onus] Building Onus (release)..." -ForegroundColor Cyan

# Check prerequisites
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "[build-onus] Rust/Cargo is not installed. Install from https://rustup.rs"
    exit 1
}

Push-Location $ProjectDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "[build-onus] Build failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }
    $Binary = Join-Path $ProjectDir "target\release\onus.exe"
    Write-Host "[build-onus] Build successful." -ForegroundColor Green
    Write-Host "[build-onus] Binary: $Binary" -ForegroundColor Green
} finally {
    Pop-Location
}
