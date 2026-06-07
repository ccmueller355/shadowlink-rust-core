#!/bin/sh
# ─── Install gitleaks ──────────────────────────────────────────────
# Downloads the gitleaks binary for the current platform.
# Requires: curl, tar
# ────────────────────────────────────────────────────────────────────
set -e

VERSION="8.21.2"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)  ARCH="x64" ;;
    aarch64) ARCH="arm64" ;;
    arm64)   ARCH="arm64" ;;
    *)       echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

URL="https://github.com/gitleaks/gitleaks/releases/download/v${VERSION}/gitleaks_${VERSION}_${OS}_${ARCH}.tar.gz"

echo "  📦 Installing gitleaks v${VERSION} for ${OS}/${ARCH}..."
curl -sL "$URL" -o /tmp/gitleaks.tar.gz
tar -xzf /tmp/gitleaks.tar.gz -C /usr/local/bin/ gitleaks
rm /tmp/gitleaks.tar.gz

echo "  ✓ gitleaks $(gitleaks version) installed."
