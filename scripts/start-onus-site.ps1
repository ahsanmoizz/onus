#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Start the Onus official website.
.DESCRIPTION
    Serves the static site/ directory.
    Requires a static file server (Python's http.server or similar).
    Default URL: http://localhost:3000
#>

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$SiteDir  = Join-Path $RepoRoot "site"
$Port     = 3000

if (-not (Test-Path (Join-Path $SiteDir "index.html"))) {
    Write-Error "[start-onus-site] Site not found at $SiteDir"
    exit 1
}

# Prefer Python 3
$Python = $null
if (Get-Command python -ErrorAction SilentlyContinue) {
    $Python = "python"
} elseif (Get-Command python3 -ErrorAction SilentlyContinue) {
    $Python = "python3"
}

Push-Location $SiteDir
try {
    if ($Python) {
        Write-Host "[start-onus-site] Starting Python HTTP server on port $Port ..." -ForegroundColor Cyan
        Write-Host "[start-onus-site] URL: http://localhost:$Port" -ForegroundColor Cyan
        Write-Host "[start-onus-site] Press Ctrl+C to stop." -ForegroundColor Cyan
        & $Python -m http.server $Port
    } else {
        Write-Error "[start-onus-site] Python is not installed. Install Python 3 or serve $SiteDir with your preferred server."
        Write-Host "[start-onus-site] Alternatives:" -ForegroundColor Yellow
        Write-Host "  npx serve $SiteDir" -ForegroundColor Yellow
        exit 1
    }
} finally {
    Pop-Location
}
