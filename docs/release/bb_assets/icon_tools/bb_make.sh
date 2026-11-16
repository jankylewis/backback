#!/usr/bin/env bash
# Build backback's pixel icon from bb_mascot/bb_mascot.svg (BB-FEAT-080):
# no tile, transparent, 64 x 64 pixel grid. Needs macOS (qlmanage, swiftc,
# iconutil) and ImageMagick (`magick`).
#   bb_mascot/icon.png       1024 px master
#   bb_mascot/icon.icns      Finder, Launchpad (bb_bundle.sh ships it)
#   bb_mascot/previews/      16, 32, 128 px and the contact sheet
#   shell/desktop/bb_assets/dock.png  256 px, set as the Dock icon at launch
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
mascot="$here/../bb_mascot"
dock="$here/../../../../bb_crates/shell/desktop/bb_assets/dock.png"
grid=64
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# 1. Render the SVG with WebKit (ImageMagick's own SVG reader drops strokes).
qlmanage -t -s 1024 -o "$work" "$mascot/bb_mascot.svg" >/dev/null 2>&1
render="$work/bb_mascot.svg.png"
test -s "$render"

# 2. Clear the dark tile (flood fill from three points on it), harden the
#    edges, and clear everything outside the tile's rounded square.
magick "$render" -alpha set -fuzz 0.5% -fill none \
  -draw "color 40,512 floodfill" -draw "color 512,990 floodfill" \
  -draw "color 980,300 floodfill" -channel A -threshold 90% +channel \
  "$work/clear.png"
magick -size 1024x1024 xc:black -fill white \
  -draw "roundrectangle 14,14 1009,1009 206,206" "$work/mask.png"
magick "$work/clear.png" \
  \( "$work/clear.png" -alpha extract "$work/mask.png" -compose multiply -composite \) \
  -alpha off -compose copy_opacity -composite "$work/cutout.png"

# 3. Pixelate: crop to the figure, fit a 64 x 64 grid, hard alpha, 1024 px.
swiftc -O "$here/pixelate.swift" -o "$work/pixelate"
"$work/pixelate" "$work/cutout.png" "$mascot/icon.png" "$grid"

# 4. Every size by nearest neighbour, so the pixels stay sharp.
set_dir="$work/icon.iconset"
mkdir "$set_dir"
for size in 16 32 128 256 512; do
  magick "$mascot/icon.png" -filter point -resize "${size}x${size}" "$set_dir/icon_${size}x${size}.png"
  double=$((size * 2))
  magick "$mascot/icon.png" -filter point -resize "${double}x${double}" "$set_dir/icon_${size}x${size}@2x.png"
done
iconutil -c icns "$set_dir" -o "$mascot/icon.icns"
cp "$set_dir/icon_128x128.png" "$mascot/previews/128.png"
cp "$set_dir/icon_32x32.png" "$mascot/previews/32.png"
cp "$set_dir/icon_16x16.png" "$mascot/previews/bb_16.png"
cp "$set_dir/icon_256x256.png" "$dock"

# 5. The contact sheet: all 10 sizes on the dark theme background.
magick "$set_dir"/*.png -bordercolor '#191c27' -border 16 -background '#191c27' \
  -gravity center +append "$mascot/previews/CONTACT_SHEET.png"
echo "icon: $mascot/icon.png, $mascot/icon.icns, $dock"
