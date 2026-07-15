#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
REVISION=$(tr -d '[:space:]' < "$ROOT/integration/codex/UPSTREAM_REVISION")
DESTINATION=${1:-"$ROOT/.codex/upstream-codex"}
MODE=${2:-verify}

if [[ ! -d "$DESTINATION/.git" ]]; then
  git clone --filter=blob:none https://github.com/openai/codex.git "$DESTINATION"
  git -C "$DESTINATION" checkout --detach "$REVISION"
fi

actual=$(git -C "$DESTINATION" rev-parse HEAD)
if [[ "$actual" != "$REVISION" ]]; then
  echo "expected Codex $REVISION, found $actual" >&2
  exit 2
fi
if [[ -n "$(git -C "$DESTINATION" status --porcelain)" ]]; then
  echo "Codex checkout must be clean before applying the Orchestra overlay" >&2
  exit 2
fi

git -C "$DESTINATION" apply --check "$ROOT/integration/codex/codex-f90e7dee.patch"
git -C "$DESTINATION" apply "$ROOT/integration/codex/codex-f90e7dee.patch"

mkdir -p "$DESTINATION/codex-rs/ext/orchestra-core/src"
mkdir -p "$DESTINATION/codex-rs/ext/orchestra-core/fixtures"
mkdir -p "$DESTINATION/codex-rs/ext/orchestra/src"
cp "$ROOT/integration/codex/overlay/codex-rs/ext/orchestra-core/Cargo.toml" "$DESTINATION/codex-rs/ext/orchestra-core/Cargo.toml"
cp "$ROOT"/crates/orchestra-core/src/*.rs "$DESTINATION/codex-rs/ext/orchestra-core/src/"
cp "$ROOT"/crates/orchestra-core/fixtures/*.workflow.ts "$DESTINATION/codex-rs/ext/orchestra-core/fixtures/"
cp "$ROOT/integration/codex/overlay/codex-rs/ext/orchestra/Cargo.toml" "$DESTINATION/codex-rs/ext/orchestra/Cargo.toml"
cp "$ROOT"/integration/codex/overlay/codex-rs/ext/orchestra/src/*.rs "$DESTINATION/codex-rs/ext/orchestra/src/"
cp "$ROOT/integration/codex/overlay/codex-rs/core/src/orchestra.rs" "$DESTINATION/codex-rs/core/src/orchestra.rs"

if [[ "$MODE" == "apply" ]]; then
  echo "Orchestra overlay applied to $DESTINATION"
  exit 0
fi

cargo test --manifest-path "$DESTINATION/codex-rs/Cargo.toml" -p codex-orchestra-core -p codex-orchestra-extension
cargo test --manifest-path "$DESTINATION/codex-rs/Cargo.toml" -p codex-core orchestra::tests
cargo check --manifest-path "$DESTINATION/codex-rs/Cargo.toml" -p codex-app-server
echo "Pinned Codex integration verified at $REVISION"
