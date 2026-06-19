#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Start the Onus daemon (core).
.DESCRIPTION
    Starts the Onus daemon in the foreground.  The daemon listens on
    port 4837 by default (override via ONUS_LISTEN_PORT).
    Press Ctrl+C to stop.
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Binary   = Join-Path $RepoRoot "onus\target\release\onus.exe"

# Check binary exists
if (-not (Test-Path $Binary)) {
    Write-Host "[start-onus-core] Binary not found. Building first..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "build-onus.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

# Load local config if present
$LocalConfig = Join-Path $RepoRoot "config\.env.local"
if (Test-Path $LocalConfig) {
    Write-Host "[start-onus-core] Loading config: $LocalConfig" -ForegroundColor Cyan
    Get-Content $LocalConfig | ForEach-Object {
        if ($_ -match '^\s*([^#]\w+)=(.*)$') {
            $EnvVar = $matches[1]
            $EnvVal = $matches[2]
            Set-Item -Path "env:$EnvVar" -Value $EnvVal
        }
    }
}

Write-Host "[start-onus-core] Starting Onus daemon on port $env:ONUS_LISTEN_PORT ..." -ForegroundColor Cyan
Write-Host "[start-onus-core] Press Ctrl+C to stop." -ForegroundColor Cyan

& $Binary daemon
