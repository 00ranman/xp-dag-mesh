#!/usr/bin/env bash
set -euo pipefail

echo "========================================"
echo "  XP DAG Mesh - Desktop Client Installer"
echo "========================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

check_cmd() {
  if ! command -v "$1" &> /dev/null; then
    echo -e "${RED}Error: $1 is not installed.${NC}"
    echo "  Install it from: $2"
    exit 1
  fi
  echo -e "  ${GREEN}[OK]${NC} $1 found"
}

echo "Checking prerequisites..."
echo ""
check_cmd "node" "https://nodejs.org"
check_cmd "npm" "https://nodejs.org"
check_cmd "rustc" "https://rustup.rs"
check_cmd "cargo" "https://rustup.rs"

# Check Rust version >= 1.70
RUST_VER=$(rustc --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
echo -e "  ${GREEN}[OK]${NC} Rust version: $RUST_VER"

echo ""
echo "Installing system dependencies..."

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  echo "Detected: Linux"
  if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
      libwebkit2gtk-4.1-dev \
      build-essential \
      curl \
      wget \
      file \
      libxdo-dev \
      libssl-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev
  elif command -v dnf &> /dev/null; then
    sudo dnf install -y \
      webkit2gtk4.1-devel \
      openssl-devel \
      curl wget file \
      libxdo-devel \
      libappindicator-gtk3-devel \
      librsvg2-devel
  fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Detected: macOS"
  if ! command -v brew &> /dev/null; then
    echo -e "${YELLOW}Homebrew not found. Installing...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  fi
else
  echo -e "${YELLOW}Warning: Unsupported OS. You may need to install dependencies manually.${NC}"
fi

echo ""
echo "Setting up the desktop client..."
cd desktop

echo "Installing Node.js dependencies..."
npm install

echo ""
echo "Building the desktop application..."
npm run tauri:build

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Build complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "The installer package is in:"
echo "  desktop/src-tauri/target/release/bundle/"
echo ""
echo "To run in development mode:"
echo "  cd desktop && npm run tauri:dev"
