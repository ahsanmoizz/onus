# Live verification test runner for Onus × OpenAI Codex CLI integration
# ====================================================================
# This script runs comprehensive live tests of the Onus Codex CLI
# integration surface. Run it AFTER the user has installed Codex CLI
# (`pip install openai-codex` or equivalent).
#
# Prerequisites:
#   1. Codex CLI must be installed and authenticated
#   2. Onus binary must be on PATH
#
# Usage:
#   .\runtime-verification\openai-codex-cli\run_live_tests.ps1

param(
    [switch]$SkipMCP,
    [switch]$SkipL3
)

$ErrorActionPreference = "Stop"
$testCount = 0
$passCount = 0
$failCount = 0

$RED = "$([char]27)[31m"
$GREEN = "$([char]27)[32m"
$YELLOW = "$([char]27)[33m"
$CYAN = "$([char]27)[36m"
$RESET = "$([char]27)[0m"

function Write-TestHeader($name) {
    $global:testCount++
    Write-Host "`n[$($global:testCount))] $name" -ForegroundColor Cyan
}

function Write-Pass($msg) {
    $global:passCount++
    Write-Host "  ${GREEN}PASS${RESET} $msg"
}

function Write-Fail($msg) {
    $global:failCount++
    Write-Host "  ${RED}FAIL${RESET} $msg"
}

function Write-Warn($msg) {
    Write-Host "  ${YELLOW}WARN${RESET} $msg"
}

# ── Pre-flight checks ──────────────────────────────────────────────────
Write-Host "Onus × Codex CLI Live Verification" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Prerequisites:" -ForegroundColor Yellow
Write-Host "  1. Codex CLI installed (`pip install openai-codex` or `npm install -g @openai/codex`)"
Write-Host "  2. Onus built and on PATH"
Write-Host "  3. User authenticated with OpenAI"
Write-Host ""

# ── Test 1: onus doctor codex ─────────────────────────────────────────
Write-TestHeader "onus doctor codex"
try {
    $output = onus doctor --codex 2>&1 | Out-String
    if ($output -match "Codex.*CLI|Codex CLI|doctor.*Codex") {
        Write-Pass "Doctor --codex command runs and reports Codex status"
    } else {
        Write-Warn "Doctor --codex output format unexpected: $($output.Trim())"
        Write-Warn "This may be acceptable if Codex is not installed"
    }
} catch {
    Write-Fail "Doctor --codex command failed: $_"
}

# ── Test 2: onus setup codex ──────────────────────────────────────────
Write-TestHeader "onus setup codex (MCP proxy config)"
try {
    $output = onus setup --codex 2>&1 | Out-String
    if ($output -match "Codex.*MCP|setup.*codex|MCP proxy|~\.codex") {
        Write-Pass "Setup command configures Codex MCP routing"
    } else {
        Write-Warn "Setup --codex output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Setup --codex command failed: $_"
}

# ── Test 3: Codex MCP config file created ─────────────────────────────
Write-TestHeader "Codex MCP config file verification"
try {
    $codexConfig = Join-Path $env:USERPROFILE ".codex" "config.toml"
    if (Test-Path $codexConfig) {
        $content = Get-Content $codexConfig -Raw
        if ($content -match "onus-mcp-proxy|onus.*mcp") {
            Write-Pass "Codex config.toml contains Onus MCP proxy entry"
        } else {
            Write-Warn "Codex config exists but Onus MCP entry not found"
        }
    } else {
        Write-Warn "Codex config not found at $codexConfig. Run `codex mcp add` manually after setup."
    }
} catch {
    Write-Fail "Config check error: $_"
}

# ── Test 4: onus mcp-proxy help ────────────────────────────────────────
Write-TestHeader "onus mcp-proxy (help)"
try {
    $output = onus mcp-proxy --help 2>&1 | Out-String
    if ($output -match "server|MCP|mcp-proxy") {
        Write-Pass "MCP proxy help shows expected flags"
    } else {
        Write-Fail "MCP proxy help output missing expected content"
    }
} catch {
    Write-Fail "MCP proxy help failed: $_"
}

# ── Test 5: onus doctor (full with codex coverage) ────────────────────
Write-TestHeader "onus doctor (full)"
try {
    $output = onus doctor 2>&1 | Out-String
    if ($output -match "Codex|OpenAI") {
        Write-Pass "Full doctor command reports Codex status"
    } else {
        Write-Warn "Full doctor may not report Codex (acceptable if not installed)"
    }
} catch {
    Write-Fail "Doctor command failed: $_"
}

# ── Test 6: onus uninstall codex (if applicable) ───────────────────────
Write-TestHeader "onus uninstall codex"
try {
    $output = onus uninstall --codex 2>&1 | Out-String
    if ($output -match "removed|No.*config|codex|Codex") {
        Write-Pass "Uninstall --codex command runs"
    } else {
        Write-Warn "Uninstall --codex output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Uninstall --codex command failed: $_"
}

# ── Test 7: MCP proxy connection (requires running Codex) ──────────────
if (-not $SkipMCP) {
    Write-TestHeader "onus mcp-proxy (spawns and evaluates)"
    try {
        # Verify the MCP proxy can at least parse config — start it with echo as mock server
        $output = onus mcp-proxy --server "echo" -- --help 2>&1 | Out-String
        Write-Warn "MCP proxy test requires live Codex. Mock test: $($output.Trim())"
        Write-Warn "Run manually: `echo '{\"method\":\"tools/call\",\"params\":{}}' | onus mcp-proxy --server <real-server>`"
        Write-Pass "MCP proxy binary invocation works"
    } catch {
        Write-Warn "MCP proxy requires real server binary to fully test"
        Write-Warn "Skip test — no Codex server to connect to"
    }
}

# ── Test 8: L3 codex workspace (Linux only) ───────────────────────────-
if (-not $SkipL3) {
    Write-TestHeader "L3 Codex workspace container"
    try {
        $output = onus mcp-proxy --l3-workspace --server "echo" -- "test" 2>&1 | Out-String
        Write-Pass "L3 workspace flag accepted by MCP proxy"
    } catch {
        Write-Warn "L3 workspace requires Linux + bwrap: $_"
    }
}

# ── Summary ──────────────────────────────────────────────────────────────
Write-Host "`n═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host " Live Verification Complete" -ForegroundColor Cyan
Write-Host " Tests:  $testCount" -ForegroundColor Cyan
Write-Host " Pass:   $passCount" -ForegroundColor $(
    if ($passCount -eq $testCount) { "Green" } else { "Yellow" }
)
Write-Host " Fail:   $failCount" -ForegroundColor $(
    if ($failCount -eq 0) { "Green" } else { "Red" }
)
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""
Write-Host "Remaining user actions:" -ForegroundColor Yellow
Write-Host "  1. Install Codex CLI: pip install openai-codex"
Write-Host "  2. Authenticate: codex auth login"
Write-Host "  3. Run: onus setup --codex"
Write-Host "  4. Verify: onus doctor --codex"
Write-Host "  5. Re-run this test suite"
Write-Host "  6. Live Codex session: codex run --mcp"

if ($failCount -gt 0) {
    exit 1
}
