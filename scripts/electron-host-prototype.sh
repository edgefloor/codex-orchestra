#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
t3code=${1:?usage: scripts/electron-host-prototype.sh /path/to/pinned-t3code}
expected=ecb35f75839925dd1ac6f854efeef5c9e291d11b

test "$(git -C "$t3code" rev-parse HEAD)" = "$expected"
electron="$t3code/apps/desktop/node_modules/.bin/electron"
test -x "$electron"

cargo build --manifest-path "$root/Cargo.toml" -p codex-orchestra-host-prototype
"$electron" "$root/prototypes/desktop-host/electron" "$root/target/debug/orchestra-host-prototype"
