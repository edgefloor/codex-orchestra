#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)

cargo build --manifest-path "$ROOT/Cargo.toml" -p codex-orchestra-host-prototype
node --test "$ROOT/prototypes/desktop-host/renderer-adapter.test.mjs"
node "$ROOT/prototypes/desktop-host/run.mjs" "$ROOT/target/debug/orchestra-host-prototype"
