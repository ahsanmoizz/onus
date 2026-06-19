#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Run Onus local tests (no credentials required).
.DESCRIPTION
    Builds the release binary, runs cargo test, runs doctor, and verifies
    the CLI help output.  Does NOT require API keys or agent login.
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Binary   = Join-Path $RepoRoot "onus\target\release\onus.exe"
$ExitCode = 0

# ---- Step 1: Build ----
Write-Host "=== Step 1: Build ===" -ForegroundColor Cyan
& (Join-Path $PSScriptRoot "build-onus.ps1")
if ($LASTEXITCODE -ne 0) { $ExitCode = 1 }

# ---- Step 2: Unit tests ----
Write-Host "=== Step 2: Unit tests ===" -ForegroundColor Cyan
Push-Location (Join-Path $RepoRoot "onus")
try {
    cargo test --lib
    if ($LASTEXITCODE -ne 0) { $ExitCode = 1 }
} finally {
    Pop-Location
}

# ---- Step 3: CLI help ----
Write-Host "=== Step 3: CLI help ===" -ForegroundColor Cyan
& $Binary --help
if ($LASTEXITCODE -ne 0) { $ExitCode = 1 }

# ---- Step 4: Doctor (no credentials needed, may warn but not error) ----
Write-Host "=== Step 4: Doctor ===" -ForegroundColor Cyan
& $Binary doctor
# doctor may warn about missing integrations — that's expected

# ---- Summary ----
if ($ExitCode -eq 0) {
    Write-Host "=== All local tests passed ===" -ForegroundColor Green
} else {
    Write-Error "=== Some local tests failed ==="
}
exit $ExitCode
