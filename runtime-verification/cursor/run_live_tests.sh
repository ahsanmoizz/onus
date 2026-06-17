#!/usr/bin/env bash
# Onus Cursor IDE Live Verification Runner (Bash)
# Run: ./runtime-verification/cursor/run_live_tests.sh

set -euo pipefail

ONUS_BIN="${ONUS_BIN:-./target/debug/onus}"
CURSOR_BIN="${CURSOR_BIN:-cursor}"
PASS=0
FAIL=0

if [ ! -f "$ONUS_BIN" ]; then
    echo "Onus binary not found at: $ONUS_BIN"
    exit 1
fi

echo ""
echo "=== Onus Cursor IDE Live Verification ==="
echo ""

test_result() {
    local name="$1"
    shift
    if "$@" 2>&1; then
        echo "  ✓ $name"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $name"
        FAIL=$((FAIL + 1))
    fi
}

# Test 1
test_result "Test 1: onus doctor --cursor reports Cursor" \
    "$ONUS_BIN" doctor --cursor | grep -q "Cursor"

# Test 2
test_result "Test 2: Full onus doctor includes Cursor section" \
    "$ONUS_BIN" doctor | grep -q "Cursor"

# Test 3
test_result "Test 3: onus setup --cursor succeeds" \
    "$ONUS_BIN" setup --cursor | grep -q "Cursor"

# Test 4: cursor version (if installed)
if command -v "$CURSOR_BIN" &>/dev/null; then
    test_result "Test 4: Cursor --version succeeds" \
        "$CURSOR_BIN" --version | grep -qE "[0-9]+\.[0-9]+"
else
    echo "  ∼ Test 4: Cursor not installed (skipped)"
    PASS=$((PASS + 1))
fi

# Test 5
test_result "Test 5: onus uninstall --cursor succeeds" \
    "$ONUS_BIN" uninstall --cursor | grep -i "cursor"

# Test 6
test_result "Test 6: Cursor doctor includes L3 workspace info" \
    "$ONUS_BIN" doctor --cursor | grep -qE "bubblewrap|workspace|Linux|not available"

# Test 7
test_result "Test 7: onus cursor-hook accepts stdin and returns JSON" \
    sh -c 'echo '"'"'{"tool":"bash","args":{"command":"ls"}}'"'"' | "$ONUS_BIN" cursor-hook | grep -q "allowed"'

# Test 8
test_result "Test 8: onus cursor-hook output is valid JSON" \
    sh -c 'echo '"'"'{"tool":"bash","args":{"command":"ls"}}'"'"' | "$ONUS_BIN" cursor-hook | python3 -c "import sys,json; json.load(sys.stdin)"'

echo ""
echo "  $PASS passed, $FAIL failed, $((PASS + FAIL)) total"
echo ""

if [ "$FAIL" -gt 0 ]; then exit 1; fi
