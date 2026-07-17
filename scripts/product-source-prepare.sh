#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
destination=${1:?usage: scripts/product-source-prepare.sh DESTINATION}
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

codex_repository=$(pin orchestra_codex_repository)
codex_revision=$(pin orchestra_codex)
desktop_repository=$(pin orchestra_desktop_repository)
desktop_revision=$(pin orchestra_desktop)

if test -e "$destination"; then
  echo "destination already exists: $destination" >&2
  exit 2
fi

mkdir -p "$destination"
git clone --filter=blob:none "$codex_repository" "$destination/codex"
git -C "$destination/codex" checkout --detach "$codex_revision"

git clone --filter=blob:none "$desktop_repository" "$destination/desktop"
git -C "$destination/desktop" checkout --detach "$desktop_revision"

"$root/scripts/product-source-verify.sh" "$destination"
echo "Prepared pinned Product sources in $destination"
