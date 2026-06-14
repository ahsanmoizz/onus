#!/usr/bin/env bash
# Onus Installer — one-command install for Linux, macOS, and Windows (Git Bash/MSYS2).
# Downloads the Onus binary, installs default rules, wires Claude Code hook.
#
# Usage:
#   curl -fsSL https://github.com/Gitlawb/onus/releases/latest/download/install.sh | bash
#   curl -fsSL https://github.com/Gitlawb/onus/releases/latest/download/install.sh | bash -s -- v0.1.0

set -euo pipefail

REPO="Gitlawb/onus"
VERSION="${1:-latest}"

# ── Detect platform ──
OS=""
ARCH=""
EXT=""

case "$(uname -s)" in
  Linux)   OS="linux" ;;
  Darwin)  OS="macos" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

if [ "$OS" = "windows" ]; then
  EXT=".exe"
fi

# ── Directories ──
if [ "$OS" = "windows" ]; then
  INSTALL_DIR="${LOCALAPPDATA:-$HOME/AppData/Local}/onus"
  CONFIG_DIR="${APPDATA:-$HOME/AppData/Roaming}/onus"
  DATA_DIR="${INSTALL_DIR}/data"
else
  INSTALL_DIR="${HOME}/.local/bin"
  CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/onus"
  DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/onus"
fi

BINARY_NAME="onus${EXT}"
INSTALL_PATH="${INSTALL_DIR}/${BINARY_NAME}"

# ── Download URL ──
if [ "$VERSION" = "latest" ]; then
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/onus-${OS}-${ARCH}${EXT}"
  VERSION_TAG="latest"
else
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/onus-${OS}-${ARCH}${EXT}"
  VERSION_TAG="${VERSION}"
fi

# ── Print banner ──
cat <<EOF
╔══════════════════════════════════════════════╗
║        Onus — AI Agent Firewall             ║
╚══════════════════════════════════════════════╝

  Platform:  ${OS}/${ARCH}
  Version:   ${VERSION_TAG}
  Install:   ${INSTALL_PATH}
  Config:    ${CONFIG_DIR}
  Data:      ${DATA_DIR}

EOF

# ── Create directories ──
mkdir -p "${INSTALL_DIR}"
mkdir -p "${CONFIG_DIR}/rules"
mkdir -p "${DATA_DIR}"

# ── Download binary ──
echo "Downloading..."
if command -v curl > /dev/null 2>&1; then
  curl -fsSL -o "${INSTALL_PATH}.tmp" "${DOWNLOAD_URL}"
elif command -v wget > /dev/null 2>&1; then
  wget -qO "${INSTALL_PATH}.tmp" "${DOWNLOAD_URL}"
else
  echo "Error: need curl or wget"
  exit 1
fi

chmod +x "${INSTALL_PATH}.tmp"
mv "${INSTALL_PATH}.tmp" "${INSTALL_PATH}"
echo "  ✓ Binary installed"

# ── Install default rules ──
echo "  ✓ Installing default safety rules..."
"${INSTALL_PATH}" rules init 2>/dev/null || {
  # Fallback: copy rules directly
  RULES_SRC="$(dirname "$0")/../rules/default.toml"
  if [ -f "$RULES_SRC" ]; then
    cp "$RULES_SRC" "${CONFIG_DIR}/rules/default.toml"
    echo "  ✓ Default rules copied"
  fi
}

# ── Wire Claude Code hook ──
echo "  ✓ Configuring Claude Code..."
CLAUDE_CONFIG="${HOME}/.claude/settings.json"
if [ -f "$CLAUDE_CONFIG" ]; then
  if grep -q "onus" "$CLAUDE_CONFIG" 2>/dev/null; then
    echo "  ✓ Claude Code hook already configured"
  else
    # Use simple sed-based JSON merge to avoid Python dependency
    TMP_CONFIG=$(mktemp)
    python3 -c "
import json, sys
with open('${CLAUDE_CONFIG}') as f:
    c = json.load(f)
c.setdefault('hooks', {})
c['hooks']['preToolUse'] = {
    'command': '/usr/local/bin/onus evaluate',
    'timeout': 5000
}
with open('${CLAUDE_CONFIG}', 'w') as f:
    json.dump(c, f, indent=2)
" 2>/dev/null && echo "  ✓ Claude Code hook wired" || echo "  ⚠ Could not auto-configure Claude Code (install python3)"
  fi
else
  echo "  ○ Claude Code not detected — skip hook setup"
fi

# ── Add to PATH ──
if [ "$OS" != "windows" ]; then
  PATH_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""
  for RC in "${HOME}/.bashrc" "${HOME}/.zshrc"; do
    if [ -f "$RC" ] && ! grep -qF "${INSTALL_DIR}" "$RC" 2>/dev/null; then
      echo "" >> "$RC"
      echo "# Onus" >> "$RC"
      echo "${PATH_LINE}" >> "$RC"
      echo "  ✓ Added to ${RC}"
    fi
  done
fi

# ── Verify ──
echo ""
if "${INSTALL_PATH}" --version 2>/dev/null; then
  echo ""
  echo "═══════════════════════════════════════════"
  echo "  Onus installed successfully!"
  echo ""
  echo "  Quick start:"
  echo "    onus shell install       # Protect terminal agents"
  echo "    onus mcp-proxy --help    # Protect MCP-based agents"
  echo "    onus rules list          # See all safety rules"
  echo "    onus --help              # Full command list"
  echo ""
  echo "  Restart your shell or:"
  echo "    source ~/.bashrc"
  echo "═══════════════════════════════════════════"
else
  echo "⚠ Binary may need PATH update"
fi
