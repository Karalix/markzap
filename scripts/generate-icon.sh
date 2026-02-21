#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ICONSET_DIR="$PROJECT_DIR/resources/AppIcon.iconset"
ICNS_FILE="$PROJECT_DIR/resources/AppIcon.icns"

SOURCE_IMAGE="${1:-}"

if [[ -z "$SOURCE_IMAGE" ]]; then
    echo "Usage: $0 <source-image-1024x1024.png>"
    echo ""
    echo "Provide a 1024x1024 PNG image to generate AppIcon.icns"
    exit 1
fi

if [[ ! -f "$SOURCE_IMAGE" ]]; then
    echo "Error: File not found: $SOURCE_IMAGE"
    exit 1
fi

rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

# Generate all required sizes using sips (built into macOS)
declare -a SIZES=(
    "16:icon_16x16.png"
    "32:icon_16x16@2x.png"
    "32:icon_32x32.png"
    "64:icon_32x32@2x.png"
    "128:icon_128x128.png"
    "256:icon_128x128@2x.png"
    "256:icon_256x256.png"
    "512:icon_256x256@2x.png"
    "512:icon_512x512.png"
    "1024:icon_512x512@2x.png"
)

for entry in "${SIZES[@]}"; do
    SIZE="${entry%%:*}"
    FILENAME="${entry##*:}"
    sips -z "$SIZE" "$SIZE" "$SOURCE_IMAGE" --out "$ICONSET_DIR/$FILENAME" > /dev/null 2>&1
done

# Convert .iconset to .icns
iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"
rm -rf "$ICONSET_DIR"

echo "Generated: $ICNS_FILE"
