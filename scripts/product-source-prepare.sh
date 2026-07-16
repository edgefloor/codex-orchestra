#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
destination=${1:?usage: scripts/product-source-prepare.sh DESTINATION}
codex_revision=$(tr -d '[:space:]' < "$root/integration/codex/UPSTREAM_REVISION")
t3code_revision=$(tr -d '[:space:]' < "$root/integration/t3code/UPSTREAM_REVISION")

if test -e "$destination"; then
  echo "destination already exists: $destination" >&2
  exit 2
fi

mkdir -p "$destination"
git clone --filter=blob:none https://github.com/openai/codex.git "$destination/codex"
git -C "$destination/codex" checkout --detach "$codex_revision"
"$root/scripts/codex-integration.sh" "$destination/codex" apply

git clone --filter=blob:none https://github.com/pingdotgg/t3code.git "$destination/t3code"
git -C "$destination/t3code" checkout --detach "$t3code_revision"
"$root/scripts/t3code-integration.sh" \
  "$destination/t3code" \
  apply \
  "$destination/codex/codex-rs/app-server-protocol/schema/typescript"

cp "$root/integration/codex/UPSTREAM_REVISION" "$destination/CODEX_REVISION"
cp "$root/integration/t3code/UPSTREAM_REVISION" "$destination/T3CODE_REVISION"
echo "Prepared pinned Product sources in $destination"
