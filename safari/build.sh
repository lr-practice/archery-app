#!/usr/bin/env bash
# Baut die Safari-Web-Extension als App-Erweiterung für Archery.app
# (nur auf dem Mac mit Xcode). Ergebnis:
#   build/safari/Archery Extension.appex
# Der Tauri-Build legt sie nach Contents/PlugIns (bundle.macOS.files, siehe
# .github/workflows/app-macos.yml).
#
# Ablauf:
#   1. Kopie von extension/ (mit Melange, falls vorhanden) und daraus ein
#      Safari-Manifest: ohne key, minimum_chrome_version, downloads, pageCapture.
#   2. safari-web-extension-converter erzeugt das Xcode-Projekt.
#   3. Unser SafariWebExtensionHandler.swift ersetzt die Vorlage.
#   4. Nur das Erweiterungs-Target bauen, unsigniert.
#   5. Signieren mit Sandbox-Entitlements: ad-hoc oder mit
#      $APPLE_SIGNING_IDENTITY (Developer ID, dann mit Hardened Runtime).
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
SAFARI="$ROOT/safari"
OUT="${OUT:-$ROOT/build/safari}"
APP_NAME="Archery"
EXT_NAME="Archery Extension"

for tool in xcrun xcodebuild codesign node plutil; do
  command -v "$tool" >/dev/null || { echo "safari/build.sh: $tool fehlt (nur auf dem Mac mit Xcode)" >&2; exit 1; }
done

APP_ID=$(node -p "require(process.argv[1]).identifier" "$ROOT/app/src-tauri/tauri.conf.json")
EXT_ID="$APP_ID.Extension"
[ -n "$APP_ID" ] || { echo "safari/build.sh: identifier fehlt in tauri.conf.json" >&2; exit 1; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/archiver-safari.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

# 1. Extension kopieren (ohne versteckte Dateien) und Manifest anpassen
mkdir -p "$WORK/ext"
cp -R "$ROOT/extension/." "$WORK/ext/"
find "$WORK/ext" -mindepth 1 -name '.*' -prune -exec rm -rf {} +
if ls "$WORK/ext/fonts/melange/"*.woff2 >/dev/null 2>&1; then
  echo "Melange: dabei"
else
  echo "Melange: fehlt (Extension fällt auf Geist zurück)"
fi
node - "$ROOT/extension/manifest.json" "$WORK/ext/manifest.json" <<'NODE'
const fs = require("fs");
const [src, dst] = process.argv.slice(2);
const m = JSON.parse(fs.readFileSync(src, "utf8"));
delete m.key;
delete m.minimum_chrome_version;
m.permissions = (m.permissions || []).filter((p) => !["downloads", "pageCapture"].includes(p));
// Safari vergibt ohnehin pro Installation eigene Adressen
for (const r of m.web_accessible_resources || []) delete r.use_dynamic_url;
// OffscreenCanvas im Hintergrund, MV3-Service-Worker
m.browser_specific_settings = { safari: { strict_min_version: "16.4" } };
fs.writeFileSync(dst, JSON.stringify(m, null, 2) + "\n");
console.log(`Safari-Manifest ${m.version}: ${m.permissions.join(", ")}`);
NODE

# 2. Xcode-Projekt erzeugen
xcrun safari-web-extension-converter "$WORK/ext" \
  --project-location "$WORK/proj" \
  --app-name "$APP_NAME" \
  --bundle-identifier "$APP_ID" \
  --swift --macos-only --copy-resources \
  --no-open --no-prompt --force

PROJECT=$(find "$WORK/proj" -maxdepth 3 -name '*.xcodeproj' | head -1)
[ -n "$PROJECT" ] || { echo "safari/build.sh: kein Xcode-Projekt erzeugt" >&2; exit 1; }

# 3. Eigener Handler statt der Vorlage
HANDLER=$(find "$WORK/proj" -name 'SafariWebExtensionHandler.swift' -not -path '*/Resources/*' | head -1)
[ -n "$HANDLER" ] || { echo "safari/build.sh: SafariWebExtensionHandler.swift nicht gefunden" >&2; exit 1; }
cp "$SAFARI/SafariWebExtensionHandler.swift" "$HANDLER"

# 4. Nur die Erweiterung bauen
TARGET=$(xcodebuild -list -json -project "$PROJECT" | node -e '
  let s = ""; process.stdin.on("data", (d) => (s += d)).on("end", () => {
    const t = JSON.parse(s).project.targets.find((n) => / Extension$/.test(n));
    if (!t) process.exit(1);
    console.log(t);
  });')
echo "Target: $TARGET"
xcodebuild -project "$PROJECT" -target "$TARGET" -configuration Release \
  SYMROOT="$WORK/build" OBJROOT="$WORK/obj" \
  PRODUCT_BUNDLE_IDENTIFIER="$EXT_ID" \
  MACOSX_DEPLOYMENT_TARGET=12.0 \
  CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY="" \
  build

APPEX=$(find "$WORK/build" -maxdepth 3 -name '*.appex' -type d | head -1)
[ -n "$APPEX" ] || { echo "safari/build.sh: keine .appex gebaut" >&2; exit 1; }

# Kennung der Erweiterung muss mit der Kennung der App beginnen
GOT=$(plutil -extract CFBundleIdentifier raw "$APPEX/Contents/Info.plist")
case "$GOT" in
  "$APP_ID".*) ;;
  *) echo "safari/build.sh: Kennung $GOT passt nicht zu $APP_ID" >&2; exit 1 ;;
esac

# 5. Signieren (eine App-Erweiterung muss in der Sandbox laufen)
IDENTITY="${APPLE_SIGNING_IDENTITY:--}"
if [ "$IDENTITY" = "-" ]; then
  codesign --force --sign - --entitlements "$SAFARI/Extension.entitlements" "$APPEX"
else
  codesign --force --sign "$IDENTITY" --options runtime --timestamp --entitlements "$SAFARI/Extension.entitlements" "$APPEX"
fi
codesign --verify --strict "$APPEX"
codesign -d --entitlements - "$APPEX" 2>/dev/null | grep -q "com.apple.security.app-sandbox" \
  || { echo "safari/build.sh: Sandbox-Entitlement fehlt" >&2; exit 1; }

mkdir -p "$OUT"
rm -rf "$OUT/$EXT_NAME.appex"
ditto "$APPEX" "$OUT/$EXT_NAME.appex"
echo "Fertig: $OUT/$EXT_NAME.appex ($GOT)"
