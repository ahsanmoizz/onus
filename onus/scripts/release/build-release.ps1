#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build Onus Windows release artifacts.
.DESCRIPTION
    Builds or reuses the release binary, packages the same archive layout used
    by GitHub Releases, creates latest aliases, and writes SHA256SUMS.
#>

param(
    [switch]$DryRun,
    [switch]$SkipBuild,
    [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $PSCommandPath
$OnusDir = Resolve-Path "$ScriptDir/../.." -ErrorAction Stop
$ProjectRoot = Resolve-Path "$OnusDir/.." -ErrorAction Stop
if (-not $OutDir) {
    $OutDir = Join-Path $ProjectRoot "dist/releases"
}

$CargoToml = Get-Content (Join-Path $OnusDir "Cargo.toml") -Raw
$VersionMatch = [regex]::Match($CargoToml, 'version\s*=\s*"([^"]+)"')
$Version = if ($VersionMatch.Success) { $VersionMatch.Groups[1].Value } else { "0.1.0" }
$Commit = if (Get-Command git -ErrorAction SilentlyContinue) {
    $hash = git -C $ProjectRoot rev-parse HEAD 2>$null
    if ($hash) { $hash.Trim() } else { "unknown" }
} else {
    "unknown"
}
$BuildDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$Platform = "windows"
$Arch = "x86_64"
$ArchiveName = "onus-$Version-$Platform-$Arch.zip"
$LatestArchiveName = "onus-latest-$Platform-$Arch.zip"
$BinaryName = "onus.exe"

Write-Host "Onus Windows release builder"
Write-Host "  Version:  $Version"
Write-Host "  Commit:   $Commit"
Write-Host "  Output:   $OutDir"

if ($DryRun) {
    Write-Host "Dry run OK. No files written."
    exit 0
}

if (-not $SkipBuild) {
    Write-Host "Building release binary..."
    Push-Location $OnusDir
    try {
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build --release failed"
        }
    } finally {
        Pop-Location
    }
} else {
    Write-Host "Skipping build because -SkipBuild was provided."
}

$BinaryPath = Join-Path $OnusDir "target/release/$BinaryName"
if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found at $BinaryPath"
}
$BinaryVersion = & $BinaryPath --version 2>&1
Write-Host "  Binary: $BinaryVersion"

$StagingDir = Join-Path $env:TEMP "onus-release-staging"
if (Test-Path $StagingDir) {
    Remove-Item -Recurse -Force $StagingDir
}
New-Item -ItemType Directory -Force -Path (Join-Path $StagingDir "bin") | Out-Null

Copy-Item $BinaryPath (Join-Path $StagingDir "bin/$BinaryName") -Force
Copy-Item (Join-Path $OnusDir "install/install-onus.ps1") (Join-Path $StagingDir "install-onus.ps1") -Force
Copy-Item (Join-Path $OnusDir "install/uninstall-onus.ps1") (Join-Path $StagingDir "uninstall-onus.ps1") -Force
if (Test-Path (Join-Path $ProjectRoot "LICENSE")) {
    Copy-Item (Join-Path $ProjectRoot "LICENSE") (Join-Path $StagingDir "LICENSE") -Force
}
if (Test-Path (Join-Path $ProjectRoot "README.md")) {
    Copy-Item (Join-Path $ProjectRoot "README.md") (Join-Path $StagingDir "README.md") -Force
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$ArchivePath = Join-Path $OutDir $ArchiveName
$LatestArchivePath = Join-Path $OutDir $LatestArchiveName
if (Test-Path $ArchivePath) { Remove-Item -Force $ArchivePath }
if (Test-Path $LatestArchivePath) { Remove-Item -Force $LatestArchivePath }

Compress-Archive -Path (Join-Path $StagingDir "*") -DestinationPath $ArchivePath -Force
Copy-Item $ArchivePath $LatestArchivePath -Force
Copy-Item (Join-Path $OnusDir "install/install-onus.ps1") (Join-Path $OutDir "install-onus.ps1") -Force
Copy-Item (Join-Path $OnusDir "install/install.ps1") (Join-Path $OutDir "install.ps1") -Force

$HashFile = Join-Path $OutDir "SHA256SUMS"
Get-ChildItem -Path $OutDir -File |
    Where-Object { $_.Name -notin @("SHA256SUMS", "release-manifest.json") } |
    Sort-Object Name |
    ForEach-Object {
        "$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower())  $($_.Name)"
    } |
    Set-Content -Path $HashFile -Encoding ascii

$ArchiveHash = (Get-FileHash $ArchivePath -Algorithm SHA256).Hash.ToLower()
$Manifest = @{
    version = $Version
    commit = $Commit
    build_date = $BuildDate
    artifacts = @(
        @{
            platform = $Platform
            architecture = $Arch
            filename = $ArchiveName
            sha256 = $ArchiveHash
            size = (Get-Item $ArchivePath).Length
            download_url = "https://github.com/ahsanmoizz/onus/releases/latest/download/$ArchiveName"
        },
        @{
            platform = $Platform
            architecture = $Arch
            filename = $LatestArchiveName
            sha256 = (Get-FileHash $LatestArchivePath -Algorithm SHA256).Hash.ToLower()
            size = (Get-Item $LatestArchivePath).Length
            download_url = "https://github.com/ahsanmoizz/onus/releases/latest/download/$LatestArchiveName"
        }
    )
}
$Manifest | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $OutDir "release-manifest.json") -Encoding utf8

Remove-Item -Recurse -Force $StagingDir -ErrorAction SilentlyContinue

Write-Host "Release artifacts ready:"
Write-Host "  $ArchivePath"
Write-Host "  $LatestArchivePath"
Write-Host "  $HashFile"
