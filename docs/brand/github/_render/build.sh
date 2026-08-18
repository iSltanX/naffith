#!/usr/bin/env bash
# =============================================================================
# نَفِّذ · rebuild the GitHub/README artwork
# -----------------------------------------------------------------------------
# Renders the three artboards in this folder to PNG with headless Chromium and
# drops them next to this directory, where the README references them.
#
#   ./docs/brand/github/_render/build.sh
#
# No network is used: the fonts come from src/design-system/fonts/ and the mark
# is inline SVG, so the output is byte-stable for a given Chromium build.
# =============================================================================
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="$(cd "$here/.." && pwd)"

# Prefer the Playwright-managed Chromium this environment ships with, then fall
# back to whatever chromium/chrome is on PATH.
chrome=""
for c in /opt/pw-browsers/chromium-*/chrome-linux/chrome \
         "$(command -v chromium || true)" \
         "$(command -v chromium-browser || true)" \
         "$(command -v google-chrome || true)"; do
  [ -n "$c" ] && [ -x "$c" ] && { chrome="$c"; break; }
done
[ -n "$chrome" ] || { echo "no chromium found" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

render() {           # render <name> <width> <height>
  local name=$1 w=$2 h=$3
  # Sizing the window to the exact artboard makes Chromium clip the bottom of
  # the page, so render into a roomier window and crop back to the true frame.
  # --disable-lcd-text forces greyscale antialiasing: subpixel AA lays coloured
  # fringes on the navy ground, which is visible in a flat brand asset.
  "$chrome" --headless --disable-gpu --no-sandbox --hide-scrollbars \
    --force-device-scale-factor=1 --disable-lcd-text --font-render-hinting=none \
    --virtual-time-budget=8000 \
    --window-size="$((w + 120)),$((h + 200))" \
    --screenshot="$tmp/$name.png" \
    "file://$here/$name.html" >/dev/null 2>&1
  python3 "$here/pngtool.py" crop "$tmp/$name.png" "$out/$name.png" "$w" "$h"
  printf '  %-19s %s  %s bytes\n' "$name.png" \
    "$(python3 "$here/pngtool.py" size "$out/$name.png")" "$(wc -c <"$out/$name.png")"
}

echo "rendering with $chrome"
render repo-header    1280 320
render readme-hero    1280 640
render social-preview 1280 640
echo "done → $out"
