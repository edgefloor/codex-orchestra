#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output=${1:-"$root/target/orchestra-product/orchestra-validate-worker"}

test "$(bun --version)" = "1.3.14"
mkdir -p "$(dirname -- "$output")"
bun install --cwd "$root/evaluator" --frozen-lockfile
bun build "$root/evaluator/worker.ts" --compile --outfile "$output"
printf '%s\n' "$output"
