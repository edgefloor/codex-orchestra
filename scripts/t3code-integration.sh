#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
target=${1:?usage: scripts/t3code-integration.sh TARGET apply|verify|build|test|smoke [PATH]}
command=${2:?usage: scripts/t3code-integration.sh TARGET apply|verify|build|test|smoke [PATH]}
protocol=${3:-}
revision=$(tr -d '[:space:]' < "$root/integration/t3code/UPSTREAM_REVISION")
overlay="$root/integration/t3code/overlay"
patch="$root/integration/t3code/t3code-ecb35f75.patch"
generated="$target/apps/web/src/orchestra/generated"

if test ! -e "$target/.git"; then
  echo "not a T3Code checkout: $target" >&2
  exit 2
fi
if test "$(git -C "$target" rev-parse HEAD)" != "$revision"; then
  echo "T3Code checkout is not pinned to $revision" >&2
  exit 2
fi

apply_overlay() {
  git -C "$target" apply "$patch"
  find "$overlay" -type f | while IFS= read -r file; do
    relative=${file#"$overlay"/}
    mkdir -p "$(dirname "$target/$relative")"
    cp "$file" "$target/$relative"
  done

  if test -z "$protocol" || test ! -f "$protocol/ClientRequest.ts"; then
    echo "generated Codex TypeScript schema directory is required when applying the desktop overlay" >&2
    exit 2
  fi
  rm -rf "$generated"
  mkdir -p "$generated"
  cp -R "$protocol/." "$generated/"
}

verify_overlay() {
  git -C "$target" apply --reverse --check "$patch"
  find "$overlay" -type f | while IFS= read -r file; do
    relative=${file#"$overlay"/}
    cmp "$file" "$target/$relative"
  done
  test -f "$generated/ClientRequest.ts"
  test -f "$generated/v2/OrchestraRunProjection.ts"
  test -f "$generated/v2/OrchestraTaskReplay.ts"
  test -f "$generated/v2/OrchestraQueryResponse.ts"
  test -f "$generated/v2/OrchestraExecutionStepProjection.ts"
  test -f "$generated/v2/OrchestraEvidenceReference.ts"
  test -f "$generated/v2/AutomationValidateParams.ts"
  test -f "$generated/v2/AutomationValidateResponse.ts"
  test -f "$generated/v2/AutomationRunFixtureParams.ts"
  test -f "$generated/v2/AutomationRunResponse.ts"
  test -f "$generated/v2/AutomationEffectReceiptProjection.ts"
  test -f "$generated/v2/AutomationEffectStatus.ts"
  test -f "$generated/v2/AutomationGatePolicy.ts"
  test -f "$generated/v2/AutomationLinearReadParams.ts"
  test -f "$generated/v2/AutomationLinearReadResponse.ts"
  test -f "$generated/v2/AutomationQueueReadParams.ts"
  test -f "$generated/v2/AutomationQueueReadResponse.ts"
  test -f "$generated/v2/AutomationReconcileParams.ts"
  test -f "$generated/v2/AutomationReconciliationStatus.ts"
  rg -q 'ORCHESTRA_CODEX_PATH' "$target/apps/server/src/provider/Drivers/CodexDriver.ts"
  rg -q 'automation/validate' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/runFixture' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/linear/read' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/queue/read' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/status' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/pause' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/refresh' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/resume' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/cancel' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'expectedProductManifestSha256' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'orchestra/lifecycle' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'orchestra/query' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'OrchestraLifecycleEntry' "$target/apps/web/src/components/chat/MessagesTimeline.tsx"
  rg -q 'AutomationProfileDialog' "$target/apps/web/src/components/chat/ChatHeader.tsx"
  rg -q 'const app = <AppRoot router=\{router\} />' "$target/apps/web/src/main.tsx"
  rg -q 'DesktopApp.program' "$target/apps/desktop/src/main.ts"
  if test -f "$target/apps/web/src/orchestra/App.tsx"; then
    echo "replacement Orchestra workflow dashboard is still present" >&2
    exit 1
  fi
}

case "$command" in
  apply)
    apply_overlay
    verify_overlay
    ;;
  verify)
    verify_overlay
    ;;
  build)
    verify_overlay
    (
      cd "$target"
      pnpm install --frozen-lockfile
      pnpm --dir apps/desktop ensure:electron
      pnpm exec vp run --filter @t3tools/web build
      pnpm exec vp run --filter @t3tools/desktop build
      pnpm --dir apps/web typecheck
      pnpm --dir apps/desktop typecheck
    )
    ;;
  test)
    verify_overlay
    (
      cd "$target"
      pnpm --dir apps/web exec vp test run \
        src/components/chat/AutomationProfileDialog.logic.test.ts \
        src/components/chat/MessagesTimeline.logic.test.ts \
        src/components/chat/MessagesTimeline.test.tsx \
        src/session-logic.test.ts
      pnpm --dir apps/server exec vp test run \
        src/provider/Layers/CodexAdapter.test.ts \
        src/provider/Layers/CodexSessionRuntime.test.ts \
        src/orchestration/Layers/ProviderCommandReactor.test.ts
    )
    ;;
  smoke)
    manifest=${3:?usage: scripts/t3code-integration.sh TARGET smoke RELEASE_MANIFEST CODEX_CLI}
    codex_cli=${4:?usage: scripts/t3code-integration.sh TARGET smoke RELEASE_MANIFEST CODEX_CLI}
    verify_overlay
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
