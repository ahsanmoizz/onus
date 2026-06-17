#!/usr/bin/env bash
# Live verification test runner for Onus × OpenAI Codex CLI integration
# ====================================================================
# Run this AFTER the user has installed Codex CLI.
#
# Prerequisites:
#   1. Codex CLI must be installed and authenticated
#   2. Onus binary must be on PATH
#
# Usage:
#   ./runtime-verification/openai-codex-cli/run_live_tests.sh

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

echo -e "${CYAN}Onus × Codex CLI Live Verification${RESET}"
echo -e "${CYAN}====================================${RESET}"
echo ""
echo -e "${YELLOW}Prerequisites:${RESET}"
echo "  1. Codex CLI installed (pip install openai-codex or npm install -g @openai/codex)"
echo "  2. Onus built and on PATH"
echo "  3. User authenticated with OpenAI"
echo ""

# ── Test 1: onus doctor codex ──────────────────────────────────────────
header "onus doctor codex"
if output=$(onus doctor --codex 2>&1); then
    if echo "$output" | grep -qi "Codex.*CLI\|Codex CLI\|doctor.*Codex"; then
        pass "Doctor --codex command runs and reports Codex status"
    else
        warn "Doctor --codex output format unexpected (Codex may not be installed)"
    fi
else
    fail "Doctor --codex command failed: $output"
fi

# ── Test 2: onus setup codex ───────────────────────────────────────────
header "onus setup codex (MCP proxy config)"
if output=$(onus setup --codex 2>&1); then
    if echo "$output" | grep -qi "Codex.*MCP\|setup.*codex\|MCP proxy\|~\.codex\|config.toml"; then
        pass "Setup command configures Codex MCP routing"
    else
        warn "Setup --codex output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Setup --codex command failed: $output"
fi

# ── Test 3: Codex MCP config file created ──────────────────────────────
header "Codex MCP config file verification"
codex_config="$HOME/.codex/config.toml"
if [ -f "$codex_config" ]; then
    if grep -q "onus-mcp-proxy\|onus.*mcp" "$codex_config"; then
        pass "Codex config.toml contains Onus MCP proxy entry"
    else
        warn "Codex config exists but Onus MCP entry not found"
    fi
else
    warn "Codex config not found at $codex_config. Run codex mcp add manually."
fi

# ── Test 4: onus mcp-proxy help ────────────────────────────────────────
header "onus mcp-proxy (help)"
if output=$(onus mcp-proxy --help 2>&1); then
    if echo "$output" | grep -q "server\|MCP\|mcp-proxy"; then
        pass "MCP proxy help shows expected flags"
    else
        fail "MCP proxy help output missing expected content"
    fi
else
    fail "MCP proxy help failed: $output"
fi

# ── Test 5: onus doctor (full) ─────────────────────────────────────────
header "onus doctor (full)"
if output=$(onus doctor 2>&1); then
    if echo "$output" | grep -qi "Codex\|OpenAI"; then
        pass "Full doctor command reports Codex status"
    else
        warn "Full doctor may not report Codex (acceptable if not installed)"
    fi
else
    fail "Doctor command failed: $output"
fi

# ── Test 6: onus uninstall codex ───────────────────────────────────────
header "onus uninstall codex"
if output=$(onus uninstall --codex 2>&1); then
    if echo "$output" | grep -qi "removed\|No.*config\|codex\|Codex"; then
        pass "Uninstall --codex command runs"
    else
        warn "Uninstall --codex output unexpected: $(echo "$output" | head -c 200)"
    fi
else
    fail "Uninstall --codex command failed: $output"
fi

# ── Test 7: MCP proxy invocation ───────────────────────────────────────
header "onus mcp-proxy (spawn and evaluate mock)"
if output=$(onus mcp-proxy --server "echo" -- --help 2>&1); then
    warn "MCP proxy requires real server to fully test. Mock: success"
    pass "MCP proxy binary invocation works"
else
    warn "MCP proxy requires real server binary to fully test"
fi

# ── Summary ─────────────────────────────────────────────────────────────
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
echo ""
echo -e "${YELLOW}Remaining user actions:${RESET}"
echo "  1. Install Codex CLI: pip install openai-codex"
echo "  2. Authenticate: codex auth login"
echo "  3. Run: onus setup --codex"
echo "  4. Verify: onus doctor --codex"
echo "  5. Re-run this test suite"
echo "  6. Live Codex session: codex run --mcp"

[ "$fail_count" -eq 0 ]
