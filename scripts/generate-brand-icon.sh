#!/usr/bin/env bash
#
# generate-brand-icon.sh — Build the Forge brand icon master + favicon set.
#
# Source of truth is assets/forge.png (the black crossed-hammers + anvil emblem
# on a transparent background). That art is non-square (392x310), too small for
# bundling, and invisible on dark docks. This script composites it onto an
# ember-gradient rounded-rect "app icon template" plate to produce a single
# 1024x1024 square master, then derives the web favicon variants from it.
#
# The 1024 master is the only input the rest of Forge's pipeline needs: forge's
# IconProcessor (crates/forge_cli/src/bundler/icons.rs) fans it out to .icns,
# MSIX tiles, and Linux hicolor sizes by resizing.
#
# Requires: ImageMagick (`magick`), and on macOS `iconutil` for the .icns.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/forge.png"
OUT="$ROOT/assets"

SIZE=1024          # master canvas (square)
MARGIN=44          # transparent inset around the plate (macOS-template style)
EMBLEM_W=430       # emblem width painted on the canvas (raised to make room for the wordmark)
EMBLEM_Y=232       # emblem top offset from the plate top (gravity north)

# "Forge" wordmark. InterDisplay is the optical-size cut meant for large/display
# use (logos, headlines); the plain Inter cut is for body text and reads loose here.
FONT="$ROOT/assets/fonts/InterDisplay-SemiBold.otf"
WORDMARK="Forge"
WORD_PT=176        # wordmark point size
WORD_Y=596         # wordmark top offset from the plate top (gravity north)
WORD_FILL="#000000"  # black wordmark fill
WORD_STROKE="#ffffff" # white keyline around the wordmark
WORD_STROKE_W=2      # stroke thickness in px

PLATE_SIZE=$((SIZE - 2 * MARGIN))
PLATE_X1=$((SIZE - MARGIN))
PLATE_Y1=$((SIZE - MARGIN))
# Apple's continuous-corner ratio (~0.2237 of the side length).
RADIUS=$(awk "BEGIN{printf \"%d\", 0.2237 * $PLATE_SIZE}")

[ -f "$SRC" ] || { echo "error: source emblem not found: $SRC" >&2; exit 1; }
command -v magick >/dev/null || { echo "error: ImageMagick 'magick' not found" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> Building ember plate (${SIZE}x${SIZE}, inset ${MARGIN}px, radius ${RADIUS}px)"

# 1. Ember linear gradient: bright forge-orange at top -> deep red at the base.
magick -size ${SIZE}x${SIZE} gradient:'#ffb15a-#5e0d05' "$TMP/base.png"

# 2. Soft warm radial highlight, composited over the gradient. This lifts the
#    black emblem off the dark lower half of the gradient so the anvil reads.
magick -size ${SIZE}x${SIZE} radial-gradient:'#fff0d8a0-#fff0d800' "$TMP/glow.png"
magick "$TMP/base.png" "$TMP/glow.png" -compose over -composite "$TMP/bg.png"

# 3. Rounded-rect "app icon" mask, applied to the gradient via CopyOpacity so
#    the corners (and inset margin) become transparent.
magick -size ${SIZE}x${SIZE} xc:none -fill white \
  -draw "roundrectangle ${MARGIN},${MARGIN},${PLATE_X1},${PLATE_Y1} ${RADIUS},${RADIUS}" \
  "$TMP/mask.png"
magick "$TMP/bg.png" "$TMP/mask.png" \
  -alpha off -compose CopyOpacity -composite "$TMP/plate.png"

[ -f "$FONT" ] || { echo "error: Inter font not found: $FONT" >&2; exit 1; }

# 4. A white silhouette of the emblem, blurred into a soft halo. Sits behind the
#    black emblem to guarantee contrast on the darkest part of the plate.
magick "$SRC" -resize ${EMBLEM_W}x \
  -channel RGB -evaluate set 100% +channel \
  "$TMP/white.png"

# 5. Compose the lockup: plate <- halo <- raised emblem <- "Forge" wordmark.
#    The emblem sits in the upper portion; the wordmark sits below it.
magick "$TMP/plate.png" \
  \( "$TMP/white.png" -blur 0x10 -channel A -evaluate multiply 0.5 +channel \) \
      -gravity north -geometry +0+${EMBLEM_Y} -composite \
  \( "$SRC" -resize ${EMBLEM_W}x \) \
      -gravity north -geometry +0+${EMBLEM_Y} -composite \
  -font "$FONT" -pointsize ${WORD_PT} -gravity north \
  -fill "${WORD_STROKE}" -stroke "${WORD_STROKE}" -strokewidth $((WORD_STROKE_W * 2)) \
      -annotate +0+${WORD_Y} "${WORDMARK}" \
  -fill "${WORD_FILL}" -stroke none \
      -annotate +0+${WORD_Y} "${WORDMARK}" \
  "$OUT/forge-icon-1024.png"

echo "==> Wrote $OUT/forge-icon-1024.png (emblem + \"${WORDMARK}\" wordmark, Inter Display SemiBold)"

# ---------------------------------------------------------------------------
# Brand MARK: the emblem on the same ember plate, WITHOUT the wordmark, with the
# emblem centered and a touch larger. Used for the docs navbar logo, where the
# adjacent "Forge" title text already supplies the word. Reuses the same plate
# so the mark and the full icon share background, corners, and glow exactly.
# ---------------------------------------------------------------------------
MARK_EMBLEM_W=600
magick "$SRC" -resize ${MARK_EMBLEM_W}x -channel RGB -evaluate set 100% +channel "$TMP/markwhite.png"
magick "$TMP/plate.png" \
  \( "$TMP/markwhite.png" -blur 0x10 -channel A -evaluate multiply 0.5 +channel \) \
      -gravity center -composite \
  \( "$SRC" -resize ${MARK_EMBLEM_W}x \) \
      -gravity center -composite \
  "$OUT/forge-mark-1024.png"

# logo.svg: vector wrapper around a 128px raster of the mark (matches the favicon
# approach). The navbar renders it at ~32px (64px hi-dpi), so 128px stays crisp
# while keeping the file lightweight.
MARK_B64="$(magick "$OUT/forge-mark-1024.png" -resize 128x128 png:- | base64 | tr -d '\n')"
cat > "$OUT/logo.svg" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128">
  <image width="128" height="128" href="data:image/png;base64,${MARK_B64}"/>
</svg>
SVG
echo "==> Wrote $OUT/forge-mark-1024.png + $OUT/logo.svg (emblem-only mark)"

# ---------------------------------------------------------------------------
# Favicon set, derived from the same master so the brand matches pixel-for-pixel.
# ---------------------------------------------------------------------------
MASTER="$OUT/forge-icon-1024.png"

echo "==> Building favicon set"
magick "$MASTER" -resize 16x16   "$OUT/favicon-16.png"
magick "$MASTER" -resize 32x32   "$OUT/favicon-32.png"
magick "$MASTER" -resize 48x48   "$OUT/favicon-48.png"
magick "$MASTER" -resize 180x180 "$OUT/apple-touch-icon.png"
# Multi-resolution .ico for legacy browsers.
magick "$OUT/favicon-16.png" "$OUT/favicon-32.png" "$OUT/favicon-48.png" "$OUT/favicon.ico"

# favicon.svg: a vector wrapper around a small (64px) raster of the master. Tab
# favicons render at ~16-32px, so 64px keeps the file lightweight while matching
# the brand exactly. The emblem art is raster (assets/forge.png embeds no clean
# vector), so a baked raster is the faithful choice over re-vectorizing. Larger
# raster needs (bookmarks, home-screen) are served by favicon.ico / apple-touch.
B64="$(magick "$MASTER" -resize 64x64 png:- | base64 | tr -d '\n')"
cat > "$OUT/favicon.svg" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
  <image width="64" height="64" href="data:image/png;base64,${B64}"/>
</svg>
SVG
echo "    favicon.svg ($(wc -c < "$OUT/favicon.svg" | tr -d ' ') bytes), favicon.ico, favicon-{16,32,48}.png, apple-touch-icon.png"

# ---------------------------------------------------------------------------
# macOS .icns (optional; needs iconutil). Brand-level artifact in assets/.
# ---------------------------------------------------------------------------
if command -v iconutil >/dev/null; then
  echo "==> Building forge.icns (macOS)"
  ICONSET="$TMP/forge.iconset"
  mkdir -p "$ICONSET"
  for s in 16 32 128 256 512; do
    magick "$MASTER" -resize ${s}x${s}       "$ICONSET/icon_${s}x${s}.png"
    magick "$MASTER" -resize $((s*2))x$((s*2)) "$ICONSET/icon_${s}x${s}@2x.png"
  done
  iconutil -c icns -o "$OUT/forge.icns" "$ICONSET"
  echo "    $OUT/forge.icns"
fi

echo "==> Done. Master: $MASTER"
