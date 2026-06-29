# Onus Windows Installer
# Usage:
#   powershell -ExecutionPolicy Bypass -c "iwr -useb https://github.com/ahsanmoizz/onus/releases/latest/download/install.ps1 | iex"

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\onus",
    [switch]$NoPath
)

$ErrorActionPreference = "Stop"

Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        Onus — AI Agent Firewall             ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$ConfigDir = "$env:APPDATA\onus"
$DataDir  = "$InstallDir\data"
$RulesDir = "$ConfigDir\rules"
$BinaryPath = "$InstallDir\onus.exe"

# ── Detect architecture ──
$Arch = "x86_64"
if ([Environment]::Is64BitOperatingSystem -eq $false) {
    Write-Host "32-bit Windows is not supported" -ForegroundColor Red
    exit 1
}

Write-Host "  Platform:  windows/${Arch}"
Write-Host "  Version:   $Version"
Write-Host "  Install:   $BinaryPath"
Write-Host "  Config:    $ConfigDir"
Write-Host ""

# ── Create directories ──
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
New-Item -ItemType Directory -Force -Path $RulesDir | Out-Null

# ── Download binary ──
if ($Version -eq "latest") {
    $DownloadUrl = "https://github.com/ahsanmoizz/onus/releases/latest/download/onus-windows-${Arch}.exe"
} else {
    $DownloadUrl = "https://github.com/ahsanmoizz/onus/releases/download/${Version}/onus-windows-${Arch}.exe"
}

Write-Host "Downloading..."
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $DownloadUrl -OutFile "${BinaryPath}.tmp" -ErrorAction Stop
    Move-Item -Force "${BinaryPath}.tmp" $BinaryPath
    Write-Host "  ✓ Binary installed" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Download failed: $_" -ForegroundColor Red
    exit 1
}

# ── Install default rules ──
Write-Host "  ✓ Installing default safety rules..."
try {
    & $BinaryPath rules init 2>$null
} catch {
    Write-Host "  ⚠ Could not initialize rules" -ForegroundColor Yellow
}

# ── Wire Claude Code hook ──
Write-Host "  ✓ Configuring Claude Code..."
$ClaudeConfig = "${env:USERPROFILE}\.claude\settings.json"
if (Test-Path $ClaudeConfig) {
    try {
        $config = Get-Content $ClaudeConfig -Raw | ConvertFrom-Json
        if (-not $config.hooks) { $config | Add-Member -Name "hooks" -Value @{} -MemberType NoteProperty }
        $config.hooks | Add-Member -Name "PreToolUse" -Value @(
            @{
                matcher = "*"
                hooks = @(
                    @{
                        type = "command"
                        command = $BinaryPath
                        args = @("claude-hook")
                        timeout = 5
                        statusMessage = "Onus reviewing Claude Code tool call"
                    }
                )
            }
        ) -MemberType NoteProperty -Force
        $config | ConvertTo-Json -Depth 10 | Set-Content $ClaudeConfig
        Write-Host "  ✓ Claude Code hook wired" -ForegroundColor Green
    } catch {
        Write-Host "  ⚠ Could not auto-configure Claude Code" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ○ Claude Code not detected — skip hook setup"
}

# ── Add to PATH ──
if (-not $NoPath) {
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "${UserPath};${InstallDir}", "User")
        # Update current session
        $env:PATH = "${env:PATH};${InstallDir}"
        Write-Host "  ✓ Added to PATH" -ForegroundColor Green
    } else {
        Write-Host "  ✓ Already in PATH"
    }
}

# ── Verify ──
Write-Host ""
try {
    $version = & $BinaryPath --version 2>&1
    Write-Host "$version" -ForegroundColor Green
    Write-Host ""
    Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  Onus installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Quick start:"
    Write-Host "    onus shell install       # Protect terminal agents"
    Write-Host "    onus mcp-proxy --help    # Protect MCP-based agents"
    Write-Host "    onus rules list          # See all safety rules"
    Write-Host "    onus --help              # Full command list"
    Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
} catch {
    Write-Host "⚠ Binary may need PATH update — restart your terminal" -ForegroundColor Yellow
}
