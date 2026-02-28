#!/bin/sh
set -e

REPO="cinco/Tmuxido"
BASE_URL="https://git.cincoeuzebio.com"
INSTALL_DIR="$HOME/.local/bin"

arch=$(uname -m)
case "$arch" in
    x86_64)        file="tmuxido-x86_64-linux" ;;
    aarch64|arm64) file="tmuxido-aarch64-linux" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

tag=$(curl -fsSL "$BASE_URL/api/v1/repos/$REPO/releases?limit=1&page=1" \
  | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)

[ -z "$tag" ] && { echo "Could not fetch latest release" >&2; exit 1; }

echo "Installing tmuxido $tag..."
mkdir -p "$INSTALL_DIR"
curl -fsSL "$BASE_URL/$REPO/releases/download/$tag/$file" -o "$INSTALL_DIR/tmuxido"
chmod +x "$INSTALL_DIR/tmuxido"
echo "Installed: $INSTALL_DIR/tmuxido"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "Note: add $INSTALL_DIR to your PATH (e.g. export PATH=\"\$HOME/.local/bin:\$PATH\")" ;;
esac
