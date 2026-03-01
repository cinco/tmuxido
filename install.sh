#!/bin/sh
set -e

REPO="cinco/tmuxido"
BASE_URL="https://github.com"
RAW_URL="https://raw.githubusercontent.com/$REPO/refs/heads/main"
API_URL="https://api.github.com"
INSTALL_DIR="$HOME/.local/bin"
ICON_DIR="$HOME/.local/share/icons/hicolor/96x96/apps"
DESKTOP_DIR="$HOME/.local/share/applications"

arch=$(uname -m)
case "$arch" in
    x86_64)        file="tmuxido-x86_64-linux" ;;
    aarch64|arm64) file="tmuxido-aarch64-linux" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

tag=$(curl -fsSL \
  -H "Accept: application/vnd.github.v3+json" \
  "$API_URL/repos/$REPO/releases/latest" \
  | grep -o '"tag_name":"[^"]*"' | cut -d'"' -f4)

[ -z "$tag" ] && { echo "Could not fetch latest release" >&2; exit 1; }

echo "Installing tmuxido $tag..."

# Binary
mkdir -p "$INSTALL_DIR"
curl -fsSL "$BASE_URL/$REPO/releases/download/$tag/$file" -o "$INSTALL_DIR/tmuxido"
chmod +x "$INSTALL_DIR/tmuxido"
echo "  binary  → $INSTALL_DIR/tmuxido"

# Icon (96×96)
mkdir -p "$ICON_DIR"
curl -fsSL "$RAW_URL/docs/assets/tmuxido-icon_96.png" -o "$ICON_DIR/tmuxido.png"
echo "  icon    → $ICON_DIR/tmuxido.png"

# .desktop entry
mkdir -p "$DESKTOP_DIR"
curl -fsSL "$RAW_URL/tmuxido.desktop" -o "$DESKTOP_DIR/tmuxido.desktop"
echo "  desktop → $DESKTOP_DIR/tmuxido.desktop"

# Refresh desktop database if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

# Refresh icon cache if available
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "Note: add $INSTALL_DIR to your PATH (e.g. export PATH=\"\$HOME/.local/bin:\$PATH\")" ;;
esac

echo "Done! Run 'tmuxido' to get started."
