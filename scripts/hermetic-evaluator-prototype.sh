#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
prototype="$root/prototypes/hermetic-evaluator"
build="$prototype/.build"
worker="$build/orchestra-validate-worker"

test "$(bun --version)" = "1.3.14"
mkdir -p "$build"
cd "$prototype"
bun install --frozen-lockfile >/dev/null
bun build --compile --target=bun-darwin-arm64 worker.ts --outfile "$worker" >/dev/null
bun "$prototype/harness.ts" "$worker"
