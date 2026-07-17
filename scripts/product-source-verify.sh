#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sources=${1:?usage: scripts/product-source-verify.sh PREPARED_SOURCES}
pins="$root/product/pins.toml"

fail() {
  echo "$*" >&2
  exit 2
}

pin() {
  key=$1
  value=$(sed -n "s/^${key} = \"\([^\"]*\)\"$/\1/p" "$pins")
  test -n "$value" || fail "missing Product source pin: $key"
  printf '%s\n' "$value"
}

verify_checkout() {
  checkout=$1
  repository=$2
  revision=$3
  tree=$4
  upstream_revision=$5
  upstream_tree=$6
  name=$7

  test -d "$checkout/.git" || fail "missing $name Git checkout: $checkout"
  test "$(git -C "$checkout" remote get-url origin)" = "$repository" \
    || fail "$name origin does not match the sealed fork repository"
  test "$(git -C "$checkout" rev-parse HEAD)" = "$revision" \
    || fail "$name HEAD does not match the sealed fork revision"
  test "$(git -C "$checkout" rev-parse "${revision}^{tree}")" = "$tree" \
    || fail "$name tree does not match the sealed fork tree"
  test -z "$(git -C "$checkout" status --porcelain)" \
    || fail "$name checkout is dirty"
  git -C "$checkout" merge-base --is-ancestor "$upstream_revision" HEAD \
    || fail "$name fork does not descend from its sealed upstream base"
  test "$(git -C "$checkout" rev-parse "${upstream_revision}^{tree}")" = "$upstream_tree" \
    || fail "$name upstream base tree does not match its sealed identity"
}

codex="$sources/codex"
desktop="$sources/desktop"
verify_checkout \
  "$codex" \
  "$(pin orchestra_codex_repository)" \
  "$(pin orchestra_codex)" \
  "$(pin orchestra_codex_tree)" \
  "$(pin codex_upstream)" \
  "$(pin codex_upstream_tree)" \
  "Orchestra Codex"
verify_checkout \
  "$desktop" \
  "$(pin orchestra_desktop_repository)" \
  "$(pin orchestra_desktop)" \
  "$(pin orchestra_desktop_tree)" \
  "$(pin t3code_upstream)" \
  "$(pin t3code_upstream_tree)" \
  "Orchestra Desktop"

test "$(git -C "$root" remote get-url origin)" = "$(pin orchestra_core_repository)" \
  || fail "coordinator origin does not match the sealed orchestra-core repository"
test "$(git -C "$root" rev-parse "$(pin orchestra_core_revision):crates/orchestra-core")" = "$(pin orchestra_core_tree)" \
  || fail "canonical orchestra-core tree does not match the sealed coordinator identity"
test "$(git -C "$codex" rev-parse "$(pin orchestra_codex):codex-rs/app-server-protocol/schema/typescript")" = "$(pin protocol_tree)" \
  || fail "Codex protocol tree does not match the sealed Product identity"

python3 "$codex/scripts/verify-orchestra-provenance.py" --source-root "$root"
python3 "$desktop/scripts/verify-orchestra-provenance.py" --codex-root "$codex"
node "$desktop/scripts/verify-retained-capabilities.mjs"
node -e '
  const fs = require("node:fs");
  const [path, revision, tree, algorithm, digest, count] = process.argv.slice(1);
  const identity = JSON.parse(fs.readFileSync(path, "utf8"));
  if (
    identity.revision !== revision ||
    identity.sourceTree !== tree ||
    identity.digestAlgorithm !== algorithm ||
    identity.digest !== digest ||
    String(identity.fileCount) !== count
  ) process.exit(2);
' \
  "$desktop/apps/web/src/orchestra/generated/ORCHESTRA_CODEX_SOURCE.json" \
  "$(pin orchestra_codex)" \
  "$(pin protocol_tree)" \
  "$(pin protocol_digest_algorithm)" \
  "$(pin protocol_digest)" \
  "$(pin protocol_file_count)" \
  || fail "desktop generated protocol identity does not match Product pins"

echo "Verified direct Product fork tuple"
