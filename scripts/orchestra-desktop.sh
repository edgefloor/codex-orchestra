#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
target=${1:?usage: scripts/orchestra-desktop.sh TARGET verify|build|test|smoke [PATHS]}
command=${2:?usage: scripts/orchestra-desktop.sh TARGET verify|build|test|smoke [PATHS]}
pins="$root/product/pins.toml"

pin() {
  key=$1
  value=$(sed -n "s/^${key} = \"\([^\"]*\)\"$/\1/p" "$pins")
  test -n "$value" || {
    echo "missing Product source pin: $key" >&2
    exit 2
  }
  printf '%s\n' "$value"
}

verify() {
  test "$(git -C "$target" remote get-url origin)" = "$(pin orchestra_desktop_repository)"
  test "$(git -C "$target" rev-parse HEAD)" = "$(pin orchestra_desktop)"
  test "$(git -C "$target" rev-parse 'HEAD^{tree}')" = "$(pin orchestra_desktop_tree)"
  git -C "$target" merge-base --is-ancestor "$(pin t3code_upstream)" HEAD
  test -z "$(git -C "$target" status --porcelain)"
  python3 "$target/scripts/verify-orchestra-provenance.py"
  node "$target/scripts/verify-retained-capabilities.mjs"
}

case "$command" in
  verify)
    verify
    ;;
  build)
    verify
    (
      cd "$target"
      pnpm install --frozen-lockfile
      pnpm --dir apps/desktop ensure:electron
      pnpm exec vp check
      pnpm exec vp run typecheck
      pnpm exec vp run --filter @t3tools/desktop build
    )
    ;;
  test)
    verify
    (
      cd "$target"
      pnpm --filter @t3tools/web test
      pnpm --filter t3 test
      pnpm --filter @t3tools/desktop test
    )
    ;;
  smoke)
    manifest=${3:?usage: scripts/orchestra-desktop.sh TARGET smoke RELEASE_MANIFEST CODEX_CLI}
    codex_cli=${4:?usage: scripts/orchestra-desktop.sh TARGET smoke RELEASE_MANIFEST CODEX_CLI}
    verify
    test -f "$manifest"
    test -x "$codex_cli"
    test -f "$target/apps/desktop/dist-electron/main.cjs"
    test -f "$target/apps/server/dist/bin.mjs"
    test -f "$target/apps/server/dist/client/index.html"
    node "$root/scripts/t3code-desktop-smoke.mjs" \
      "$target/apps/desktop/node_modules/.bin/electron" \
      "$target/apps/desktop/dist-electron/main.cjs" \
      "$manifest" \
      "$codex_cli"
    ;;
  *)
    echo "unknown command: $command" >&2
    exit 2
    ;;
esac
