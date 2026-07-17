#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sources=${1:?usage: scripts/product-dogfood.sh PREPARED_SOURCES [PRODUCT_OUTPUT]}
output=${2:-"$root/target/orchestra-product"}
desktop="$sources/desktop"

case "$output" in
  /*) ;;
  *) output="$root/$output" ;;
esac

"$root/scripts/product-source-verify.sh" "$sources"
test -x "$output/codex"
test -f "$output/release-manifest.json"
test -f "$desktop/apps/desktop/dist-electron/main.cjs"
test -f "$desktop/apps/server/dist/bin.mjs"
test -f "$desktop/apps/server/dist/client/index.html"

T3CODE_HOME="${ORCHESTRA_T3CODE_HOME:-$output/t3code-home}" \
ORCHESTRA_CODEX_PATH="$output/codex" \
ORCHESTRA_RELEASE_MANIFEST="$output/release-manifest.json" \
  "$desktop/apps/desktop/node_modules/.bin/electron" \
  "$desktop/apps/desktop/dist-electron/main.cjs"
