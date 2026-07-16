#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
worker="$root/target/orchestra-evaluator-test/orchestra-validate-worker"

"$root/scripts/evaluator-build.sh" "$worker" >/dev/null
ORCHESTRA_EVALUATOR_BIN="$worker" \
  cargo test --manifest-path "$root/Cargo.toml" -p codex-orchestra-core \
  --test evaluator_worker -- --ignored
