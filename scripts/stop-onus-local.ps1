#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Stop Onus daemon and related processes.
.DESCRIPTION
    Stops the Onus daemon process and any MCP proxy processes
    started by the user.  Does NOT force-kill unrelated processes.
    Only stops processes that match the onus binary name.
#>

$ErrorActionPreference = "Continue"

Write-Host "[stop-onus-local] Stopping Onus daemon processes..." -ForegroundColor Cyan

# Find and stop onus daemon processes
$DaemonProcesses = Get-Process -Name "onus" -ErrorAction SilentlyContinue
if ($DaemonProcesses) {
    foreach ($Proc in $DaemonProcesses) {
        Write-Host "[stop-onus-local] Stopping PID $($Proc.Id)..." -ForegroundColor Yellow
        Stop-Process -Id $Proc.Id -Force -ErrorAction SilentlyContinue
    }
    Write-Host "[stop-onus-local] Stopped $($DaemonProcesses.Count) process(es)." -ForegroundColor Green
} else {
    Write-Host "[stop-onus-local] No Onus daemon processes found." -ForegroundColor Green
}

Write-Host "[stop-onus-local] Cleanup complete." -ForegroundColor Green
