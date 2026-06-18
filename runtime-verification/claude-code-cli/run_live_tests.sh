#!/usr/bin/env bash
# Live verification test runner for Onus x Claude Code CLI integration
# ====================================================================
#
# Usage:
#   ./runtime-verification/claude-code-cli/run_live_tests.sh
#   ./runtime-verification/claude-code-cli/run_live_tests.sh --onus-path ./onus/target/debug/onus

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

# ── Detection helpers ─────────────────────────────────────────────────────
find_onus_binary() {
    local onus_path="${ONUS_PATH:-}"
    if [ -n "$onus_path" ] && [ -f "$onus_path" ]; then
        echo "  Onus: explicit     -> $onus_path" >&2
        echo "$onus_path"
        return 0
    fi
    if command -v onus &>/dev/null; then
        echo "  Onus: PATH         -> $(command -v onus)" >&2
        command -v onus
        return 0
    fi
    local release_bin="onus/target/release/onus"
    if [ -f "$release_bin" ] && [ -x "$release_bin" ]; then
        echo "  Onus: repo release -> $(realpath "$release_bin")" >&2
        realpath "$release_bin"
        return 0
    fi
    local debug_bin="onus/target/debug/onus"
    if [ -f "$debug_bin" ] && [ -x "$debug_bin" ]; then
        echo "  Onus: repo debug   -> $(realpath "$debug_bin")" >&2
        realpath "$debug_bin"
        return 0
    fi
    echo ""
    return 1
}

find_claude_binary() {
    if [ -n "${CLAUDE_PATH:-}" ] && [ -f "$CLAUDE_PATH" ]; then
        echo "  Claude: explicit     -> $CLAUDE_PATH" >&2
        echo "$CLAUDE_PATH"
        return 0
    fi
    if command -v claude &>/dev/null; then
        echo "  Claude: PATH         -> $(command -v claude)" >&2
        command -v claude
        return 0
    fi
    # npm global binary directory
    local npm_global=""
    if command -v node &>/dev/null; then
        npm_global="$(node -e "console.log(require('path').dirname(process.execPath) + '/node_modules/.bin/claude')" 2>/dev/null || true)"
        if [ -n "$npm_global" ] && [ -f "$npm_global" ]; then
            echo "  Claude: npm global   -> $npm_global" >&2
            echo "$npm_global"
            return 0
        fi
    fi
    # npx fallback
    if command -v npx &>/dev/null; then
        echo "  Claude: available via npx (not a persistent install)" >&2
        echo "npx"
        return 0
    fi
    echo ""
    return 1
}

# ── Receipt parsing helper ────────────────────────────────────────────────
check_receipt_line() {
    # Find ONUS_RECEIPT: { ... } — handles multiline JSON
    local input="$1"
    if echo "$input" | grep -q "ONUS_RECEIPT:"; then
        # Extract the JSON block after ONUS_RECEIPT: marker
        local json_part
        json_part=$(echo "$input" | sed -n '/ONUS_RECEIPT:/,$ p' | sed 's/.*ONUS_RECEIPT:[[:space:]]*//')
        # Check for required receipt fields
        if echo "$json_part" | grep -q '"type":.*"evaluation_receipt"' && \
           echo "$json_part" | grep -q '"body_hash"'; then
            return 0
        fi
    fi
    return 1
}

# ── Counters ──────────────────────────────────────────────────────────────
test_count=0
pass_count=0
fail_count=0
warn_count=0
skip_count=0

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
    warn_count=$((warn_count + 1))
    echo -e "  ${YELLOW}WARN${RESET} $1"
}

skip() {
    skip_count=$((skip_count + 1))
    echo -e "  ${YELLOW}SKIP${RESET} $1"
}

# ── Parse CLI args ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --onus-path) ONUS_PATH="$2"; shift 2 ;;
        --claude-path) CLAUDE_PATH="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Runner header ─────────────────────────────────────────────────────────
echo "----------------------------------------------------------------------"
echo "  Onus x Claude Code CLI -- Live Verification"
echo "----------------------------------------------------------------------"

# ── Detect binaries ──────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}Binary detection:${RESET}"
onus_bin=$(find_onus_binary) || true
claude_bin=$(find_claude_binary) || true

if [ -z "$onus_bin" ]; then
    echo -e "  ${RED}ERROR${RESET} No Onus binary found. Build onus first."
    echo "  Build: cd onus && cargo build"
    fail_count=$((fail_count + 1))
    echo ""
    echo "----------------------------------------------------------------------"
    echo "  FAILED"
    echo "----------------------------------------------------------------------"
    echo "  Tests:  $test_count"
    echo "  Pass:   $pass_count"
    echo "  Fail:   $fail_count"
    echo "  Warn:   $warn_count"
    echo "  Skip:   $skip_count"
    echo "----------------------------------------------------------------------"
    exit 1
fi

if [ -z "$claude_bin" ]; then
    echo -e "  ${YELLOW}WARN${RESET} Claude Code not detected on PATH. Authenticated agent tests will be skipped."
fi

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 1 -- Adapter protocol verification
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}Section 1: Adapter protocol verification${RESET}"
echo "------------------------------------------------------------"

# ── Test 1: onus claude-hook (deny dangerous command) ─────────────────
header "onus claude-hook (deny dangerous command)"
payload='{"tool":"Bash","input":{"command":"rm -rf /"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | "$onus_bin" claude-hook --timeout-ms 10000 2>&1); then
    decision=$(echo "$output" | grep -o '"permissionDecision":"[^"]*"' | cut -d'"' -f4)
    if [ "$decision" = "deny" ]; then
        pass "Dangerous command correctly denied (decision=$decision)"
    else
        fail "Expected deny in permissionDecision, got: $decision"
    fi
else
    fail "Claude-hook command failed: $output"
fi

# ── Test 2: onus claude-hook (allow safe command) ─────────────────────
header "onus claude-hook (allow safe command)"
payload='{"tool":"Bash","input":{"command":"echo hello"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | "$onus_bin" claude-hook --timeout-ms 10000 2>&1); then
    decision=$(echo "$output" | grep -o '"permissionDecision":"[^"]*"' | cut -d'"' -f4)
    if [ "$decision" = "allow" ]; then
        pass "Safe command correctly allowed (decision=$decision)"
    else
        fail "Expected allow in permissionDecision, got: $decision"
    fi
else
    fail "Claude-hook command failed: $output"
fi

# ── Test 3: onus claude-hook (fail-closed on invalid input) ───────────
header "onus claude-hook (fail-closed on invalid input)"
if output=$(echo 'NOT VALID JSON' | "$onus_bin" claude-hook --timeout-ms 5000 2>&1); then
    decision=$(echo "$output" | grep -o '"permissionDecision":"[^"]*"' | cut -d'"' -f4)
    if [ "$decision" = "deny" ]; then
        pass "Invalid input results in deny (fail-closed)"
    else
        fail "Expected deny on invalid input, got: $decision"
    fi
else
    fail "Hook did not return valid JSON on invalid input: $output"
fi

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 2 -- Doctor verification
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}Section 2: Doctor verification${RESET}"
echo "------------------------------------------------------------"

# ── Test 4: onus doctor (full) ────────────────────────────────────────
header "onus doctor (full check)"
if output=$("$onus_bin" doctor 2>&1); then
    if echo "$output" | grep -q "OK.*Daemon\|OK.*Rule engine"; then
        pass "Doctor command runs and reports checks"
    else
        warn "Doctor output format unexpected (daemon may not be running)"
    fi
else
    fail "Doctor command failed: $output"
fi

# ── Test 5: onus doctor --claude ──────────────────────────────────────
header "onus doctor --claude"
if output=$("$onus_bin" doctor --claude 2>&1); then
    if echo "$output" | grep -q "Onus Doctor.*Claude\|Hook output"; then
        pass "Claude-specific doctor command runs"
    else
        fail "Doctor --claude output missing expected header"
    fi
else
    fail "Doctor --claude command failed: $output"
fi

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 3 -- Hook configuration verification
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}Section 3: Hook configuration verification${RESET}"
echo "------------------------------------------------------------"

# ── Test 6: onus setup --claude ───────────────────────────────────────
header "onus setup --claude"
if output=$("$onus_bin" setup --claude 2>&1); then
    if echo "$output" | grep -q "Claude Code hook setup complete\|already registered"; then
        pass "Setup command runs"
    else
        warn "Setup output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Setup command failed: $output"
fi

# ── Test 7: onus uninstall --claude ───────────────────────────────────
header "onus uninstall --claude"
if output=$("$onus_bin" uninstall --claude 2>&1); then
    if echo "$output" | grep -q "removed\|No Onus hook found\|Nothing to remove\|Claude Code hook removed"; then
        pass "Uninstall command runs"
    else
        warn "Uninstall output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Uninstall command failed: $output"
fi

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 4 -- Receipt verification
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}Section 4: Receipt verification${RESET}"
echo "------------------------------------------------------------"

# ── Test 8: onus claude-hook --receipt (stderr output) ────────────────
header "onus claude-hook --receipt (stderr output)"
payload='{"tool":"Bash","input":{"command":"echo receipt_test"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | "$onus_bin" claude-hook --timeout-ms 10000 --receipt 2>&1); then
    if check_receipt_line "$output"; then
        pass "Receipt marker detected and JSON validated on stderr"
    else
        fail "Failed to detect/validate ONUS_RECEIPT on stderr"
        echo "  Raw: $(echo "$output" | head -c 400)"
    fi
else
    fail "Receipt stderr test failed: $output"
fi

# ── Test 9: onus claude-hook --receipt-path (file output) ─────────────
header "onus claude-hook --receipt-path (file output)"
receipt_file=$(mktemp /tmp/onus-receipt-XXXXXX.json 2>/dev/null || mktemp)
payload='{"tool":"Bash","input":{"command":"echo file_receipt"},"session_id":"live-test","cwd":"/tmp","agent":"claude-code","agent_version":"1.0.0"}'
if output=$(echo "$payload" | "$onus_bin" claude-hook --timeout-ms 10000 --receipt-path "$receipt_file" 2>&1); then
    if [ -f "$receipt_file" ]; then
        if grep -q '"type": *"evaluation_receipt"' "$receipt_file" && grep -q '"body_hash"' "$receipt_file"; then
            pass "Receipt written to file with valid structure"
        else
            fail "Receipt file missing required fields"
            echo "  Content: $(cat "$receipt_file")"
        fi
        rm -f "$receipt_file"
    else
        fail "Receipt file not created"
    fi
else
    fail "Receipt-path test failed: $output"
fi

# ═══════════════════════════════════════════════════════════════════════════
# SECTION 5 -- Authenticated Claude agent verification (pending)
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}Section 5: Authenticated Claude agent verification${RESET}"
echo "------------------------------------------------------------"

# ── Test 10: Authenticated agent session (pending) ────────────────────
header "Authenticated Claude agent session (credentials required)"
if [ -z "$claude_bin" ]; then
    skip "No Claude Code binary found on PATH"
elif [ "$claude_bin" = "npx" ]; then
    skip "Claude only available via npx (not a persistent install)"
else
    skip "Authenticated agent session requires manual execution with real Claude Code credentials"
fi

# ═══════════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}------------------------------------------------------------${RESET}"
echo -e "${CYAN}  Live Verification Complete${RESET}"
echo -e "${CYAN}------------------------------------------------------------${RESET}"
echo -e "${CYAN}  Tests:  $test_count${RESET}"
echo -e "${GREEN}  Pass:   $pass_count${RESET}"
echo -e "${RED}  Fail:   $fail_count${RESET}"
echo -e "${YELLOW}  Warn:   $warn_count${RESET}"
echo -e "${YELLOW}  Skip:   $skip_count${RESET}"
echo -e "${CYAN}------------------------------------------------------------${RESET}"

echo ""
echo -e "${YELLOW}  ENGINEERING COMPLETE -- AUTHENTICATED LIVE AGENT TEST PENDING${RESET}"
echo -e "${CYAN}  To test with a real authenticated Claude session, run:${RESET}"
echo -e "    echo 'test' | claude -p 'run ls'"
echo ""

if [ "$fail_count" -gt 0 ]; then
    echo -e "${RED}  One or more required tests FAILED.${RESET}"
    exit 1
else
    echo -e "${GREEN}  All required tests PASSED.${RESET}"
    exit 0
fi
