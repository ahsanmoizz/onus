#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Create an untracked local Onus configuration file from the template.
.DESCRIPTION
    Copies config/examples/onus.env.example to config/.env.local with
    deterministic provider selected by default.  Does NOT populate secrets.
    The user must edit the file to add API keys (if using cloud provider).
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Example = Join-Path $RepoRoot "config\examples\onus.env.example"
$Local   = Join-Path $RepoRoot "config\.env.local"

if (Test-Path $Local) {
    Write-Host "[configure-onus] Configuration already exists: $Local" -ForegroundColor Yellow
    Write-Host "[configure-onus] Delete it first to recreate." -ForegroundColor Yellow
    exit 0
}

if (-not (Test-Path $Example)) {
    Write-Error "[configure-onus] Template not found: $Example"
    exit 1
}

Copy-Item -Path $Example -Destination $Local
Write-Host "[configure-onus] Created: $Local" -ForegroundColor Green
Write-Host "[configure-onus] Edit this file to configure your provider (default: deterministic)." -ForegroundColor Cyan
Write-Host "[configure-onus] For cloud provider, add your ONUS_API_KEY (never commit it)." -ForegroundColor Cyan
