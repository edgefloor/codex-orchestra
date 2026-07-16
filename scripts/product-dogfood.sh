#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sources=${1:?usage: scripts/product-dogfood.sh PREPARED_SOURCES [PRODUCT_OUTPUT]}
output=${2:-"$root/target/orchestra-product"}
t3code="$sources/t3code"

case "$output" in
  /*) ;;
  *) output="$root/$output" ;;
esac

"$root/scripts/t3code-integration.sh" "$t3code" verify
test -x "$output/codex"
test -f "$output/release-manifest.json"
test -f "$t3code/apps/desktop/dist-electron/main.cjs"
test -f "$t3code/apps/server/dist/bin.mjs"
test -f "$t3code/apps/server/dist/client/index.html"

T3CODE_HOME="${ORCHESTRA_T3CODE_HOME:-$output/t3code-home}" \
ORCHESTRA_CODEX_PATH="$output/codex" \
ORCHESTRA_RELEASE_MANIFEST="$output/release-manifest.json" \
  "$t3code/apps/desktop/node_modules/.bin/electron" \
  "$t3code/apps/desktop/dist-electron/main.cjs"
