#!/bin/bash
set -euo pipefail

APP_NAME="MarkZap"
BINARY_NAME="markzap"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Parse arguments
BUILD_MODE="release"
if [[ "${1:-}" == "--debug" ]]; then
    BUILD_MODE="debug"
fi

echo "Building $APP_NAME ($BUILD_MODE)..."

# Build the binary
if [[ "$BUILD_MODE" == "release" ]]; then
    cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"
    BINARY_PATH="$PROJECT_DIR/target/release/$BINARY_NAME"
else
    cargo build --manifest-path "$PROJECT_DIR/Cargo.toml"
    BINARY_PATH="$PROJECT_DIR/target/debug/$BINARY_NAME"
fi

# Create the .app bundle structure
BUNDLE_DIR="$PROJECT_DIR/target/$BUILD_MODE/$APP_NAME.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "Creating app bundle at $BUNDLE_DIR..."

rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy the binary
cp "$BINARY_PATH" "$MACOS_DIR/$BINARY_NAME"

# Copy Info.plist
cp "$PROJECT_DIR/resources/Info.plist" "$CONTENTS_DIR/Info.plist"

# Copy icon if it exists
if [[ -f "$PROJECT_DIR/resources/AppIcon.icns" ]]; then
    cp "$PROJECT_DIR/resources/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
fi

# Create PkgInfo
echo -n "APPL????" > "$CONTENTS_DIR/PkgInfo"

# Code signing
ENTITLEMENTS="$PROJECT_DIR/resources/MarkZap.entitlements"
SIGNING_IDENTITY="${CODESIGN_IDENTITY:--}"

echo "Signing bundle with identity: $SIGNING_IDENTITY"

CODESIGN_ARGS=(--sign "$SIGNING_IDENTITY" --force --options runtime --deep)

if [[ -f "$ENTITLEMENTS" ]]; then
    CODESIGN_ARGS+=(--entitlements "$ENTITLEMENTS")
fi

codesign "${CODESIGN_ARGS[@]}" "$BUNDLE_DIR"

echo "Bundle signed successfully."

echo ""
echo "Bundle created: $BUNDLE_DIR"
echo ""
echo "To install, run:"
echo "  cp -r \"$BUNDLE_DIR\" /Applications/"
echo ""
echo "To register file associations, run:"
echo "  /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f /Applications/$APP_NAME.app"
