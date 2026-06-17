# Onus Antigravity Live Verification Runner (PowerShell)
# Run: .\runtime-verification\google-antigravity\run_live_tests.ps1

$ONUS_BIN = if ($env:ONUS_BIN) { $env:ONUS_BIN } else { ".\target\debug\onus.exe" }
$ANTIGRAVITY_BIN = if ($env:ANTIGRAVITY_BIN) { $env:ANTIGRAVITY_BIN } else { "D:\Antigravity\bin\antigravity" }
$PASS = 0
$FAIL = 0

function Test-Result($name, $script) {
    $result = & $script 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ $name" -ForegroundColor Green
        $script:PASS += 1
    } else {
        Write-Host "  ✗ $name" -ForegroundColor Red
        Write-Host "    $result" -ForegroundColor DarkRed
        $script:FAIL += 1
    }
}

if (-not (Test-Path $ONUS_BIN)) {
    Write-Host "Onus binary not found at: $ONUS_BIN" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Onus Antigravity Live Verification ===`n"

# Test 1: Doctor --antigravity
Test-Result "Test 1: onus doctor --antigravity reports Antigravity" {
    $out = & $ONUS_BIN doctor --antigravity 2>&1 | Out-String
    if ($out -match "Antigravity") { return $true } else { return $false }
}

# Test 2: Full doctor includes Antigravity
Test-Result "Test 2: Full onus doctor includes Antigravity section" {
    $out = & $ONUS_BIN doctor 2>&1 | Out-String
    if ($out -match "antigravity" -or $out -match "Antigravity") { return $true } else { return $false }
}

# Test 3: Setup --antigravity
Test-Result "Test 3: onus setup --antigravity succeeds" {
    $out = & $ONUS_BIN setup --antigravity 2>&1 | Out-String
    if ($out -match "Antigravity") { return $true } else { return $false }
}

# Test 4: Antigravity version
Test-Result "Test 4: Antigravity --version succeeds" {
    $out = & $ANTIGRAVITY_BIN --version 2>&1 | Out-String
    if ($out -match "1.107") { return $true } else { return $false }
}

# Test 5: Extension listed
Test-Result "Test 5: Antigravity lists onus-firewall extension" {
    $out = & $ANTIGRAVITY_BIN --list-extensions 2>&1 | Out-String
    if ($out -match "onus.onus-firewall") { return $true } else { return $false }
}

# Test 6: Uninstall --antigravity
Test-Result "Test 6: onus uninstall --antigravity succeeds" {
    $out = & $ONUS_BIN uninstall --antigravity 2>&1 | Out-String
    if ($out -match "antigravity" -or $out -match "Antigravity") { return $true } else { return $false }
}

# Test 7: L3 workspace advice
Test-Result "Test 7: Antigravity doctor includes L3 workspace info" {
    $out = & $ONUS_BIN doctor --antigravity 2>&1 | Out-String
    if ($out -match "bubblewrap|workspace|Linux|not available") { return $true } else { return $false }
}

# Test 8: Extension directory exists
Test-Result "Test 8: Extension directory exists" {
    $extDir = "$env:USERPROFILE\.antigravity\extensions\onus.onus-firewall-0.1.0"
    if (Test-Path $extDir) { return $true } else { return $false }
}

Write-Host "`n  $PASS passed, $FAIL failed, $($PASS + $FAIL) total`n"

if ($FAIL -gt 0) { exit 1 }
