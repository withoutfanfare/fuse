#!/bin/bash
# Build Fuse for production and show the output folder
set -e

cd "$(dirname "$0")"

echo "Building Fuse..."
npm run tauri build

BUNDLE_DIR="src-tauri/target/release/bundle"

echo ""
echo "Build complete:"
echo ""

if [ -d "$BUNDLE_DIR/macos" ]; then
  ls -lh "$BUNDLE_DIR/macos/"
fi

if [ -d "$BUNDLE_DIR/dmg" ]; then
  echo ""
  ls -lh "$BUNDLE_DIR/dmg/"*.dmg 2>/dev/null
fi

echo ""
echo "Bundle location: $(cd "$BUNDLE_DIR" && pwd)"

# Open the bundle folder in Finder
open "$BUNDLE_DIR/macos"
