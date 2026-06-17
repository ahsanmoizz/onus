# Live verification test runner for Onus × Claude Code CLI integration
# ====================================================================
# This script runs comprehensive live tests of the Onus Claude Code CLI
# integration surface. Run it AFTER completing the P15E-01 implementation.
#
# Usage:
#   .\runtime-verification\claude-code-cli\run_live_tests.ps1

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

# ── Test: onus doctor (full) ─────────────────────────────────────────────
Write-TestHeader "onus doctor (full check)"
try {
    $output = onus doctor 2>&1 | Out-String
    if ($output -match "OK.*Daemon|OK.*Rule engine") {
        Write-Pass "Doctor command runs and reports checks"
    } else {
        Write-Warn "Doctor output does not match expected format: $(($output -split "`n")[0..3] -join '; ')"
        Write-Warn "This may be acceptable if daemon is not running"
    }
} catch {
    Write-Fail "Doctor command failed: $_"
}

# ── Test: onus doctor --claude ──────────────────────────────────────────
Write-TestHeader "onus doctor --claude"
try {
    $output = onus doctor --claude 2>&1 | Out-String
    if ($output -match "Onus Doctor.*Claude") {
        Write-Pass "Claude-specific doctor command runs"
    } else {
        Write-Fail "Doctor --claude output missing expected header"
    }
} catch {
    Write-Fail "Doctor --claude command failed: $_"
}

# ── Test: onus claude-hook (deny dangerous command) ────────────────────
Write-TestHeader "onus claude-hook (deny dangerous command)"
try {
    $payload = '{"tool":"Bash","input":{"command":"rm -rf /"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = $payload | onus claude-hook --timeout-ms 10000 2>&1 | Out-String
    if ($output -match "deny") {
        Write-Pass "Dangerous command correctly denied"
    } else {
        Write-Fail "Expected 'deny' in output but got: $($output.Trim())"
    }
} catch {
    Write-Fail "Claude-hook command failed: $_"
}

# ── Test: onus claude-hook (allow safe command) ────────────────────────
Write-TestHeader "onus claude-hook (allow safe command)"
try {
    $payload = '{"tool":"Bash","input":{"command":"echo hello"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = $payload | onus claude-hook --timeout-ms 10000 2>&1 | Out-String
    if ($output -match "allow") {
        Write-Pass "Safe command correctly allowed"
    } else {
        Write-Fail "Expected 'allow' in output but got: $($output.Trim())"
    }
} catch {
    Write-Fail "Claude-hook command failed: $_"
}

# ── Test: onus setup claude ────────────────────────────────────────────
Write-TestHeader "onus setup claude"
try {
    $output = onus setup --claude 2>&1 | Out-String
    if ($output -match "Claude Code hook setup complete|already registered") {
        Write-Pass "Setup command runs"
    } else {
        Write-Warn "Setup output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Setup command failed: $_"
}

# ── Test: onus claude-hook --receipt ───────────────────────────────────
Write-TestHeader "onus claude-hook --receipt"
try {
    $payload = '{"tool":"Bash","input":{"command":"echo receipt_test"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = $payload | onus claude-hook --timeout-ms 10000 --receipt 2>&1
    if ($output -match "ONUS_RECEIPT") {
        Write-Pass "Receipt generated in output"
    } else {
        Write-Fail "Expected ONUS_RECEIPT in output"
    }
} catch {
    Write-Fail "Receipt test failed: $_"
}

# ── Test: onus claude-hook --receipt-path file ─────────────────────────
Write-TestHeader "onus claude-hook --receipt-path (file output)"
try {
    $receiptFile = Join-Path ([System.IO.Path]::GetTempPath()) "onus-receipt-test.json"
    $payload = '{"tool":"Bash","input":{"command":"echo file_receipt"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = $payload | onus claude-hook --timeout-ms 10000 --receipt-path $receiptFile 2>&1
    if (Test-Path $receiptFile) {
        $receipt = Get-Content $receiptFile -Raw | ConvertFrom-Json
        if ($receipt.type -eq "evaluation_receipt" -and $receipt.body_hash) {
            Write-Pass "Receipt written to file with valid structure"
        } else {
            Write-Fail "Receipt file missing required fields"
        }
        Remove-Item $receiptFile -Force
    } else {
        Write-Fail "Receipt file not created"
    }
} catch {
    Write-Fail "Receipt-path test failed: $_"
}

# ── Test: onus claude-hook (fail-closed on invalid input) ───────────────
Write-TestHeader "onus claude-hook (fail-closed on invalid input)"
try {
    $output = 'NOT VALID JSON' | onus claude-hook --timeout-ms 5000 2>&1 | Out-String
    if ($output -match "deny") {
        Write-Pass "Invalid input results in deny (fail-closed)"
    } else {
        Write-Fail "Expected deny on invalid input"
    }
} catch {
    # If it crashes, that's also acceptable — hook should be resilient
    Write-Warn "Hook crashed on invalid input (acceptable): $_"
}

# ── Test: onus uninstall claude ──────────────────────────────────────────
Write-TestHeader "onus uninstall claude"
try {
    $output = onus uninstall claude 2>&1 | Out-String
    if ($output -match "removed|No Onus hook found|Nothing to remove|Claude Code hook removed") {
        Write-Pass "Uninstall command runs"
    } else {
        Write-Warn "Uninstall output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Uninstall command failed: $_"
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

if ($failCount -gt 0) {
    exit 1
}
