#!/usr/bin/env bash
# Onus Antigravity Live Verification Runner
# Run: bash runtime-verification/google-antigravity/run_live_tests.sh
set -euo pipefail

ONUS_BIN="${ONUS_BIN:-./target/debug/onus.exe}"
ANTIGRAVITY_BIN="${ANTIGRAVITY_BIN:-antigravity}"
PASS=0
FAIL=0

green() { printf "  \033[32m✓\033[0m %s\n" "$1"; }
red()   { printf "  \033[31m✗\033[0m %s\n" "$1"; }

# Prerequisite check
if [ ! -f "$ONUS_BIN" ]; then
    echo "Onus binary not found at: $ONUS_BIN"
    echo "Run 'cargo build' first or set ONUS_BIN"
    exit 1
fi

echo ""
echo "=== Onus Antigravity Live Verification ==="
echo ""

# Test 1: Antigravity doctor
if "$ONUS_BIN" doctor --antigravity 2>&1 | grep -q "Antigravity"; then
    green "Test 1: onus doctor --antigravity reports Antigravity"
    PASS=$((PASS + 1))
else
    red "Test 1: onus doctor --antigravity failed"
    FAIL=$((FAIL + 1))
fi

# Test 2: Full doctor includes Antigravity
if "$ONUS_BIN" doctor 2>&1 | grep -qi "antigravity"; then
    green "Test 2: Full onus doctor includes Antigravity section"
    PASS=$((PASS + 1))
else
    red "Test 2: Full onus doctor missing Antigravity"
    FAIL=$((FAIL + 1))
fi

# Test 3: Setup --antigravity
if "$ONUS_BIN" setup --antigravity 2>&1 | grep -q "Antigravity"; then
    green "Test 3: onus setup --antigravity succeeds"
    PASS=$((PASS + 1))
else
    red "Test 3: onus setup --antigravity failed"
    FAIL=$((FAIL + 1))
fi

# Test 4: Antigravity version check
if "$ANTIGRAVITY_BIN" --version 2>&1 | grep -q "1.107"; then
    green "Test 4: Antigravity --version shows v1.107"
    PASS=$((PASS + 1))
else
    red "Test 4: Antigravity --version unexpected"
    FAIL=$((FAIL + 1))
fi

# Test 5: Extension listed
if "$ANTIGRAVITY_BIN" --list-extensions 2>&1 | grep -q "onus.onus-firewall"; then
    green "Test 5: Antigravity lists onus-firewall extension"
    PASS=$((PASS + 1))
else
    red "Test 5: onus-firewall not listed in extensions"
    FAIL=$((FAIL + 1))
fi

# Test 6: Uninstall --antigravity
if "$ONUS_BIN" uninstall --antigravity 2>&1 | grep -qi "antigravity"; then
    green "Test 6: onus uninstall --antigravity succeeds"
    PASS=$((PASS + 1))
else
    red "Test 6: onus uninstall --antigravity failed"
    FAIL=$((FAIL + 1))
fi

# Test 7: L3 workspace advice
if "$ONUS_BIN" doctor --antigravity 2>&1 | grep -iqE "bubblewrap|workspace|linux"; then
    green "Test 7: Antigravity doctor includes L3 workspace info"
    PASS=$((PASS + 1))
else
    red "Test 7: Antigravity doctor missing L3 workspace info"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "  $PASS passed, $FAIL failed, $((PASS + FAIL)) total"
echo ""

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
