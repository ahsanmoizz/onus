#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Run full Onus diagnostics.
.DESCRIPTION
    Runs onus doctor for all integrations.  Reports provider configuration,
    installed agents, hook status, and MCP proxy configuration.
    Repository-relative paths only — no embedded credentials.
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Binary   = Join-Path $RepoRoot "onus\target\release\onus.exe"

if (-not (Test-Path $Binary)) {
    Write-Host "[doctor-onus] Binary not found. Building first..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "build-onus.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "[doctor-onus] Running Onus diagnostics..." -ForegroundColor Cyan
Write-Host ""

& $Binary doctor
Write-Host ""

Write-Host "[doctor-onus] Diagnostics complete." -ForegroundColor Green
