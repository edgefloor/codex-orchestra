#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
command=${1:-help}

fail() {
  echo "$*" >&2
  exit 2
}

absolute() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *) printf '%s\n' "$root/$1" ;;
  esac
}

require_file() {
  test -f "$1" || fail "missing file: $1"
}

require_tool() {
  command -v "$1" >/dev/null 2>&1 || fail "missing release tool: $1"
}

verify_sources() {
  sources=$1
  "$root/scripts/product-source-verify.sh" "$sources"
  rg -q '"electron-updater": "6.8.3"' "$sources/desktop/apps/desktop/package.json" \
    || fail "Orchestra Desktop does not pin electron-updater 6.8.3 exactly"
  rg -q 'specifier: 6.8.3' "$sources/desktop/pnpm-lock.yaml" \
    || fail "Orchestra Desktop lockfile does not pin electron-updater 6.8.3 exactly"
}

release_preflight() {
  sources=$(absolute "$1")
  verify_sources "$sources"
  for tool in cargo rustup bun pnpm node codesign xcrun spctl lipo ditto; do
    require_tool "$tool"
  done
  "$root/scripts/verify-evaluator-toolchain.sh" --remote
  cargo run --quiet -p codex-orchestra-product -- doctor --root "$root"
  echo "OK release preflight"
}

signing_enabled() {
  test -n "${CSC_LINK:-}" \
    && test -n "${CSC_KEY_PASSWORD:-}" \
    && test -n "${APPLE_API_KEY:-}" \
    && test -n "${APPLE_API_KEY_ID:-}" \
    && test -n "${APPLE_API_ISSUER:-}" \
    && test -n "${T3CODE_APPLE_TEAM_ID:-}" \
    && test -n "${T3CODE_MACOS_PROVISIONING_PROFILE:-}"
}

build_architecture() {
  sources=$1
  output=$2
  target=$3
  desktop_arch=$4
  bun_target=$5
  version=$6
  signed=$7
  codex="$sources/codex"
  desktop="$sources/desktop"
  target_output="$output/$target"
  resources="$target_output/resources"
  artifacts="$target_output/artifacts"

  mkdir -p "$resources" "$artifacts"
  cargo build --manifest-path "$codex/codex-rs/Cargo.toml" -p codex-cli --release --target "$target"
  cargo build --manifest-path "$root/Cargo.toml" -p codex-orchestra-product --release --target "$target"
  bun build "$root/evaluator/worker.ts" \
    --compile \
    --target="$bun_target" \
    --outfile "$resources/orchestra-validate-worker"
  evaluator_revision=$(sed -n 's/^revision = "\(.*\)"/\1/p' "$root/product/pins.toml" | head -n 1)
  node "$root/scripts/evaluator-smoke.mjs" \
    "$resources/orchestra-validate-worker" "$evaluator_revision"
  cp "$codex/codex-rs/target/$target/release/codex" "$resources/codex"
  cp "$root/target/$target/release/orchestra-product" "$resources/orchestra-product"
  cp "$root/product/release.toml" "$resources/release.toml"

  printf '%s\n' \
    '{' \
    '  "schemaVersion": 1,' \
    '  "active": false,' \
    '  "installsNativeExecution": false,' \
    '  "lifecycle": "explicit"' \
    '}' > "$resources/plugin-baseline.json"

  cargo run --quiet -p codex-orchestra-product -- manifest \
    --root "$root" \
    --target "$target" \
    --output "$resources/release-manifest.json" \
    --artifact "codex-cli=$resources/codex" \
    --artifact "orchestra-product=$resources/orchestra-product" \
    --artifact "orchestra-validate-worker=$resources/orchestra-validate-worker" \
    --artifact "plugin-baseline=$resources/plugin-baseline.json" \
    --artifact "protocol-json=$codex/codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json" \
    --artifact "desktop-main=$desktop/apps/desktop/dist-electron/main.cjs" \
    --artifact "desktop-server=$desktop/apps/server/dist/bin.mjs" \
    --artifact "desktop-renderer=$desktop/apps/server/dist/client/index.html"

  set -- pnpm --dir "$desktop" exec node "$desktop/scripts/build-desktop-artifact.ts" \
    --platform mac \
    --target dmg \
    --arch "$desktop_arch" \
    --build-version "$version" \
    --output-dir "$artifacts" \
    --skip-build
  if test "$signed" = true; then
    set -- "$@" --signed
  fi
  ORCHESTRA_PRODUCT_RESOURCES="$resources" "$@"

  find "$artifacts" -maxdepth 1 -type f -name 'latest*.yml' -exec \
    node "$root/scripts/annotate-update-metadata.mjs" "$resources/release-manifest.json" {} \;

  archive=$(find "$artifacts" -maxdepth 1 -type f -name '*.zip' -print | head -n 1)
  test -n "$archive" || fail "no full-app zip produced for $target"
  cp "$resources/release-manifest.json" "$target_output/release-manifest.json"
  echo "Built $target candidate in $target_output"
}

build_release() {
  sources=$(absolute "$1")
  output=$(absolute "$2")
  test ! -e "$output" || fail "release output already exists: $output"
  release_preflight "$sources"

  signed=true
  if ! signing_enabled; then
    if test "${ORCHESTRA_ALLOW_UNSIGNED:-0}" != 1; then
      fail "Apple signing inputs are required; set ORCHESTRA_ALLOW_UNSIGNED=1 only for a non-publishable rehearsal"
    fi
    signed=false
    echo "Building a non-publishable unsigned rehearsal" >&2
  fi

  version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/product/pins.toml" | head -n 1)
  test -n "$version" || fail "Product version is missing"
  mkdir -p "$output"

  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  "$root/scripts/orchestra-desktop.sh" "$sources/desktop" build
  "$root/scripts/orchestra-desktop.sh" "$sources/desktop" test
  bun install --cwd "$root/evaluator" --frozen-lockfile

  build_architecture "$sources" "$output" aarch64-apple-darwin arm64 bun-darwin-arm64 "$version" "$signed"
  build_architecture "$sources" "$output" x86_64-apple-darwin x64 bun-darwin-x64 "$version" "$signed"

  mkdir -p "$output/evidence" "$output/source"
  cp "$root/LICENSE" "$output/evidence/ORCHESTRA-LICENSE.txt"
  cp "$sources/codex/LICENSE" "$output/evidence/CODEX-LICENSE.txt"
  cp "$sources/desktop/LICENSE" "$output/evidence/ORCHESTRA-DESKTOP-LICENSE.txt"
  cp "$root/product/pins.toml" "$output/evidence/PRODUCT-PINS.toml"
  cp "$sources/codex/orchestra-provenance.json" "$output/evidence/ORCHESTRA-CODEX-PROVENANCE.json"
  cp "$sources/desktop/orchestra-provenance.json" "$output/evidence/ORCHESTRA-DESKTOP-PROVENANCE.json"
  cargo metadata --format-version 1 --locked > "$output/evidence/orchestra-cargo-metadata.json"
  cargo metadata \
    --manifest-path "$sources/codex/codex-rs/Cargo.toml" \
    --format-version 1 \
    --locked > "$output/evidence/codex-cargo-metadata.json"
  (cd "$sources/desktop" && pnpm licenses list --json --prod) > "$output/evidence/pnpm-licenses.json"
  created=${SOURCE_DATE_EPOCH:+$(date -u -r "$SOURCE_DATE_EPOCH" '+%Y-%m-%dT%H:%M:%SZ')}
  created=${created:-$(date -u '+%Y-%m-%dT%H:%M:%SZ')}
  node "$root/scripts/generate-spdx-sbom.mjs" \
    --version "$version" \
    --created "$created" \
    --cargo "orchestra=$output/evidence/orchestra-cargo-metadata.json" \
    --cargo "codex=$output/evidence/codex-cargo-metadata.json" \
    --pnpm "$output/evidence/pnpm-licenses.json" \
    --pins "$root/product/pins.toml" \
    --output "$output/evidence/orchestra.spdx.json" \
    --notices "$output/evidence/THIRD-PARTY-NOTICES.md"
  tar \
    --exclude='.git' \
    --exclude='node_modules' \
    --exclude='target' \
    -czf "$output/source/corresponding-source.tar.gz" \
    -C "$root" . \
    -C "$sources" codex desktop
  echo "Built release candidate. Complete signed distribution and gate evidence before sealing."
}

verify_architecture() {
  candidate=$(absolute "$1")
  target=$2
  directory="$candidate/$target"
  archive=$(find "$directory/artifacts" -maxdepth 1 -type f -name '*.zip' -print | head -n 1)
  test -n "$archive" || fail "no zip archive found for $target"
  temporary=$(mktemp -d "${TMPDIR:-/tmp}/orchestra-release-verify.XXXXXX")
  trap 'rm -rf "$temporary"' EXIT HUP INT TERM
  ditto -x -k "$archive" "$temporary"
  app=$(find "$temporary" -maxdepth 2 -type d -name '*.app' -print | head -n 1)
  test -n "$app" || fail "archive contains no application"
  codesign --verify --deep --strict --verbose=2 "$app"
  codesign -dv --verbose=4 "$app" 2>&1 | rg -q 'flags=.*runtime' \
    || fail "Hardened Runtime is not enabled"
  spctl --assess --type execute --verbose=2 "$app"
  xcrun stapler validate "$app"
  bundled="$app/Contents/Resources/orchestra"
  require_file "$bundled/codex"
  require_file "$bundled/orchestra-product"
  require_file "$bundled/orchestra-validate-worker"
  require_file "$bundled/release-manifest.json"
  case "$target" in
    aarch64-apple-darwin) expected_arch=arm64 ;;
    x86_64-apple-darwin) expected_arch=x86_64 ;;
    *) fail "unknown target: $target" ;;
  esac
  lipo -archs "$bundled/codex" | rg -q "(^| )$expected_arch( |$)" \
    || fail "bundled Codex has the wrong architecture"
  "$bundled/orchestra-product" verify-manifest --manifest "$bundled/release-manifest.json"
  for artifact in \
    "codex-cli=$bundled/codex" \
    "orchestra-product=$bundled/orchestra-product" \
    "orchestra-validate-worker=$bundled/orchestra-validate-worker" \
    "plugin-baseline=$bundled/plugin-baseline.json"
  do
    name=${artifact%%=*}
    path=${artifact#*=}
    "$bundled/orchestra-product" verify-artifact \
      --manifest "$bundled/release-manifest.json" \
      --name "$name" \
      --artifact "$path"
  done
  evaluator_revision=$(sed -n 's/^revision = "\(.*\)"/\1/p' "$root/product/pins.toml" | head -n 1)
  node "$root/scripts/evaluator-smoke.mjs" \
    "$bundled/orchestra-validate-worker" "$evaluator_revision"
  echo "OK signed architecture: $target"
}

seal_release() {
  candidate=$(absolute "$1")
  evidence=$(absolute "$2")
  cargo run --quiet -p codex-orchestra-product -- release-gate \
    --root "$root" \
    --candidate "$candidate" \
    --evidence "$evidence" \
    --output "$candidate/release-record.json"
}

publication_gate() {
  candidate=$(absolute "$1")
  evidence=$(absolute "$2")
  cargo run --quiet -p codex-orchestra-product -- publication-gate \
    --candidate "$candidate" \
    --record "$candidate/release-record.json" \
    --evidence "$evidence"
}

case "$command" in
  preflight)
    test "$#" -eq 2 || fail "usage: scripts/product-release.sh preflight PREPARED_SOURCES"
    release_preflight "$2"
    ;;
  build)
    test "$#" -eq 3 || fail "usage: scripts/product-release.sh build PREPARED_SOURCES OUTPUT"
    build_release "$2" "$3"
    ;;
  verify-arch)
    test "$#" -eq 3 || fail "usage: scripts/product-release.sh verify-arch CANDIDATE TARGET"
    verify_architecture "$2" "$3"
    ;;
  seal)
    test "$#" -eq 3 || fail "usage: scripts/product-release.sh seal CANDIDATE RELEASE_EVIDENCE_JSON"
    seal_release "$2" "$3"
    ;;
  publication-gate)
    test "$#" -eq 3 || fail "usage: scripts/product-release.sh publication-gate CANDIDATE PUBLICATION_EVIDENCE_JSON"
    publication_gate "$2" "$3"
    ;;
  *)
    echo "usage: scripts/product-release.sh preflight|build|verify-arch|seal|publication-gate ..."
    ;;
esac
