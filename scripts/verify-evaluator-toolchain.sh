#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
pins="$root/product/pins.toml"
remote=${1:-}

fail() {
  echo "$*" >&2
  exit 2
}

pin() {
  sed -n "s/^$1 = \"\(.*\)\"/\1/p" "$pins" | head -n 1
}

sha256_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

require_equal() {
  test "$1" = "$2" || fail "$3: expected $2, got $1"
}

bun_version=$(pin bun_version)
bun_revision=$(pin bun)
zod_version=$(pin zod_version)
zod_revision=$(pin zod)
zod_package_revision=$(pin zod_package_revision)
zod_integrity=$(pin zod_package_integrity)
zod_shasum=$(pin zod_package_shasum)
evaluator_revision=$(sed -n 's/^revision = "\(.*\)"/\1/p' "$pins" | head -n 1)

require_equal "$(bun --version)" "$bun_version" "Bun version mismatch"
require_equal "$(bun --revision)" "$bun_version+$(printf '%s' "$bun_revision" | cut -c1-9)" \
  "Bun release revision mismatch"
require_equal "$(sha256_file "$root/evaluator/worker.ts")" \
  "$(pin evaluator_worker_source_sha256)" "evaluator worker source digest mismatch"
require_equal "$(sha256_file "$root/evaluator/bun.lock")" \
  "$(pin evaluator_lock_sha256)" "evaluator lockfile digest mismatch"
require_equal "$(sha256_file "$root/evaluator/package.json")" \
  "$(pin evaluator_package_sha256)" "evaluator package digest mismatch"

node -e '
  const fs = require("node:fs");
  const [path, expected] = process.argv.slice(1);
  const value = JSON.parse(fs.readFileSync(path, "utf8"));
  if (value.dependencies?.zod !== expected) process.exit(1);
' "$root/evaluator/package.json" "$zod_version" \
  || fail "evaluator package does not pin Zod $zod_version exactly"
grep -Fq "zod@$zod_version" "$root/evaluator/bun.lock" \
  || fail "evaluator lockfile does not pin Zod $zod_version"
grep -Fq "$zod_integrity" "$root/evaluator/bun.lock" \
  || fail "evaluator lockfile does not contain the pinned Zod package integrity"
grep -Fq "const EVALUATOR_REVISION = \"$evaluator_revision\"" "$root/evaluator/worker.ts" \
  || fail "evaluator worker does not embed the Product evaluator revision"

bun_build_scripts=$(rg -l 'bun build' "$root/scripts" \
  --glob '!verify-evaluator-toolchain.sh' | sort)
expected_bun_build_scripts=$(printf '%s\n%s' \
  "$root/scripts/evaluator-build.sh" \
  "$root/scripts/product-release.sh" | sort)
require_equal "$bun_build_scripts" "$expected_bun_build_scripts" \
  "Bun compilation escaped the sealed evaluator boundary"
for script in $bun_build_scripts; do
  grep -Fq '$root/evaluator/worker.ts' "$script" \
    || fail "Bun compilation in $script does not target the sealed evaluator worker"
done
if rg -n 'bun[^\n]*(workflow|\.workflow\.ts)|(workflow|\.workflow\.ts)[^\n]*bun' \
  "$root/scripts" "$root/crates" --glob '!verify-evaluator-toolchain.sh' >/dev/null; then
  fail "workflow TypeScript must remain Rust-parsed authoring input and must never be passed to Bun"
fi

if test "$remote" = --remote; then
  bun_repository=$(pin bun_repository)
  zod_repository=$(pin zod_repository)
  require_equal "$(git ls-remote "$bun_repository" "refs/tags/bun-v$bun_version" | awk '{print $1}')" \
    "$bun_revision" "Bun release tag revision mismatch"
  require_equal "$(git ls-remote "$zod_repository" "refs/tags/v$zod_version" | awk '{print $1}')" \
    "$zod_revision" "Zod release tag revision mismatch"
  npm_metadata=$(npm view "zod@$zod_version" gitHead dist.integrity dist.shasum --json)
  node -e '
    const value = JSON.parse(process.argv[1]);
    const expected = JSON.parse(process.argv[2]);
    if (value.gitHead !== expected.revision ||
        value["dist.integrity"] !== expected.integrity ||
        value["dist.shasum"] !== expected.shasum) process.exit(1);
  ' "$npm_metadata" \
    "{\"revision\":\"$zod_package_revision\",\"integrity\":\"$zod_integrity\",\"shasum\":\"$zod_shasum\"}" \
    || fail "published Zod package provenance does not match Product pins"
elif test -n "$remote"; then
  fail "usage: scripts/verify-evaluator-toolchain.sh [--remote]"
fi

echo "Verified sealed Bun $bun_version / Zod $zod_version evaluator toolchain"
