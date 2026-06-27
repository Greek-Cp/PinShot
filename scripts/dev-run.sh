#!/usr/bin/env bash
# Dev build + run for macOS that PRESERVES the Screen Recording permission across
# rebuilds. The trick: sign the bundle with a stable self-signed code-signing
# identity ("PinShot Dev") instead of an ad-hoc signature. Ad-hoc signatures get
# a new cdhash every build, which invalidates the TCC grant and forces a re-grant;
# a fixed certificate keeps the TCC designated requirement stable, so you grant
# Screen Recording exactly once.
#
# One-time setup (creates + imports the "PinShot Dev" identity):
#   scripts/dev-run.sh --setup-cert
#
# Normal use (build UI + app bundle, sign, relaunch):
#   scripts/dev-run.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/target/debug/bundle/macos/PinShot.app"
IDENTITY="PinShot Dev"

setup_cert() {
  local tmp; tmp="$(mktemp -d)"
  openssl req -x509 -newkey rsa:2048 -keyout "$tmp/k.key" -out "$tmp/c.crt" -days 3650 -nodes \
    -subj "/CN=$IDENTITY" \
    -addext "basicConstraints=critical,CA:FALSE" \
    -addext "keyUsage=critical,digitalSignature" \
    -addext "extendedKeyUsage=critical,codeSigning"
  # -legacy so macOS `security` can read the PKCS#12 (OpenSSL 3 default is unreadable).
  openssl pkcs12 -export -legacy -inkey "$tmp/k.key" -in "$tmp/c.crt" -out "$tmp/i.p12" \
    -passout pass:pinshot -name "$IDENTITY"
  security import "$tmp/i.p12" -k ~/Library/Keychains/login.keychain-db -P pinshot \
    -T /usr/bin/codesign -A
  rm -rf "$tmp"
  echo "Imported code-signing identity: $IDENTITY"
}

if [[ "${1:-}" == "--setup-cert" ]]; then
  setup_cert
  exit 0
fi

echo "==> Building UI"
( cd "$ROOT/ui" && npm run build )

echo "==> Building debug app bundle"
( cd "$ROOT" && cargo tauri build --debug --bundles app )

echo "==> Signing with stable identity ($IDENTITY)"
codesign --force --deep --sign "$IDENTITY" "$APP"

echo "==> Relaunching"
pkill -f "PinShot.app/Contents/MacOS" 2>/dev/null || true
sleep 1
open "$APP"
echo "Done. PinShot relaunched (Screen Recording grant is preserved)."
