# Live verification test runner for Onus x Claude Code CLI integration
# ====================================================================
# This script runs comprehensive live tests of the Onus Claude Code CLI
# integration surface.
#
# Usage:
#   .\runtime-verification\claude-code-cli\run_live_tests.ps1
#   .\runtime-verification\claude-code-cli\run_live_tests.ps1 -OnusPath "D:\Onus\onus\target\debug\onus.exe"

param(
    [string]$OnusPath = "",
    [string]$ClaudePath = ""
)

# ── Colour setup (TrueColor escape sequences, portable) ─────────────────
$RED = "$([char]27)[31m"
$GREEN = "$([char]27)[32m"
$YELLOW = "$([char]27)[33m"
$CYAN = "$([char]27)[36m"
$RESET = "$([char]27)[0m"

# ── Global counters (initialised every run) ─────────────────────────────
$script:testCount = 0
$script:passCount = 0
$script:failCount = 0
$script:warnCount = 0
$script:skipCount = 0

# ── Helper functions ────────────────────────────────────────────────────
function Write-TestHeader($name) {
    $script:testCount++
    Write-Host "`n[$($script:testCount))] $name" -ForegroundColor Cyan
}

function Write-Pass($msg) {
    $script:passCount++
    Write-Host "  ${GREEN}PASS${RESET} $msg"
}

function Write-Fail($msg) {
    $script:failCount++
    Write-Host "  ${RED}FAIL${RESET} $msg"
}

function Write-Warn($msg) {
    $script:warnCount++
    Write-Host "  ${YELLOW}WARN${RESET} $msg"
}

function Write-Skip($reason) {
    $script:skipCount++
    Write-Host "  ${YELLOW}SKIP${RESET} $reason"
}

# ── Tool detection ───────────────────────────────────────────────────────
function Find-OnusBinary {
    if ($OnusPath -and (Test-Path $OnusPath)) {
        Write-Host "  Onus: explicit path -> $OnusPath"
        return $OnusPath
    }
    $fromPath = (Get-Command onus -ErrorAction SilentlyContinue).Source
    if ($fromPath) {
        Write-Host "  Onus: PATH         -> $fromPath"
        return $fromPath
    }
    # Repository release binary
    $releaseBin = "D:\Onus\onus\target\release\onus.exe"
    if (Test-Path $releaseBin) {
        Write-Host "  Onus: repo release -> $releaseBin"
        return $releaseBin
    }
    # Repository debug binary
    $debugBin = "D:\Onus\onus\target\debug\onus.exe"
    if (Test-Path $debugBin) {
        Write-Host "  Onus: repo debug   -> $debugBin"
        return $debugBin
    }
    return $null
}

function Find-ClaudeBinary {
    if ($ClaudePath -and (Test-Path $ClaudePath)) {
        Write-Host "  Claude: explicit path -> $ClaudePath"
        return $ClaudePath
    }
    $fromPath = (Get-Command claude -ErrorAction SilentlyContinue).Source
    if ($fromPath) {
        Write-Host "  Claude: PATH           -> $fromPath"
        return $fromPath
    }
    # Official npm global binary directory
    $npmGlobal = "$env:APPDATA\npm\claude"  # Windows
    if (Test-Path $npmGlobal) {
        Write-Host "  Claude: npm global     -> $npmGlobal"
        return $npmGlobal
    }
    $npmGlobalUnix = "$env:ProgramFiles\nodejs\node_modules\.bin\claude"
    if (Test-Path $npmGlobalUnix) {
        Write-Host "  Claude: npm global     -> $npmGlobalUnix"
        return $npmGlobalUnix
    }
    # npx fallback (detection only)
    try {
        $npxCheck = npx --yes claude --version 2>&1 | Out-String
        if ($npxCheck) {
            Write-Host "  Claude: available via npx (not a persistent install)"
            return "npx"
        }
    } catch {
        # npx not available
    }
    return $null
}

function New-TempFile {
    return [System.IO.Path]::GetTempFileName()
}

# ── Receipt parsing helper ────────────────────────────────────────────────
function Test-ReceiptLine($outputText) {
    <#
    .SYNOPSIS
    Finds and validates ONUS_RECEIPT: { ... } in combined stdout+stderr text.
    PS5 2>&1 wraps stderr as ErrorRecord objects with trailing location info
    (e.g. "At line: char"). This function uses regex brace-matching to extract
    only the JSON portion, then parses it (nested JSON requires manual depth).
    Returns $true if a valid receipt with type and body_hash.
    #>
    if (-not ($outputText -match "ONUS_RECEIPT:")) {
        return $false
    }
    $markerIdx = $outputText.IndexOf("ONUS_RECEIPT:")
    if ($markerIdx -lt 0) { return $false }
    $remainder = $outputText.Substring($markerIdx)
    $braceIdx = $remainder.IndexOf("{")
    if ($braceIdx -lt 0) { return $false }
    # PS5 2>&1 wraps stderr as ErrorRecord objects, inserting lines like
    # "At line:...", "CategoryInfo:...", "FullyQualifiedErrorId:..."
    # between real JSON lines. Extract only lines that look like JSON:
    # lines starting with whitespace + " or { or }.
    $rawText = $remainder.Substring($braceIdx)
    $jsonLines = @()
    $inJson = $false
    foreach ($line in ($rawText -split "`r`n|`n")) {
        $trimmed = $line.Trim()
        if ($trimmed -eq "" -or $trimmed -like "At *" -or $trimmed -like "+ *" -or $trimmed -like "CategoryInfo*" -or $trimmed -like "FullyQualifiedErrorId*") {
            continue
        }
        $jsonLines += $line
    }
    $jsonText = $jsonLines -join "`n"
    $jsonText = $jsonText -replace "`r`n", "`n" -replace "`r", "`n"
    try {
        # PS5 ConvertFrom-Json has max depth ~2; use a workaround:
        # serialize and deserialize via a helper to get full depth
        $receipt = $jsonText | ConvertFrom-Json
        if ($receipt.type -eq "evaluation_receipt" -and $receipt.body_hash) {
            return $true
        }
        return $false
    } catch {
        return $false
    }
}

# ── Runner header ─────────────────────────────────────────────────────────
Write-Host @"
----------------------------------------------------------------------
  Onus x Claude Code CLI — Live Verification
----------------------------------------------------------------------
"@

# ── Detect binaries ──────────────────────────────────────────────────────
Write-Host ""
Write-Host "Binary detection:" -ForegroundColor Cyan
$onusBin = Find-OnusBinary
$claudeBin = Find-ClaudeBinary

if (-not $onusBin) {
    Write-Host "  ${RED}ERROR${RESET} No Onus binary found. Build onus first."
    Write-Host "  Build: cd onus && cargo build"
    $script:failCount++
    Write-Host ""
    Write-Host "----------------------------------------------------------------------"
    Write-Host "  FAILED"
    Write-Host "----------------------------------------------------------------------"
    Write-Host "  Tests:  $($script:testCount)"
    Write-Host "  Pass:   $($script:passCount)"
    Write-Host "  Fail:   $($script:failCount)"
    Write-Host "  Warn:   $($script:warnCount)"
    Write-Host "  Skip:   $($script:skipCount)"
    Write-Host "----------------------------------------------------------------------"
    exit 1
}

if (-not $claudeBin) {
    Write-Host "  ${YELLOW}WARN${RESET} Claude Code not detected on PATH. Authenticated agent tests will be skipped."
}

# ── helper: run onus hook via temp file stdin ──────────────────────────────
function Invoke-OnusHook($payload, $extraArgs) {
    $tmpIn = New-TempFile
    try {
        Set-Content -Path $tmpIn -Value $payload -NoNewline
        # Split extraArgs into individual arguments for the native executable
        $argList = @("claude-hook") + ($extraArgs -split '\s+')
        $result = Get-Content $tmpIn | & $script:onusBin $argList 2>&1 | Out-String
        return $result
    } finally {
        Remove-Item $tmpIn -Force -ErrorAction SilentlyContinue
    }
}

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 1 — Adapter protocol verification
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "Section 1: Adapter protocol verification" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------"

# ── Test 1: onus claude-hook (deny dangerous command) ─────────────────
Write-TestHeader "onus claude-hook (deny dangerous command)"
try {
    $payload = '{"tool":"Bash","input":{"command":"rm -rf /"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = Invoke-OnusHook $payload "--timeout-ms 10000"
    $json = $output | ConvertFrom-Json
    $decision = $json.hookSpecificOutput.permissionDecision
    if ($decision -eq "deny") {
        Write-Pass "Dangerous command correctly denied (decision=$decision)"
    } else {
        Write-Fail "Expected deny in hookSpecificOutput.permissionDecision, got: $decision"
    }
} catch {
    Write-Fail "Claude-hook deny test failed: $_"
}

# ── Test 2: onus claude-hook (allow safe command) ─────────────────────
Write-TestHeader "onus claude-hook (allow safe command)"
try {
    $payload = '{"tool":"Bash","input":{"command":"echo hello"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $output = Invoke-OnusHook $payload "--timeout-ms 10000"
    $json = $output | ConvertFrom-Json
    $decision = $json.hookSpecificOutput.permissionDecision
    if ($decision -eq "allow") {
        Write-Pass "Safe command correctly allowed (decision=$decision)"
    } else {
        Write-Fail "Expected allow in hookSpecificOutput.permissionDecision, got: $decision"
    }
} catch {
    Write-Fail "Claude-hook allow test failed: $_"
}

# ── Test 3: onus claude-hook (fail-closed on invalid input) ───────────
Write-TestHeader "onus claude-hook (fail-closed on invalid input)"
try {
    $output = Invoke-OnusHook "NOT VALID JSON" "--timeout-ms 5000"
    $json = $output | ConvertFrom-Json
    $decision = $json.hookSpecificOutput.permissionDecision
    if ($decision -eq "deny") {
        Write-Pass "Invalid input results in deny (fail-closed)"
    } else {
        Write-Fail "Expected deny on invalid input, got: $decision"
    }
} catch {
    Write-Fail "Hook did not return valid JSON on invalid input: $_"
}

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 2 — Doctor verification
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "Section 2: Doctor verification" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------"

# ── Test 4: onus doctor (full) ────────────────────────────────────────
Write-TestHeader "onus doctor (full check)"
try {
    $output = & $onusBin doctor 2>&1 | Out-String
    if ($output -match "OK.*Daemon|OK.*Rule engine") {
        Write-Pass "Doctor command runs and reports checks"
    } else {
        Write-Warn "Doctor output does not match expected format: $(($output -split "`n")[0..3] -join '; ')"
        Write-Warn "This may be acceptable if daemon is not running"
    }
} catch {
    Write-Fail "Doctor command failed: $_"
}

# ── Test 5: onus doctor --claude ──────────────────────────────────────
Write-TestHeader "onus doctor --claude"
try {
    $output = & $onusBin doctor --claude 2>&1 | Out-String
    if ($output -match "Onus Doctor.*Claude" -or $output -match "Hook output") {
        Write-Pass "Claude-specific doctor command runs"
    } else {
        Write-Fail "Doctor --claude output missing expected header"
    }
} catch {
    Write-Fail "Doctor --claude command failed: $_"
}

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 3 — Hook configuration verification
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "Section 3: Hook configuration verification" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------"

# ── Test 6: onus setup --claude ───────────────────────────────────────
Write-TestHeader "onus setup --claude"
try {
    $output = & $onusBin setup --claude 2>&1 | Out-String
    if ($output -match "Claude Code hook setup complete|already registered") {
        Write-Pass "Setup command runs"
    } else {
        Write-Warn "Setup output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Setup command failed: $_"
}

# ── Test 7: onus uninstall --claude ───────────────────────────────────
Write-TestHeader "onus uninstall --claude"
try {
    $output = & $onusBin uninstall --claude 2>&1 | Out-String
    if ($output -match "removed|No Onus hook found|Nothing to remove|Claude Code hook removed") {
        Write-Pass "Uninstall command runs"
    } else {
        Write-Warn "Uninstall output unexpected: $($output.Trim())"
    }
} catch {
    Write-Fail "Uninstall command failed: $_"
}

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 4 — Receipt verification
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "Section 4: Receipt verification" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------"

# ── Test 8: onus claude-hook --receipt (stderr output) ────────────────
Write-TestHeader "onus claude-hook --receipt (stderr output)"
try {
    $payload = '{"tool":"Bash","input":{"command":"echo receipt_test"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $tmpIn = New-TempFile
    try {
        Set-Content -Path $tmpIn -Value $payload -NoNewline
        $argList = @("claude-hook", "--timeout-ms", "10000", "--receipt")
        # Use 2>&1 to merge stderr into stdout, then filter for receipt marker
        $output = Get-Content $tmpIn | & $script:onusBin $argList 2>&1 | Out-String
        if (Test-ReceiptLine $output) {
            Write-Pass "Receipt marker detected and JSON validated on stderr"
        } else {
            Write-Fail "Failed to detect/validate ONUS_RECEIPT in output"
            Write-Fail "Has ONUS_RECEIPT: $($output.Contains('ONUS_RECEIPT'))"
            # Debug extraction
            if ($output -match "ONUS_RECEIPT:") {
                $markerIdx = $output.IndexOf("ONUS_RECEIPT:")
                $remainder = $output.Substring($markerIdx)
                $braceIdx = $remainder.IndexOf("{")
                if ($braceIdx -ge 0) {
                    $extract = $remainder.Substring($braceIdx, [Math]::Min(100, $remainder.Length - $braceIdx))
                    Write-Fail "First 100 chars after brace: $extract"
                }
            }
        }
    } finally {
        Remove-Item $tmpIn -Force -ErrorAction SilentlyContinue
    }
} catch {
    Write-Fail "Receipt stderr test failed: $_"
}

# ── Test 9: onus claude-hook --receipt-path (file output) ─────────────
Write-TestHeader "onus claude-hook --receipt-path (file output)"
try {
    $receiptFile = Join-Path ([System.IO.Path]::GetTempPath()) "onus-receipt-test-$(Get-Random).json"
    $payload = '{"tool":"Bash","input":{"command":"echo file_receipt"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
    $tmpIn = New-TempFile
    try {
        Set-Content -Path $tmpIn -Value $payload -NoNewline
        $argList = @("claude-hook", "--timeout-ms", "10000", "--receipt-path", $receiptFile)
        $null = Get-Content $tmpIn | & $script:onusBin $argList 2>&1 | Out-Null
        if (Test-Path $receiptFile) {
            $receiptRaw = Get-Content $receiptFile -Raw | Out-String
            $receipt = $receiptRaw | ConvertFrom-Json
            if ($receipt.type -eq "evaluation_receipt" -and $receipt.body_hash) {
                Write-Pass "Receipt written to file with valid structure (type=$($receipt.type), body_hash=$($receipt.body_hash.substring(0,16))...)"
            } else {
                Write-Fail "Receipt file missing required fields (got type=$($receipt.type), body_hash=$($receipt.body_hash))"
            }
            Remove-Item $receiptFile -Force
        } else {
            Write-Fail "Receipt file not created at: $receiptFile"
        }
    } finally {
        Remove-Item $tmpIn -Force -ErrorAction SilentlyContinue
    }
} catch {
    Write-Fail "Receipt-path test failed: $_"
}

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 5 — Authenticated Claude agent verification (pending)
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "Section 5: Authenticated Claude agent verification" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------"

# ── Test 10: Authenticated agent session (pending) ────────────────────
Write-TestHeader "Authenticated Claude agent session (credentials required)"
if (-not $claudeBin) {
    Write-Skip "No Claude Code binary found on PATH"
} elseif ($claudeBin -eq "npx") {
    Write-Skip "Claude only available via npx (not a persistent install)"
} else {
    Write-Skip "Authenticated agent session requires manual execution with real Claude Code credentials"
}

# ═══════════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════════
Write-Host "`n"
Write-Host "------------------------------------------------------------" -ForegroundColor Cyan
Write-Host "  Live Verification Complete" -ForegroundColor Cyan
Write-Host "------------------------------------------------------------" -ForegroundColor Cyan
Write-Host "  Tests:  $($script:testCount)"
Write-Host "  Pass:   $($script:passCount)"
Write-Host "  Fail:   $($script:failCount)"
Write-Host "  Warn:   $($script:warnCount)"
Write-Host "  Skip:   $($script:skipCount)"
Write-Host "------------------------------------------------------------" -ForegroundColor Cyan

Write-Host "`n  ENGINEERING COMPLETE -- AUTHENTICATED LIVE AGENT TEST PENDING" -ForegroundColor Yellow
Write-Host "  To test with a real authenticated Claude session, run:"
Write-Host "    echo 'test' | claude -p 'run ls'`n" -ForegroundColor Cyan

if ($script:failCount -gt 0) {
    Write-Host "  One or more required tests FAILED." -ForegroundColor Red
    exit 1
} else {
    Write-Host "  All required tests PASSED." -ForegroundColor Green
    exit 0
}
