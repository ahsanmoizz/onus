#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Start the Onus console (web UI).
.DESCRIPTION
    A dedicated Onus console (React/Next.js dashboard) is not yet built.
    For now, the console is the CLI dashboard:
      onus dashboard

    When a web console is added in a future phase, this script will launch it.
#>

Write-Host "[start-onus-console] A dedicated web console is not yet available." -ForegroundColor Yellow
Write-Host "[start-onus-console] Use the CLI dashboard instead:" -ForegroundColor Cyan
Write-Host "" -ForegroundColor Cyan
Write-Host "  onus dashboard" -ForegroundColor Cyan
Write-Host "" -ForegroundColor Cyan

# Future: When a console/ directory is added with package.json, use:
#   cd onus/console && npm ci && npm run dev
