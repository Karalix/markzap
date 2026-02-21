#!/bin/bash
set -euo pipefail

APP_NAME="MarkZap"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Parse arguments
BUILD_MODE="release"
if [[ "${1:-}" == "--debug" ]]; then
    BUILD_MODE="debug"
fi

BUNDLE_DIR="$PROJECT_DIR/target/$BUILD_MODE/$APP_NAME.app"
DMG_DIR="$PROJECT_DIR/target/$BUILD_MODE"
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
DMG_PATH="$DMG_DIR/$DMG_NAME"

# Verify bundle exists
if [[ ! -d "$BUNDLE_DIR" ]]; then
    echo "Error: Bundle not found at $BUNDLE_DIR"
    echo "Run 'make bundle' first."
    exit 1
fi

# Check for create-dmg
if ! command -v create-dmg &> /dev/null; then
    echo "Error: create-dmg not found."
    echo "Install it with: brew install create-dmg"
    exit 1
fi

# Remove old DMG if it exists (create-dmg won't overwrite)
rm -f "$DMG_PATH"

echo "Creating DMG: $DMG_NAME..."

# Build create-dmg command
CMD=(create-dmg
    --volname "$APP_NAME"
    --window-pos 200 120
    --window-size 600 400
    --icon-size 100
    --icon "$APP_NAME.app" 150 190
    --app-drop-link 450 190
    --no-internet-enable
)

# Add background image if it exists
if [[ -f "$PROJECT_DIR/resources/dmg-background.png" ]]; then
    CMD+=(--background "$PROJECT_DIR/resources/dmg-background.png")
fi

# Add volume icon if .icns exists
if [[ -f "$PROJECT_DIR/resources/AppIcon.icns" ]]; then
    CMD+=(--volicon "$PROJECT_DIR/resources/AppIcon.icns")
fi

CMD+=("$DMG_PATH" "$BUNDLE_DIR")

"${CMD[@]}"

echo ""
echo "DMG created: $DMG_PATH"
echo "Size: $(du -h "$DMG_PATH" | cut -f1)"
