# Onus Cursor IDE Live Verification Runner (PowerShell)
# Run: .\runtime-verification\cursor\run_live_tests.ps1

$ONUS_BIN = if ($env:ONUS_BIN) { $env:ONUS_BIN } else { ".\target\debug\onus.exe" }
$PASS = 0
$FAIL = 0

function Test-Result {
    param($name, $script)
    $result = & $script 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  OK $name" -ForegroundColor Green
        $script:PASS = $script:PASS + 1
    } else {
        Write-Host "  FAIL $name" -ForegroundColor Red
        Write-Host "    $result" -ForegroundColor DarkRed
        $script:FAIL = $script:FAIL + 1
    }
}

if (-not (Test-Path $ONUS_BIN)) {
    Write-Host "Onus binary not found at: $ONUS_BIN" -ForegroundColor Red
    exit 1
}

$CURSOR_BIN = if ($env:CURSOR_BIN) { $env:CURSOR_BIN } else { "cursor" }

Write-Host ""
Write-Host "=== Onus Cursor IDE Live Verification ==="
Write-Host ""

# Test 1: Doctor --cursor
Test-Result "Test 1: onus doctor --cursor reports Cursor" {
    $out = & $ONUS_BIN doctor --cursor 2>&1 | Out-String
    if ($out -match "Cursor") { return $true } else { return $false }
}

# Test 2: Full doctor includes Cursor
Test-Result "Test 2: Full onus doctor includes Cursor section" {
    $out = & $ONUS_BIN doctor 2>&1 | Out-String
    if ($out -match "Cursor") { return $true } else { return $false }
}

# Test 3: Setup --cursor
Test-Result "Test 3: onus setup --cursor succeeds" {
    $out = & $ONUS_BIN setup --cursor 2>&1 | Out-String
    if ($out -match "Cursor") { return $true } else { return $false }
}

# Test 4: Cursor version (if installed)
Test-Result "Test 4: cursor --version (if installed)" {
    $out = & $CURSOR_BIN --version 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0 -and $out -match "\d+\.\d+") { return $true }
    if ($LASTEXITCODE -ne 0) { return $true }
    return $false
}

# Test 5: Uninstall --cursor
Test-Result "Test 5: onus uninstall --cursor succeeds" {
    $out = & $ONUS_BIN uninstall --cursor 2>&1 | Out-String
    if ($out -match "Cursor" -or $out -match "cursor") { return $true } else { return $false }
}

# Test 6: L3 workspace advice in Cursor doctor
Test-Result "Test 6: Cursor doctor includes L3 workspace info" {
    $out = & $ONUS_BIN doctor --cursor 2>&1 | Out-String
    if ($out -match "bubblewrap|workspace|Linux|not available") { return $true } else { return $false }
}

# Test 7: cursor-hook subcommand (basic JSON I/O)
Test-Result "Test 7: onus cursor-hook accepts stdin and returns JSON" {
    $inputJson = '{"tool":"bash","args":{"command":"ls"}}'
    $out = $inputJson | & $ONUS_BIN cursor-hook 2>&1 | Out-String
    if ($out -match '"allowed"') { return $true } else { return $false }
}

# Test 8: cursor-hook returns valid JSON
Test-Result "Test 8: onus cursor-hook output is valid JSON" {
    $inputJson = '{"tool":"bash","args":{"command":"ls"}}'
    $raw = $inputJson | & $ONUS_BIN cursor-hook 2>&1
    try { $parsed = $raw | ConvertFrom-Json; return $true } catch { return $false }
}

$TOTAL = $PASS + $FAIL
Write-Host ""
Write-Host ("  " + $PASS + " passed, " + $FAIL + " failed, " + $TOTAL + " total")
Write-Host ""

if ($FAIL -gt 0) { exit 1 }
