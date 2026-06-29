<#
.SYNOPSIS
    Onus — AI Agent Firewall Uninstaller (Windows)
.DESCRIPTION
    Removes Onus binary, PATH entries, and optionally configuration.
    Preserves audit data by default.
.PARAMETER Purge
    Also delete configuration, rules, and audit data
.PARAMETER NoInteractive
    Skip confirmation prompts
.EXAMPLE
    .\uninstall-onus.ps1
    .\uninstall-onus.ps1 -Purge
    .\uninstall-onus.ps1 -NoInteractive
#>

param(
    [switch]$Purge,
    [switch]$NoInteractive
)

$ErrorActionPreference = "Stop"
$Host.UI.RawUI.WindowTitle = "Onus Uninstaller"

# ── Configuration ──
$InstallDir = "$env:LOCALAPPDATA\Onus\bin"
$ConfigDir = "$env:APPDATA\Onus"
$DataDir = "$env:LOCALAPPDATA\Onus\data"
$BinaryPath = "$InstallDir\onus.exe"

Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        Onus — AI Agent Firewall             ║" -ForegroundColor Cyan
Write-Host "║        Windows Uninstaller                   ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Detect installation ──
$found = @()
if (Test-Path $BinaryPath) { $found += "Binary: $BinaryPath" }
if (Test-Path $ConfigDir) { $found += "Config: $ConfigDir" }
if (Test-Path $DataDir) { $found += "Data: $DataDir" }

if ($found.Count -eq 0) {
    Write-Host "Onus does not appear to be installed. Nothing to remove." -ForegroundColor Yellow
    exit 0
}

Write-Host "Found Onus installation:" -ForegroundColor Yellow
foreach ($item in $found) {
    Write-Host "  $item"
}
Write-Host ""

if ($Purge) {
    Write-Host "  Mode: PURGE (all data including audit trail will be deleted)" -ForegroundColor Red
} else {
    Write-Host "  Mode: STANDARD (audit data and configuration preserved)" -ForegroundColor Green
    Write-Host "  Use --Purge to delete all configuration and audit data."
}
Write-Host ""

# ── Confirm ──
if (-not $NoInteractive) {
    $confirm = Read-Host "Remove Onus? (y/N)"
    if ($confirm -ne "y" -and $confirm -ne "Y") {
        Write-Host "Uninstall cancelled."
        exit 0
    }
}

# ── Stop daemon if running ──
if (Test-Path $BinaryPath) {
    Write-Host "  >> Stopping Onus daemon if running..." -ForegroundColor Yellow
    & $BinaryPath daemon stop 2>$null | Out-Null
    Start-Sleep -Milliseconds 500
}

# ── Remove from PATH ──
Write-Host "  >> Removing from PATH..." -ForegroundColor Yellow
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath) {
    $entries = $userPath -split ";" | Where-Object { $_ -ne $InstallDir -and $_ -ne "" }
    $newPath = $entries -join ";"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "    Removed $InstallDir from user PATH"
}

# ── Remove binary ──
Write-Host "  >> Removing binary..." -ForegroundColor Yellow
if (Test-Path $BinaryPath) {
    Remove-Item -Force $BinaryPath -ErrorAction SilentlyContinue
    Write-Host "    Removed $BinaryPath"
}

# Remove binary directory if empty
if (Test-Path $InstallDir) {
    $remaining = Get-ChildItem -Path $InstallDir -ErrorAction SilentlyContinue
    if (-not $remaining) {
        Remove-Item -Force $InstallDir -ErrorAction SilentlyContinue
        Write-Host "    Removed empty directory $InstallDir"
    }
}

# ── Remove configuration (only if --purge) ──
if ($Purge) {
    Write-Host "  >> Purging configuration and data..." -ForegroundColor Red
    if (Test-Path $ConfigDir) {
        Remove-Item -Recurse -Force $ConfigDir -ErrorAction SilentlyContinue
        Write-Host "    Removed $ConfigDir"
    }
    if (Test-Path $DataDir) {
        Remove-Item -Recurse -Force $DataDir -ErrorAction SilentlyContinue
        Write-Host "    Removed $DataDir"
    }
} else {
    Write-Host "  >> Preserving configuration and audit data:" -ForegroundColor Green
    Write-Host "    Config: $ConfigDir"
    Write-Host "    Data:   $DataDir"
}

# ── Remove uninstall registry key ──
try {
    $uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Onus"
    if (Test-Path $uninstallKey) {
        Remove-Item -Path $uninstallKey -Recurse -Force -ErrorAction SilentlyContinue
    }
} catch {
    # Non-critical
}

Write-Host ""
Write-Host "Onus has been removed." -ForegroundColor Green
if (-not $Purge) {
    Write-Host "To reinstall: run install-onus.ps1"
    Write-Host "To also remove audit data: run with -Purge"
}
Write-Host ""

exit 0
