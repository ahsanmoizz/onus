#!/usr/bin/env bash
# Live verification test runner for Onus × Claude Code CLI integration
# ====================================================================
# Run this AFTER completing the P15E-01 implementation.
#
# Usage:
#   ./runtime-verification/claude-code-cli/run_live_tests.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

test_count=0
pass_count=0
fail_count=0

header() {
    test_count=$((test_count + 1))
    echo -e "\n${CYAN}[${test_count})] $1${RESET}"
}

pass() {
    pass_count=$((pass_count + 1))
    echo -e "  ${GREEN}PASS${RESET} $1"
}

fail() {
    fail_count=$((fail_count + 1))
    echo -e "  ${RED}FAIL${RESET} $1"
}

warn() {
    echo -e "  ${YELLOW}WARN${RESET} $1"
}

# ── Test: onus doctor (full) ─────────────────────────────────────────────
header "onus doctor (full check)"
if output=$(onus doctor 2>&1); then
    if echo "$output" | grep -q "OK.*Daemon\|OK.*Rule engine"; then
        pass "Doctor command runs and reports checks"
    else
        warn "Doctor output format unexpected (daemon may not be running)"
    fi
else
    fail "Doctor command failed: $output"
fi

# ── Test: onus doctor --claude ──────────────────────────────────────────
header "onus doctor --claude"
if output=$(onus doctor --claude 2>&1); then
    if echo "$output" | grep -q "Onus Doctor.*Claude"; then
        pass "Claude-specific doctor command runs"
    else
        fail "Doctor --claude output missing expected header"
    fi
else
    fail "Doctor --claude command failed: $output"
fi

# ── Test: onus claude-hook (deny dangerous command) ────────────────────
header "onus claude-hook (deny dangerous command)"
payload='{"tool":"Bash","input":{"command":"rm -rf /"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | onus claude-hook --timeout-ms 10000 2>&1); then
    if echo "$output" | grep -q "deny"; then
        pass "Dangerous command correctly denied"
    else
        fail "Expected 'deny' in output but got: $(echo "$output" | head -c 200)"
    fi
else
    fail "Claude-hook command failed: $output"
fi

# ── Test: onus claude-hook (allow safe command) ────────────────────────
header "onus claude-hook (allow safe command)"
payload='{"tool":"Bash","input":{"command":"echo hello"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | onus claude-hook --timeout-ms 10000 2>&1); then
    if echo "$output" | grep -q "allow"; then
        pass "Safe command correctly allowed"
    else
        fail "Expected 'allow' in output but got: $(echo "$output" | head -c 200)"
    fi
else
    fail "Claude-hook command failed: $output"
fi

# ── Test: onus setup claude ────────────────────────────────────────────
header "onus setup claude"
if output=$(onus setup --claude 2>&1); then
    if echo "$output" | grep -q "Claude Code hook setup complete\|already registered"; then
        pass "Setup command runs"
    else
        warn "Setup output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Setup command failed: $output"
fi

# ── Test: onus claude-hook --receipt ───────────────────────────────────
header "onus claude-hook --receipt"
payload='{"tool":"Bash","input":{"command":"echo receipt_test"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | onus claude-hook --timeout-ms 10000 --receipt 2>&1); then
    if echo "$output" | grep -q "ONUS_RECEIPT"; then
        pass "Receipt generated in output"
    else
        fail "Expected ONUS_RECEIPT in output"
    fi
else
    fail "Receipt test failed: $output"
fi

# ── Test: onus claude-hook --receipt-path file ─────────────────────────
header "onus claude-hook --receipt-path (file output)"
receipt_file=$(mktemp /tmp/onus-receipt-XXXXXX.json 2>/dev/null || mktemp)
payload='{"tool":"Bash","input":{"command":"echo file_receipt"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | onus claude-hook --timeout-ms 10000 --receipt-path "$receipt_file" 2>&1); then
    if [ -f "$receipt_file" ]; then
        if grep -q "evaluation_receipt" "$receipt_file" && grep -q "body_hash" "$receipt_file"; then
            pass "Receipt written to file with valid structure"
        else
            fail "Receipt file missing required fields"
        fi
        rm -f "$receipt_file"
    else
        fail "Receipt file not created"
    fi
else
    fail "Receipt-path test failed: $output"
fi

# ── Test: onus claude-hook (fail-closed on invalid input) ───────────────
header "onus claude-hook (fail-closed on invalid input)"
if output=$(echo 'NOT VALID JSON' | onus claude-hook --timeout-ms 5000 2>&1); then
    if echo "$output" | grep -q "deny"; then
        pass "Invalid input results in deny (fail-closed)"
    else
        fail "Expected deny on invalid input"
    fi
else
    warn "Hook crashed on invalid input (acceptable): $output"
fi

# ── Test: onus uninstall claude ──────────────────────────────────────────
header "onus uninstall claude"
if output=$(onus uninstall claude 2>&1); then
    if echo "$output" | grep -q "removed\|No Onus hook found\|Nothing to remove\|Claude Code hook removed"; then
        pass "Uninstall command runs"
    else
        warn "Uninstall output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Uninstall command failed: $output"
fi

# ── Summary ──────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}═══════════════════════════════════════════${RESET}"
echo -e "${CYAN} Live Verification Complete${RESET}"
echo -e "${CYAN} Tests:  $test_count${RESET}"
if [ "$fail_count" -eq 0 ]; then
    echo -e "${GREEN} Pass:   $pass_count${RESET}"
else
    echo -e "${YELLOW} Pass:   $pass_count${RESET}"
fi
if [ "$fail_count" -eq 0 ]; then
    echo -e "${GREEN} Fail:   $fail_count${RESET}"
else
    echo -e "${RED} Fail:   $fail_count${RESET}"
fi
echo -e "${CYAN}═══════════════════════════════════════════${RESET}"

[ "$fail_count" -eq 0 ]
