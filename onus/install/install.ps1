<#
.SYNOPSIS
    Short Windows bootstrapper for Onus.
.DESCRIPTION
    Downloads the full archive-based installer from the latest GitHub release
    and runs it with the provided arguments.
#>

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Onus\bin",
    [switch]$NoPath,
    [switch]$NoVerify,
    [switch]$NoInteractive
)

$ErrorActionPreference = "Stop"
$Repo = "ahsanmoizz/onus"
$ReleaseBase = "https://github.com/$Repo/releases"
$ScriptName = "install-onus.ps1"

if ($Version -eq "latest") {
    $InstallerUrl = "$ReleaseBase/latest/download/$ScriptName"
} else {
    $InstallerUrl = "$ReleaseBase/download/$Version/$ScriptName"
}

$TempInstaller = Join-Path $env:TEMP "onus-$ScriptName"

Write-Host "Onus Windows bootstrapper"
Write-Host "Downloading installer:"
Write-Host "  $InstallerUrl"

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $InstallerUrl -OutFile $TempInstaller -ErrorAction Stop

$arguments = @(
    "-ExecutionPolicy", "Bypass",
    "-File", $TempInstaller,
    "-Version", $Version,
    "-InstallDir", $InstallDir
)

if ($NoPath) { $arguments += "-NoPath" }
if ($NoVerify) { $arguments += "-NoVerify" }
if ($NoInteractive) { $arguments += "-NoInteractive" }

& powershell @arguments
exit $LASTEXITCODE
