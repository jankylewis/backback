#!/bin/bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
build_dir="${1:-$repo_dir/.target/bundle}"
app_dir="$build_dir/backback.app"
asset_dir="$repo_dir/docs/release/bb_assets/bb_mascot"
version="$(cd "$repo_dir/bb_crates" && cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); print(next(p["version"] for p in data["packages"] if p["name"] == "bb"))')"

mkdir -p "$build_dir"
cd "$repo_dir/bb_crates"
cargo build --release -p backback -p bb
rm -rf "${app_dir:?}"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
cp "$repo_dir/.target/release/backback" "$app_dir/Contents/MacOS/backback"
cp "$repo_dir/.target/release/bb" "$app_dir/Contents/Resources/bb"
cmp "$repo_dir/.target/release/backback" "$app_dir/Contents/MacOS/backback"
cp "$asset_dir/icon.icns" "$app_dir/Contents/Resources/backback.icns"
cat > "$app_dir/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>io.github.jankylewis.backback</string>
<key>CFBundleName</key><string>backback</string>
<key>CFBundleDisplayName</key><string>backback</string>
<key>CFBundleExecutable</key><string>backback</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleVersion</key><string>$version</string>
<key>CFBundleIconFile</key><string>backback</string>
<key>NSRequiresAquaSystemAppearance</key><false/>
<key>LSMinimumSystemVersion</key><string>13.0</string>
</dict></plist>
EOF
codesign --force --deep --sign - "$app_dir"
codesign --verify --deep --strict "$app_dir"
plutil -lint "$app_dir/Contents/Info.plist"
test -x "$app_dir/Contents/MacOS/backback"
test -x "$app_dir/Contents/Resources/bb"
test -s "$app_dir/Contents/Resources/backback.icns"
echo "$app_dir"
