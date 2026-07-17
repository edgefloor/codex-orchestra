#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output=${1:-"$root/target/orchestra-product/orchestra-validate-worker"}

"$root/scripts/verify-evaluator-toolchain.sh"
mkdir -p "$(dirname -- "$output")"
bun install --cwd "$root/evaluator" --frozen-lockfile
bun build "$root/evaluator/worker.ts" --compile --outfile "$output"
revision=$(sed -n 's/^revision = "\(.*\)"/\1/p' "$root/product/pins.toml" | head -n 1)
node "$root/scripts/evaluator-smoke.mjs" "$output" "$revision"
printf '%s\n' "$output"
