#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sources=${1:?usage: scripts/product-dev-build.sh PREPARED_SOURCES [OUTPUT]}
output=${2:-"$root/target/orchestra-product"}
codex="$sources/codex"
desktop="$sources/desktop"

case "$output" in
  /*) ;;
  *) output="$root/$output" ;;
esac

mkdir -p "$output"
rm -f "$output/orchestra-host"

"$root/scripts/product-source-verify.sh" "$sources"

cargo build --manifest-path "$codex/codex-rs/Cargo.toml" -p codex-cli
"$root/scripts/orchestra-desktop.sh" "$desktop" build
"$root/scripts/orchestra-desktop.sh" "$desktop" test
cargo build --manifest-path "$root/Cargo.toml" -p codex-orchestra-product
"$root/scripts/evaluator-build.sh" "$output/orchestra-validate-worker"
ORCHESTRA_EVALUATOR_BIN="$output/orchestra-validate-worker" \
  cargo test --manifest-path "$root/Cargo.toml" -p codex-orchestra-core \
  --test evaluator_worker -- --ignored
cp "$codex/codex-rs/target/debug/codex" "$output/codex"
cp "$root/product/release.toml" "$output/release.toml"

target=$(rustc -vV | sed -n 's/^host: //p')
"$root/target/debug/orchestra-product" manifest \
  --root "$root" \
  --target "$target" \
  --output "$output/release-manifest.json" \
  --artifact "codex-cli=$output/codex" \
  --artifact "protocol-json=$codex/codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json" \
  --artifact "protocol-client-ts=$codex/codex-rs/app-server-protocol/schema/typescript/ClientRequest.ts" \
  --artifact "orchestra-validate-worker=$output/orchestra-validate-worker" \
  --artifact "desktop-main=$desktop/apps/desktop/dist-electron/main.cjs" \
  --artifact "desktop-preload=$desktop/apps/desktop/dist-electron/preload.cjs" \
  --artifact "desktop-server=$desktop/apps/server/dist/bin.mjs" \
  --artifact "desktop-renderer=$desktop/apps/server/dist/client/index.html"

"$root/target/debug/orchestra-product" host-smoke \
  --host "$output/codex" \
  --host-arg app-server \
  --manifest "$output/release-manifest.json"
"$root/scripts/orchestra-desktop.sh" \
  "$desktop" \
  smoke \
  "$output/release-manifest.json" \
  "$output/codex"

echo "Built Product development tuple in $output"
