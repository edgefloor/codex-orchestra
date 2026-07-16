#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sources=${1:?usage: scripts/product-dev-build.sh PREPARED_SOURCES [OUTPUT]}
output=${2:-"$root/target/orchestra-product"}
codex="$sources/codex"
t3code="$sources/t3code"

case "$output" in
  /*) ;;
  *) output="$root/$output" ;;
esac

mkdir -p "$output"
rm -f "$output/orchestra-host"

test "$(tr -d '[:space:]' < "$sources/CODEX_REVISION")" = "$(tr -d '[:space:]' < "$root/integration/codex/UPSTREAM_REVISION")"
test "$(tr -d '[:space:]' < "$sources/T3CODE_REVISION")" = "$(tr -d '[:space:]' < "$root/integration/t3code/UPSTREAM_REVISION")"

cargo build --manifest-path "$codex/codex-rs/Cargo.toml" -p codex-cli
"$root/scripts/t3code-integration.sh" "$t3code" build
"$root/scripts/t3code-integration.sh" "$t3code" test
cargo build --manifest-path "$root/Cargo.toml" -p codex-orchestra-product
"$root/scripts/evaluator-build.sh" "$output/orchestra-validate-worker"
ORCHESTRA_EVALUATOR_BIN="$output/orchestra-validate-worker" \
  cargo test --manifest-path "$root/Cargo.toml" -p codex-orchestra-core \
  --test evaluator_worker -- --ignored
cp "$codex/codex-rs/target/debug/codex" "$output/codex"

target=$(rustc -vV | sed -n 's/^host: //p')
"$root/target/debug/orchestra-product" manifest \
  --root "$root" \
  --target "$target" \
  --output "$output/release-manifest.json" \
  --artifact "codex-cli=$output/codex" \
  --artifact "protocol-json=$codex/codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json" \
  --artifact "protocol-client-ts=$codex/codex-rs/app-server-protocol/schema/typescript/ClientRequest.ts" \
  --artifact "orchestra-validate-worker=$output/orchestra-validate-worker" \
  --artifact "desktop-main=$t3code/apps/desktop/dist-electron/main.cjs" \
  --artifact "desktop-preload=$t3code/apps/desktop/dist-electron/preload.cjs" \
  --artifact "desktop-server=$t3code/apps/server/dist/bin.mjs" \
  --artifact "desktop-renderer=$t3code/apps/server/dist/client/index.html"

"$root/target/debug/orchestra-product" host-smoke \
  --host "$output/codex" \
  --host-arg app-server \
  --manifest "$output/release-manifest.json"
"$root/scripts/t3code-integration.sh" \
  "$t3code" \
  smoke \
  "$output/release-manifest.json" \
  "$output/codex"

echo "Built Product development tuple in $output"
