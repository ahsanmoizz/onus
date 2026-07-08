<#
.SYNOPSIS
    Onus Windows installer.
.DESCRIPTION
    Installs the Onus CLI from a GitHub release archive, verifies checksums,
    configures PATH, creates a strict local config, and runs diagnostics.
#>

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Onus\bin",
    [switch]$NoPath,
    [switch]$DryRun,
    [switch]$Repair,
    [switch]$Upgrade,
    [switch]$NoVerify,
    [switch]$NoInteractive
)

$ErrorActionPreference = "Stop"

$Repo = "ahsanmoizz/onus"
$ReleaseBase = "https://github.com/$Repo/releases"
$ConfigDir = "$env:APPDATA\Onus"
$DataDir = "$env:LOCALAPPDATA\Onus\data"
$RulesDir = "$ConfigDir\rules"
$BinaryPath = Join-Path $InstallDir "onus.exe"
$ChecksumFile = "SHA256SUMS"

if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Host "32-bit Windows is not supported. Onus requires 64-bit Windows." -ForegroundColor Red
    exit 1
}

$ArchiveName = if ($Version -eq "latest") {
    "onus-latest-windows-x86_64.zip"
} else {
    "onus-$Version-windows-x86_64.zip"
}

if ($Version -eq "latest") {
    $DownloadUrl = "$ReleaseBase/latest/download/$ArchiveName"
    $ChecksumUrl = "$ReleaseBase/latest/download/$ChecksumFile"
} else {
    $DownloadUrl = "$ReleaseBase/download/$Version/$ArchiveName"
    $ChecksumUrl = "$ReleaseBase/download/$Version/$ChecksumFile"
}

function Write-Step {
    param([string]$Message)
    Write-Host ">> $Message" -ForegroundColor Yellow
}

function Write-OK {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor DarkYellow
}

function Write-Fail {
    param([string]$Message)
    Write-Host "[FAIL] $Message" -ForegroundColor Red
}

function Invoke-InstallStep {
    param([string]$Message, [scriptblock]$ScriptBlock)
    Write-Step $Message
    if (-not $DryRun) {
        & $ScriptBlock
    }
}

Write-Host ""
Write-Host "Onus Windows Installer" -ForegroundColor Cyan
Write-Host "  Version:  $Version"
Write-Host "  Archive:  $ArchiveName"
Write-Host "  Install:  $InstallDir"
Write-Host "  Config:   $ConfigDir"
Write-Host ""

if ($DryRun) {
    Write-Warn "Dry run mode. No changes will be written."
}
if ($Repair) {
    Write-Warn "Repair mode enabled."
}
if ($Upgrade) {
    Write-Warn "Upgrade mode enabled."
}

if (-not $NoInteractive -and -not $DryRun) {
    $confirm = Read-Host "Proceed with installation? (Y/n)"
    if ($confirm -eq "n" -or $confirm -eq "N") {
        Write-Host "Installation cancelled."
        exit 0
    }
}

$ArchivePath = $null
if (Test-Path $ArchiveName) {
    $ArchivePath = (Get-Item $ArchiveName).FullName
    Write-OK "Found local archive: $ArchivePath"
} else {
    Invoke-InstallStep "Downloading release archive" {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $ArchivePath = Join-Path $env:TEMP $ArchiveName
        Write-Host "  $DownloadUrl"
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchivePath -ErrorAction Stop
        Write-OK "Downloaded archive to $ArchivePath"
    }
}

if (-not $DryRun -and (-not $ArchivePath -or -not (Test-Path $ArchivePath))) {
    Write-Fail "Could not locate or download $ArchiveName"
    exit 1
}

if (-not $NoVerify -and -not $DryRun) {
    Write-Step "Verifying SHA-256 checksum"
    $expectedHash = $null
    if (Test-Path $ChecksumFile) {
        $checksumContent = Get-Content $ChecksumFile
    } else {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $checksumContent = (Invoke-WebRequest -Uri $ChecksumUrl -ErrorAction Stop).Content -split "`n"
    }
    $expectedHash = $checksumContent |
        Where-Object { $_ -match [regex]::Escape($ArchiveName) } |
        ForEach-Object { ($_ -split '\s+')[0] } |
        Select-Object -First 1

    if (-not $expectedHash) {
        Write-Fail "No checksum found for $ArchiveName"
        exit 1
    }

    $actualHash = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLower()
    if ($actualHash -ne $expectedHash.ToLower().Trim()) {
        Write-Fail "Checksum mismatch"
        Write-Host "  Expected: $expectedHash"
        Write-Host "  Actual:   $actualHash"
        exit 1
    }
    Write-OK "Checksum verified"
}

Invoke-InstallStep "Creating directories" {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    New-Item -ItemType Directory -Force -Path $RulesDir | Out-Null
    Write-OK "Directories ready"
}

Invoke-InstallStep "Extracting archive" {
    $ExtractDir = Join-Path $env:TEMP "onus-install-extract"
    if (Test-Path $ExtractDir) {
        Remove-Item -Recurse -Force $ExtractDir
    }
    New-Item -ItemType Directory -Force -Path $ExtractDir | Out-Null
    Expand-Archive -Path $ArchivePath -DestinationPath $ExtractDir -Force
    $extractedExe = Get-ChildItem -Path $ExtractDir -Recurse -Filter "onus.exe" | Select-Object -First 1
    if (-not $extractedExe) {
        Write-Fail "onus.exe not found in archive"
        exit 1
    }
    Copy-Item $extractedExe.FullName $BinaryPath -Force

    $uninstaller = Get-ChildItem -Path $ExtractDir -Recurse -Filter "uninstall-onus.ps1" | Select-Object -First 1
    if ($uninstaller) {
        Copy-Item $uninstaller.FullName (Join-Path $InstallDir "uninstall-onus.ps1") -Force
    }
    Remove-Item -Recurse -Force $ExtractDir -ErrorAction SilentlyContinue
    Write-OK "Installed onus.exe"
}

if (-not $NoPath -and -not $DryRun) {
    Write-Step "Configuring user PATH"
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $pathEntries = @()
    if ($userPath) {
        $pathEntries = $userPath -split ";" | Where-Object { $_ }
    }
    if ($pathEntries -contains $InstallDir) {
        Write-OK "$InstallDir already in PATH"
    } else {
        $newPath = (($pathEntries + $InstallDir) -join ";")
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        $env:PATH = "$env:PATH;$InstallDir"
        Write-OK "Added $InstallDir to PATH"
    }
}

Invoke-InstallStep "Creating local configuration" {
    $configFile = Join-Path $ConfigDir "onus.env"
    if (Test-Path $configFile) {
        Write-OK "Config already exists and was preserved"
    } else {
        $uiToken = [guid]::NewGuid().ToString("N") + [guid]::NewGuid().ToString("N")
        $semanticEndpoint = if ($env:ONUS_MANAGED_SEMANTIC_ENDPOINT) {
            $env:ONUS_MANAGED_SEMANTIC_ENDPOINT
        } else {
            "https://YOUR-ONUS-GATEWAY/v1/chat/completions"
        }
        $semanticToken = if ($env:ONUS_MANAGED_CLIENT_TOKEN) {
            $env:ONUS_MANAGED_CLIENT_TOKEN
        } else {
            "PASTE_ONUS_CLIENT_TOKEN_AFTER_ACTIVATION"
        }
        $configLines = @(
            "# Onus Configuration",
            "# Created by installer on $(Get-Date -Format 'yyyy-MM-dd')",
            "ONUS_STRICT=1",
            "ONUS_MISSING_CONTRACT=block_mutating",
            "ONUS_LOCAL_UI_TOKEN=$uiToken",
            "ONUS_SEMANTIC_PROVIDER=cloud",
            "ONUS_SEMANTIC_ENDPOINT=$semanticEndpoint",
            "ONUS_SEMANTIC_MODEL=onus-managed",
            "ONUS_SEMANTIC_API_KEY=$semanticToken",
            "ONUS_SEMANTIC_FALLBACK=fail_closed",
            "ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL=1",
            "ONUS_SEMANTIC_PRIVACY_MODE=strict",
            "ONUS_SEMANTIC_REDACT=1",
            "ONUS_SEMANTIC_TIMEOUT_MS=120000"
        )
        $configLines | Set-Content -Path $configFile -Encoding utf8
        Write-OK "Created config at $configFile"
    }
}

Invoke-InstallStep "Verifying binary" {
    $versionOutput = & $BinaryPath --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "Binary verification failed: $versionOutput"
        exit 1
    }
    Write-OK "onus --version: $versionOutput"
}

Invoke-InstallStep "Running diagnostics" {
    $doctorOutputPath = Join-Path $env:TEMP "onus-doctor-stdout.txt"
    $doctorErrorPath = Join-Path $env:TEMP "onus-doctor-stderr.txt"
    foreach ($path in @($doctorOutputPath, $doctorErrorPath)) {
        if (Test-Path $path) {
            Remove-Item -Force $path
        }
    }
    $doctorProcess = Start-Process `
        -FilePath $BinaryPath `
        -ArgumentList @("doctor") `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -RedirectStandardOutput $doctorOutputPath `
        -RedirectStandardError $doctorErrorPath
    $doctorExitCode = $doctorProcess.ExitCode
    $doctorOutput = Get-Content $doctorOutputPath -Raw -Encoding utf8 -ErrorAction SilentlyContinue
    if (Test-Path $doctorErrorPath) {
        $doctorError = Get-Content $doctorErrorPath -Raw -Encoding utf8 -ErrorAction SilentlyContinue
        if ($doctorError) {
            $doctorOutput = @($doctorOutput; $doctorError.Trim())
        }
    }
    foreach ($path in @($doctorOutputPath, $doctorErrorPath)) {
        Remove-Item -Force $path -ErrorAction SilentlyContinue
    }
    Write-Host $doctorOutput
    if ($doctorExitCode -eq 0) {
        Write-OK "Doctor passed"
    } else {
        Write-Warn "Doctor reported issues. Review output above."
    }
}

if (-not $DryRun) {
    try {
        $uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Onus"
        New-Item -Path $uninstallKey -Force -ErrorAction SilentlyContinue | Out-Null
        Set-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value "Onus AI Agent Firewall" -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value "$Version" -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value "$InstallDir" -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "$(Join-Path $InstallDir 'uninstall-onus.ps1')" -ErrorAction SilentlyContinue
    } catch {
        Write-Warn "Could not write uninstall registry key"
    }
}

Write-Host ""
Write-Host "Installation complete." -ForegroundColor Green
Write-Host "Next:"
Write-Host "  onus doctor"
Write-Host "  onus start"
Write-Host "  onus console --port 3001"
Write-Host ""

exit 0
