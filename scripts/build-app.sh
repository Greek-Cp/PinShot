#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

CONFIG="${1:-release}"
echo "🔨 Building PinShot ($CONFIG)..."
swift build -c "$CONFIG"

BIN_PATH="$ROOT_DIR/.build/$CONFIG/PinShot"
APP_DIR="$ROOT_DIR/PinShot.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "📦 Creating macOS App Bundle at $APP_DIR..."
# Kill existing running process if any
pkill -f "PinShot.app/Contents/MacOS/PinShot" || true

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

# Copy binary
cp "$BIN_PATH" "$MACOS_DIR/PinShot"
chmod +x "$MACOS_DIR/PinShot"

# Copy Info.plist
cp "$ROOT_DIR/Resources/Info.plist" "$CONTENTS_DIR/Info.plist"

# Create PkgInfo
echo -n "APPLPSHT" > "$CONTENTS_DIR/PkgInfo"

# Code-sign with ad-hoc signature and entitlements
echo "✍️  Signing PinShot.app (Bundle ID: com.fable.PinShot)..."
codesign --force --deep --sign - --entitlements "$ROOT_DIR/Resources/Entitlements.plist" --identifier "com.fable.PinShot" "$APP_DIR"

echo "✅ PinShot.app built successfully!"
echo "👉 Run with: open $APP_DIR"
