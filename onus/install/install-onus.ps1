<#
.SYNOPSIS
    Onus — AI Agent Firewall Installer (Windows)
.DESCRIPTION
    Installs Onus, verifies checksums, configures PATH, and runs setup.
    Supports interactive, non-interactive, upgrade, repair, and dry-run modes.
.PARAMETER Version
    Release version to install (default: "latest")
.PARAMETER InstallDir
    Binary installation directory (default: %LOCALAPPDATA%\Onus\bin)
.PARAMETER NoPath
    Skip PATH modification
.PARAMETER DryRun
    Show what would happen without making changes
.PARAMETER Repair
    Reinstall without removing existing configuration
.PARAMETER Upgrade
    Upgrade from an existing installation
.PARAMETER NoVerify
    Skip SHA-256 checksum verification
.PARAMETER NoInteractive
    Run non-interactively (no prompts)
.EXAMPLE
    .\install-onus.ps1
    .\install-onus.ps1 -Version v0.1.0 -DryRun
    .\install-onus.ps1 -Upgrade -NoInteractive
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
$Host.UI.RawUI.WindowTitle = "Onus Installer"

# ── Configuration ──
$Repo = "ahsanmoizz/onus"
$ConfigDir = "$env:APPDATA\Onus"
$DataDir = "$env:LOCALAPPDATA\Onus\data"
$RulesDir = "$ConfigDir\rules"
$BinaryPath = "$InstallDir\onus.exe"
$ArchiveName = "onus-$Version-windows-x86_64.zip"
$ChecksumFile = "SHA256SUMS"
$ReleaseBase = "https://github.com/$Repo/releases"

if ($Version -eq "latest") {
    $DownloadUrl = "$ReleaseBase/latest/download/$ArchiveName"
    $ChecksumUrl = "$ReleaseBase/latest/download/$ChecksumFile"
} else {
    $DownloadUrl = "$ReleaseBase/download/$Version/$ArchiveName"
    $ChecksumUrl = "$ReleaseBase/download/$Version/$ChecksumFile"
}

# ── Helper functions ──
function Write-Banner {
    Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║        Onus — AI Agent Firewall             ║" -ForegroundColor Cyan
    Write-Host "║        Windows Installer                     ║" -ForegroundColor Cyan
    Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Step {
    param([string]$Message)
    Write-Host "  >> $Message" -ForegroundColor Yellow
}

function Write-OK {
    param([string]$Message)
    Write-Host "  [OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "  [WARN] $Message" -ForegroundColor DarkYellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "  [FAIL] $Message" -ForegroundColor Red
}

function Test-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# ── Dry-run guard ──
function Invoke-Step {
    param([string]$Message, [scriptblock]$ScriptBlock)
    Write-Step $Message
    if (-not $DryRun) {
        & $ScriptBlock
    }
}

# ── Main ──
Write-Banner

# ── 1. System detection ──
Write-Step "Detecting system architecture..."
$Arch = "x86_64"
if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Err "32-bit Windows is not supported. Onus requires a 64-bit operating system."
    exit 1
}
Write-OK "Windows / $Arch detected"

if (-not (Test-Administrator)) {
    Write-OK "Running as standard user (administrator rights not required)"
} else {
    Write-Warn "Running as administrator — Onus does not require admin rights"
}

# ── 2. Display plan ──
Write-Host ""
Write-Host "  Platform:   windows/$Arch"
Write-Host "  Version:    $Version"
Write-Host "  Binary:     $BinaryPath"
Write-Host "  Config:     $ConfigDir"
Write-Host "  Data:       $DataDir"
if ($DryRun) { Write-Host "  Mode:       DRY RUN (no changes)" }
if ($Repair) { Write-Host "  Mode:       REPAIR" }
if ($Upgrade) { Write-Host "  Mode:       UPGRADE" }
Write-Host ""

# ── 3. Confirm ──
if (-not $NoInteractive -and -not $DryRun) {
    $confirm = Read-Host "Proceed with installation? (Y/n)"
    if ($confirm -eq "n" -or $confirm -eq "N") {
        Write-Host "Installation cancelled."
        exit 0
    }
}

# ── 4. Locate or download archive ──
$ArchivePath = $null
if (Test-Path $ArchiveName) {
    $ArchivePath = (Get-Item $ArchiveName).FullName
    Write-OK "Found local archive: $ArchivePath"
} elseif (Test-Path ".\$ArchiveName") {
    $ArchivePath = (Get-Item ".\$ArchiveName").FullName
    Write-OK "Found local archive: $ArchivePath"
} else {
    Invoke-Step "Downloading $ArchiveName from GitHub releases..." {
        Write-Host "    URL: $DownloadUrl"
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $tmp = "$env:TEMP\$ArchiveName"
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $tmp -ErrorAction Stop
        $ArchivePath = $tmp
        Write-OK "Downloaded to $ArchivePath"
    }
}

if (-not $DryRun -and (-not $ArchivePath -or -not (Test-Path $ArchivePath))) {
    Write-Err "Could not locate or download $ArchiveName"
    exit 1
}

# ── 5. Verify SHA-256 checksum ──
if (-not $NoVerify -and -not $DryRun) {
    Write-Step "Verifying SHA-256 checksum..."
    $checksumPassed = $false
    $checksumsLocal = if (Test-Path $ChecksumFile) { Get-Content $ChecksumFile } else { $null }
    if ($checksumsLocal) {
        Write-Host "    Using local checksum file"
        $expectedHash = ($checksumsLocal | Where-Object { $_ -match [regex]::Escape($ArchiveName) } | ForEach-Object { ($_ -split '\s+')[0] })
    } else {
        Write-Host "    Downloading checksum file from GitHub..."
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            $checksumsContent = (Invoke-WebRequest -Uri $ChecksumUrl -ErrorAction Stop).Content
            $expectedHash = ($checksumsContent -split "`n" | Where-Object { $_ -match [regex]::Escape($ArchiveName) } | ForEach-Object { ($_ -split '\s+')[0] })
        } catch {
            Write-Warn "Could not download checksum file: $_"
        }
    }

    if ($expectedHash) {
        $actualHash = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLower()
        $expectedHash = $expectedHash.ToLower().Trim()
        if ($actualHash -eq $expectedHash) {
            Write-OK "Checksum verified ($($actualHash.Substring(0,16))...)"
            $checksumPassed = $true
        } else {
            Write-Err "Checksum MISMATCH"
            Write-Host "    Expected: $expectedHash"
            Write-Host "    Actual:   $actualHash"
            Write-Host "    The archive may be corrupted or tampered with."
            if (-not $NoInteractive) {
                $continue = Read-Host "Continue anyway? (y/N)"
                if ($continue -ne "y" -and $continue -ne "Y") { exit 1 }
            } else {
                exit 1
            }
        }
    } else {
        Write-Warn "No checksum found for $ArchiveName in checksum file — skipping verification"
    }
}

# ── 6. Create directories ──
Invoke-Step "Creating installation directories..." {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    New-Item -ItemType Directory -Force -Path $RulesDir | Out-Null
    Write-OK "Directories created"
}

# ── 7. Extract archive ──
Invoke-Step "Extracting archive..." {
    if ($ArchivePath.EndsWith('.zip')) {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($ArchivePath, $InstallDir, $true)
    } else {
        Write-Err "Unsupported archive format: $ArchivePath"
        exit 1
    }

    # If extracted onus.exe is in a subdirectory, move it
    $extractedExe = Get-ChildItem -Path $InstallDir -Recurse -Filter "onus.exe" | Select-Object -First 1
    if ($extractedExe -and $extractedExe.DirectoryName -ne $InstallDir) {
        Move-Item -Force $extractedExe.FullName "$InstallDir\onus.exe"
    }

    if (-not (Test-Path $BinaryPath)) {
        Write-Err "onus.exe not found after extraction"
        exit 1
    }
    Write-OK "Extracted onus.exe to $BinaryPath"
}

# ── 8. Add to PATH ──
if (-not $NoPath -and -not $DryRun) {
    Write-Step "Configuring PATH..."
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -split ";" -contains $InstallDir) {
        Write-OK "$InstallDir already in PATH"
    } else {
        $newPath = if ($userPath) { "$userPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        # Also update current session
        $env:PATH = "$env:PATH;$InstallDir"
        Write-OK "Added $InstallDir to user PATH"
        Write-Host "    (may need to restart terminal for change to take effect)"
    }
}

# ── 9. Create placeholder config ──
Invoke-Step "Creating configuration..." {
    $configFile = "$ConfigDir\onus.env"
    if (-not (Test-Path $configFile)) {
        $uiToken = [guid]::NewGuid().ToString("N") + [guid]::NewGuid().ToString("N")
        $semanticEndpoint = if ($env:ONUS_MANAGED_SEMANTIC_ENDPOINT) { $env:ONUS_MANAGED_SEMANTIC_ENDPOINT } else { "https://YOUR-ONUS-GATEWAY/v1/chat/completions" }
        $semanticToken = if ($env:ONUS_MANAGED_CLIENT_TOKEN) { $env:ONUS_MANAGED_CLIENT_TOKEN } else { "PASTE_ONUS_CLIENT_TOKEN_AFTER_ACTIVATION" }
@"
# Onus Configuration
# Created by installer on $(Get-Date -Format 'yyyy-MM-dd')
# This file is loaded automatically by the Onus CLI.

ONUS_STRICT=1
ONUS_MISSING_CONTRACT=block_mutating
ONUS_LOCAL_UI_TOKEN=$uiToken

# Managed semantic review.
# This token is an Onus gateway client token, not a raw model-provider key.
ONUS_SEMANTIC_PROVIDER=cloud
ONUS_SEMANTIC_ENDPOINT=$semanticEndpoint
ONUS_SEMANTIC_MODEL=onus-managed
ONUS_SEMANTIC_API_KEY=$semanticToken
ONUS_SEMANTIC_FALLBACK=fail_closed
ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL=1
ONUS_SEMANTIC_PRIVACY_MODE=strict
ONUS_SEMANTIC_REDACT=1
ONUS_SEMANTIC_TIMEOUT_MS=30000

"@ | Out-File -FilePath $configFile -Encoding utf8
        Write-OK "Created production-safe config at $configFile"
    } else {
        Write-OK "Config already exists at $configFile (preserved)"
    }
}

# ── 10. Verify binary ──
Invoke-Step "Verifying Onus binary..." {
    $result = & $BinaryPath --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-OK "onus --version: $result"
    } else {
        Write-Err "Binary verification failed: $result"
        exit 1
    }
}

# ── 11. Run doctor ──
Invoke-Step "Running Onus diagnostics..." {
    Write-Host ""
    $doctorResult = & $BinaryPath doctor 2>&1
    Write-Host "$doctorResult"
    Write-Host ""
    if ($LASTEXITCODE -eq 0) {
        Write-OK "Doctor check passed"
    } else {
        Write-Warn "Doctor reported issues (non-zero exit)"
    }
}

# ── 12. Write uninstall registry key ──
if (-not $DryRun) {
    try {
        $uninstallKey = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Onus"
        if (-not (Test-Path $uninstallKey)) {
            # Try HKCU since we don't require admin
            $uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Onus"
            $null = New-Item -Path $uninstallKey -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path $uninstallKey) {
            Set-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value "Onus AI Agent Firewall" -ErrorAction SilentlyContinue
            Set-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value "$Version" -ErrorAction SilentlyContinue
            Set-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value "$InstallDir" -ErrorAction SilentlyContinue
            Set-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "$InstallDir\uninstall-onus.ps1" -ErrorAction SilentlyContinue
        }
    } catch {
        Write-Warn "Could not write uninstall registry key (non-admin install)"
    }
}

# ── 13. Print next steps ──
Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║        Installation Complete!                ║" -ForegroundColor Green
Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "  Next steps:"
Write-Host "   1. Run:         onus setup"
Write-Host "   2. Doctor:      onus doctor"
Write-Host "   3. Start:       onus daemon start"
Write-Host "   4. Console:     onus dashboard"
Write-Host ""
Write-Host "  Documentation: https://ahsanmoizz.github.io/onus/docs"
Write-Host "  Uninstall:     $InstallDir\uninstall-onus.ps1"
Write-Host ""

exit 0
