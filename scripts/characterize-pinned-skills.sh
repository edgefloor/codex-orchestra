#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
REVISION=$(tr -d '[:space:]' < "$ROOT/integration/codex/UPSTREAM_REVISION")
CODEX_ROOT=${1:-"$ROOT/.codex/upstream-codex"}

if [[ ! -d "$CODEX_ROOT/.git" || ! -f "$CODEX_ROOT/codex-rs/Cargo.toml" ]]; then
  echo "usage: $0 <pinned-codex-checkout>" >&2
  exit 2
fi

actual=$(git -C "$CODEX_ROOT" rev-parse HEAD)
if [[ "$actual" != "$REVISION" ]]; then
  echo "expected Codex $REVISION, found $actual" >&2
  exit 2
fi

overlay="$ROOT/integration/codex/overlay/codex-rs/core/src/orchestra.rs"
installed="$CODEX_ROOT/codex-rs/core/src/orchestra.rs"
if ! cmp -s "$overlay" "$installed"; then
  echo "Orchestra core overlay is not applied or is stale in $CODEX_ROOT" >&2
  exit 2
fi

manifest="$CODEX_ROOT/codex-rs/Cargo.toml"
require_source() {
  local file=$1
  local needle=$2
  local description=$3
  if ! rg --fixed-strings --quiet "$needle" "$CODEX_ROOT/codex-rs/$file"; then
    echo "pinned source no longer proves: $description" >&2
    exit 3
  fi
}

require_source core/src/agent/control/spawn.rs \
  'self.send_input_after_capacity_check(new_thread.thread_id, &state, input)' \
  'child initial input enters the ordinary turn path'
require_source core/src/session/turn.rs \
  'let mentioned_skills = collect_explicit_skill_mentions(' \
  'turn input drives explicit skill selection'
require_source core/src/session/turn.rs \
  'let SkillInjections {' \
  'selected skill instructions are injected into the turn'
require_source core-skills/src/injection.rs \
  'match fs.read_file_text(&path, /*sandbox*/ None).await {' \
  'an explicitly selected skill body is read at invocation time'
require_source core-skills/src/service.rs \
  'if let Some(snapshot) = self.cached_snapshot_for_config(&cache_key) {' \
  'unchanged effective config reuses discovery metadata'
require_source core-skills/src/model.rs \
  'pub tools: Vec<SkillToolDependency>' \
  'machine-readable dependencies model tools rather than other skills'

cargo test --manifest-path "$manifest" -p codex-core \
  'orchestra::tests::'
RUST_MIN_STACK=33554432 cargo test --manifest-path "$manifest" -p codex-core --test all \
  'user_turn_includes_skill_instructions'
cargo test --manifest-path "$manifest" -p codex-core-skills \
  'collect_explicit_skill_mentions'
cargo test --manifest-path "$manifest" -p codex-core-skills \
  'loads_skill_policy_from_yaml'
cargo test --manifest-path "$manifest" -p codex-core-skills \
  'skills_for_config_reuses_cache_for_same_effective_config'

echo "Pinned native skill behavior characterized at $REVISION"
