#!/usr/bin/env bash
# Onus — AI Agent Firewall Uninstaller (Linux)
# Removes Onus binary, PATH entries, and optionally configuration.
# Preserves audit data by default.
#
# Usage:
#   ./uninstall-onus.sh
#   ./uninstall-onus.sh --purge
#   ./uninstall-onus.sh --no-interactive

set -euo pipefail

# ── Configuration ──
PURGE=0
NO_INTERACTIVE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --purge) PURGE=1; shift ;;
        --no-interactive) NO_INTERACTIVE=1; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

ok()   { echo -e "  ${GREEN}[OK]${NC} $1"; }
warn() { echo -e "  ${YELLOW}[WARN]${NC} $1"; }
err()  { echo -e "  ${RED}[FAIL]${NC} $1"; }
step() { echo -e "  ${CYAN}>>${NC} $1"; }

echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        Onus — AI Agent Firewall             ║${NC}"
echo -e "${CYAN}║        Linux Uninstaller                     ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# ── Detect installation ──
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/onus"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/onus"
BINARY_PATH="${INSTALL_DIR}/onus"

# Check system install too
SYS_BINARY="/usr/local/bin/onus"
SYS_CONFIG="/etc/onus"

FOUND=0
[ -f "$BINARY_PATH" ] && FOUND=1 && echo "  Binary: $BINARY_PATH"
[ -f "$SYS_BINARY" ] && FOUND=1 && echo "  Binary: $SYS_BINARY" && BINARY_PATH="$SYS_BINARY" && INSTALL_DIR="/usr/local/bin"
[ -d "$CONFIG_DIR" ] && echo "  Config: $CONFIG_DIR"
[ -d "$DATA_DIR" ] && echo "  Data:   $DATA_DIR"
[ -d "$SYS_CONFIG" ] && echo "  Config: $SYS_CONFIG" && CONFIG_DIR="$SYS_CONFIG"

if [ "$FOUND" -eq 0 ]; then
    echo "Onus does not appear to be installed. Nothing to remove."
    exit 0
fi

echo ""
if [ "$PURGE" = 1 ]; then
    echo -e "  Mode: ${RED}PURGE${NC} (all data including audit trail will be deleted)"
else
    echo -e "  Mode: ${GREEN}STANDARD${NC} (audit data and configuration preserved)"
    echo "  Use --purge to delete all configuration and audit data."
fi
echo ""

# ── Confirm ──
if [ "$NO_INTERACTIVE" != 1 ]; then
    read -r -p "Remove Onus? (y/N) " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Uninstall cancelled."
        exit 0
    fi
fi

# ── Stop daemon if running ──
if [ -f "$BINARY_PATH" ]; then
    step "Stopping Onus daemon if running..."
    "$BINARY_PATH" daemon stop 2>/dev/null || true
    sleep 1
fi

# ── Remove binary ──
step "Removing binary..."
if [ -f "$BINARY_PATH" ]; then
    rm -f "$BINARY_PATH"
    ok "Removed $BINARY_PATH"
fi

# Remove bin dir if empty
BIN_DIR="$(dirname "$BINARY_PATH")"
if [ -d "$BIN_DIR" ] && [ -z "$(ls -A "$BIN_DIR" 2>/dev/null)" ]; then
    rmdir "$BIN_DIR" 2>/dev/null || true
fi

# ── Remove configuration (only if --purge) ──
if [ "$PURGE" = 1 ]; then
    step "Purging configuration and data..."
    [ -d "$CONFIG_DIR" ] && rm -rf "$CONFIG_DIR" && ok "Removed $CONFIG_DIR"
    [ -d "$DATA_DIR" ] && rm -rf "$DATA_DIR" && ok "Removed $DATA_DIR"
else
    step "Preserving configuration and audit data:"
    echo "    Config: $CONFIG_DIR"
    echo "    Data:   $DATA_DIR"
fi

echo ""
echo -e "${GREEN}Onus has been removed.${NC}"
if [ "$PURGE" != 1 ]; then
    echo "To reinstall: run install-onus.sh"
    echo "To also remove audit data: run with --purge"
fi
echo ""

exit 0
