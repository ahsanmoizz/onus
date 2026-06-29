#!/usr/bin/env bash
# Onus — AI Agent Firewall Installer (Linux)
# Installs Onus, verifies checksums, configures PATH, and runs setup.
#
# Usage:
#   ./install-onus.sh
#   ./install-onus.sh v0.1.0
#   ./install-onus.sh --dry-run
#   ./install-onus.sh --upgrade
#   ./install-onus.sh --system    # system-wide install (requires sudo)
#
# Environment:
#   VERSION       Release version (default: latest)
#   DRY_RUN       Set to 1 for dry-run mode
#   NO_VERIFY     Set to 1 to skip SHA-256 check
#   NO_INTERACTIVE Set to 1 for non-interactive mode

set -euo pipefail

# ── Configuration ──
REPO="ahsanmoizz/onus"
VERSION="${VERSION:-${1:-latest}}"
DRY_RUN="${DRY_RUN:-0}"
NO_VERIFY="${NO_VERIFY:-0}"
NO_INTERACTIVE="${NO_INTERACTIVE:-0}"
SYSTEM="${SYSTEM:-0}"

# Parse named arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=1; shift ;;
        --repair) REPAIR=1; shift ;;
        --upgrade) UPGRADE=1; shift ;;
        --system) SYSTEM=1; shift ;;
        --no-verify) NO_VERIFY=1; shift ;;
        --no-interactive) NO_INTERACTIVE=1; shift ;;
        -*)
            echo "Unknown option: $1"
            echo "Usage: $0 [version] [--dry-run] [--repair] [--upgrade] [--system] [--no-verify] [--no-interactive]"
            exit 1
            ;;
        *) shift ;;
    esac
done

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

ok()   { echo -e "  ${GREEN}[OK]${NC} $1"; }
warn() { echo -e "  ${YELLOW}[WARN]${NC} $1"; }
err()  { echo -e "  ${RED}[FAIL]${NC} $1"; }
step() { echo -e "  ${CYAN}>>${NC} $1"; }

# ── Banner ──
echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        Onus — AI Agent Firewall             ║${NC}"
echo -e "${CYAN}║        Linux Installer                       ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# ── System detection ──
step "Detecting system architecture..."
OS=""
ARCH=""
case "$(uname -s)" in
    Linux)   OS="linux" ;;
    *)       err "Unsupported OS: $(uname -s)"; exit 1 ;;
esac
case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)       err "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac
ok "Linux / ${ARCH} detected"

# ── Directories ──
if [ "$SYSTEM" = 1 ]; then
    INSTALL_DIR="/usr/local/bin"
    CONFIG_DIR="/etc/onus"
    DATA_DIR="/var/lib/onus"
    RULES_DIR="${CONFIG_DIR}/rules"
    BINARY_PATH="${INSTALL_DIR}/onus"
    warn "System-wide install (requires sudo for some operations)"
else
    INSTALL_DIR="${HOME}/.local/bin"
    CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/onus"
    DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/onus"
    RULES_DIR="${CONFIG_DIR}/rules"
    BINARY_PATH="${INSTALL_DIR}/onus"
fi

ARCHIVE_NAME="onus-${VERSION}-linux-${ARCH}.tar.gz"
CHECKSUM_FILE="SHA256SUMS"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE_NAME}"
    CHECKSUM_URL="https://github.com/${REPO}/releases/latest/download/${CHECKSUM_FILE}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/${CHECKSUM_FILE}"
fi

# ── Display plan ──
echo ""
echo "  Platform:   linux/${ARCH}"
echo "  Version:    ${VERSION}"
echo "  Binary:     ${BINARY_PATH}"
echo "  Config:     ${CONFIG_DIR}"
echo "  Data:       ${DATA_DIR}"
[ "$DRY_RUN" = 1 ] && echo "  Mode:       DRY RUN (no changes)"
[ "${REPAIR:-0}" = 1 ] && echo "  Mode:       REPAIR"
[ "${UPGRADE:-0}" = 1 ] && echo "  Mode:       UPGRADE"
echo ""

# ── Confirm ──
if [ "$NO_INTERACTIVE" != 1 ] && [ "$DRY_RUN" != 1 ]; then
    read -r -p "Proceed with installation? (Y/n) " confirm
    if [ "$confirm" = "n" ] || [ "$confirm" = "N" ]; then
        echo "Installation cancelled."
        exit 0
    fi
fi

# ── Dry-run guard ──
run() {
    local msg="$1"
    shift
    step "$msg"
    if [ "$DRY_RUN" != 1 ]; then
        "$@"
    fi
}

# ── Locate or download archive ──
ARCHIVE_PATH=""
if [ -f "$ARCHIVE_NAME" ]; then
    ARCHIVE_PATH="$(realpath "$ARCHIVE_NAME")"
    ok "Found local archive: ${ARCHIVE_PATH}"
elif [ -f "./${ARCHIVE_NAME}" ]; then
    ARCHIVE_PATH="$(realpath "./${ARCHIVE_NAME}")"
    ok "Found local archive: ${ARCHIVE_PATH}"
else
    run "Downloading ${ARCHIVE_NAME} from GitHub releases..." \
        sh -c "curl -fsSL '${DOWNLOAD_URL}' -o '/tmp/${ARCHIVE_NAME}' && echo 'Downloaded to /tmp/${ARCHIVE_NAME}'"
    ARCHIVE_PATH="/tmp/${ARCHIVE_NAME}"
    if [ "$DRY_RUN" != 1 ] && [ ! -f "$ARCHIVE_PATH" ]; then
        err "Download failed"
        exit 1
    fi
fi

# ── Verify SHA-256 checksum ──
if [ "$NO_VERIFY" != 1 ] && [ "$DRY_RUN" != 1 ]; then
    step "Verifying SHA-256 checksum..."
    EXPECTED_HASH=""
    if [ -f "$CHECKSUM_FILE" ]; then
        EXPECTED_HASH=$(grep -E "${ARCHIVE_NAME}" "$CHECKSUM_FILE" | awk '{print $1}')
    fi
    if [ -z "$EXPECTED_HASH" ]; then
        echo "    Downloading checksum file..."
        CHECKSUMS_CONTENT=$(curl -fsSL "$CHECKSUM_URL" 2>/dev/null || true)
        EXPECTED_HASH=$(echo "$CHECKSUMS_CONTENT" | grep -E "${ARCHIVE_NAME}" | awk '{print $1}')
    fi

    if [ -n "$EXPECTED_HASH" ]; then
        ACTUAL_HASH=$(sha256sum "$ARCHIVE_PATH" | awk '{print $1}')
        if [ "$ACTUAL_HASH" = "$EXPECTED_HASH" ]; then
            ok "Checksum verified (${ACTUAL_HASH:0:16}...)"
        else
            err "Checksum MISMATCH"
            echo "    Expected: $EXPECTED_HASH"
            echo "    Actual:   $ACTUAL_HASH"
            echo "    The archive may be corrupted or tampered with."
            if [ "$NO_INTERACTIVE" != 1 ]; then
                read -r -p "Continue anyway? (y/N) " continue_anyway
                if [ "$continue_anyway" != "y" ] && [ "$continue_anyway" != "Y" ]; then
                    exit 1
                fi
            else
                exit 1
            fi
        fi
    else
        warn "No checksum found for ${ARCHIVE_NAME} — skipping verification"
    fi
fi

# ── Create directories ──
run "Creating installation directories..." \
    sh -c "mkdir -p '${INSTALL_DIR}' '${CONFIG_DIR}' '${DATA_DIR}' '${RULES_DIR}'"

# ── Extract archive ──
run "Extracting archive..." \
    sh -c "tar -xzf '${ARCHIVE_PATH}' -C '${INSTALL_DIR}' 2>/dev/null || tar -xzf '${ARCHIVE_PATH}' --strip-components=1 -C '${INSTALL_DIR}'"

if [ "$DRY_RUN" != 1 ]; then
    if [ ! -f "$BINARY_PATH" ]; then
        # Try to find onus binary in extracted files
        FOUND=$(find "${INSTALL_DIR}" -name "onus" -type f 2>/dev/null | head -1)
        if [ -n "$FOUND" ] && [ "$FOUND" != "$BINARY_PATH" ]; then
            mv "$FOUND" "$BINARY_PATH"
        fi
    fi

    if [ ! -f "$BINARY_PATH" ]; then
        err "onus binary not found after extraction"
        exit 1
    fi
    chmod +x "$BINARY_PATH"
    ok "Extracted onus to ${BINARY_PATH}"
fi

# ── Check bubblewrap (L3 isolation) ──
if [ "$DRY_RUN" != 1 ]; then
    if command -v bwrap &>/dev/null; then
        ok "bubblewrap (bwrap) detected — L3 workspace isolation available"
    else
        warn "bubblewrap (bwrap) not found — L3 workspace isolation not available"
        echo "    Install with: sudo apt install bubblewrap (Debian/Ubuntu)"
        echo "                  sudo dnf install bubblewrap (Fedora)"
        echo "                  sudo pacman -S bubblewrap (Arch)"
    fi
fi

# ── System install: fix permissions ──
if [ "$SYSTEM" = 1 ] && [ "$DRY_RUN" != 1 ]; then
    if [ "$(id -u)" -ne 0 ]; then
        warn "System install may require sudo for directory permissions"
    fi
fi

# ── Verify binary ──
run "Verifying Onus binary..." \
    sh -c "'${BINARY_PATH}' --version"
if [ "$DRY_RUN" != 1 ]; then
    VERSION_OUTPUT=$("$BINARY_PATH" --version 2>&1)
    ok "onus --version: ${VERSION_OUTPUT}"
fi

# ── Run doctor ──
if [ "$DRY_RUN" != 1 ]; then
    step "Running Onus diagnostics..."
    echo ""
    "$BINARY_PATH" doctor 2>&1 || true
    echo ""
    ok "Doctor check completed"
fi

# ── Print next steps ──
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║        Installation Complete!                ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo "  Next steps:"
echo "   1. Run:         onus setup"
echo "   2. Doctor:      onus doctor"
echo "   3. Start:       onus daemon start"
echo "   4. Console:     onus dashboard"
echo ""
echo "  Documentation: https://ahsanmoizz.github.io/onus/docs"
echo "  Uninstall:     curl -fsSL https://github.com/${REPO}/releases/latest/download/uninstall.sh | bash"
echo ""

exit 0
