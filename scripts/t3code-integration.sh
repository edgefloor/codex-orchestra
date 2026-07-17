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
  test -f "$generated/v2/OrchestraEvidenceContentProjection.ts"
  test -f "$generated/v2/AutomationValidateParams.ts"
  test -f "$generated/v2/AutomationValidateResponse.ts"
  test -f "$generated/v2/AutomationRunFixtureParams.ts"
  test -f "$generated/v2/AutomationRunResponse.ts"
  test -f "$generated/v2/AutomationProfileRevisionStatus.ts"
  test -f "$generated/v2/AutomationEffectReceiptProjection.ts"
  test -f "$generated/v2/AutomationEffectStatus.ts"
  test -f "$generated/v2/AutomationGatePolicy.ts"
  test -f "$generated/v2/AutomationLinearReadParams.ts"
  test -f "$generated/v2/AutomationLinearReadResponse.ts"
  test -f "$generated/v2/AutomationQueueReadParams.ts"
  test -f "$generated/v2/AutomationQueueReadResponse.ts"
  test -f "$generated/v2/AutomationReconcileParams.ts"
  test -f "$generated/v2/AutomationCancelIssueParams.ts"
  test -f "$generated/v2/AutomationReconciliationStatus.ts"
  rg -q 'ORCHESTRA_CODEX_PATH' "$target/apps/server/src/provider/Drivers/CodexDriver.ts"
  rg -q '"electron-updater": "6.8.3"' "$target/apps/desktop/package.json"
  rg -q 'specifier: 6.8.3' "$target/pnpm-lock.yaml"
  rg -q 'ORCHESTRA_PRODUCT_RESOURCES' "$target/scripts/build-desktop-artifact.ts"
  rg -q 'extraResources' "$target/scripts/build-desktop-artifact.ts"
  rg -q 'com.edgefloor.orchestra' "$target/scripts/build-desktop-artifact.ts"
  rg -q 'ORCHESTRA_RELEASE_MANIFEST' "$target/apps/desktop/src/backend/DesktopBackendConfiguration.ts"
  rg -q 'ORCHESTRA_EVALUATOR_BIN' "$target/apps/desktop/src/backend/DesktopBackendConfiguration.ts"
  rg -q 'desktop-update-stage' "$target/apps/desktop/src/updates/OrchestraProductLifecycle.ts"
  rg -q 'beginStartup' "$target/apps/desktop/src/app/DesktopApp.ts"
  rg -q 'stageUpdate' "$target/apps/desktop/src/updates/DesktopUpdates.ts"
  rg -q 'APP_BASE_NAME.*Orchestra' "$target/apps/web/src/branding.ts"
  test "$(rg -c -F 'setDisableDifferentialDownload(true)' "$target/apps/desktop/src/updates/DesktopUpdates.ts")" -eq 2
  rg -q 'automation/validate' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/runFixture' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/linear/read' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/queue/read' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/status' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/pause' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/refresh' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/resume' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/cancelIssue' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'automation/cancel' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'expectedProductManifestSha256' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'orchestra/lifecycle' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'orchestra/query' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'evidence_content' "$target/packages/contracts/src/orchestra.ts"
  rg -q 'OrchestraLifecycleEntry' "$target/apps/web/src/components/chat/MessagesTimeline.tsx"
  rg -q 'aria-label="Symphony automation"' "$target/apps/web/src/components/chat/ChatHeader.tsx"
  rg -q 'AutomationWorkspace' "$target/apps/web/src/components/ChatView.tsx"
  rg -q 'data-automation-workspace' "$target/apps/web/src/components/chat/AutomationProfileDialog.tsx"
  rg -q 'Cancel issue' "$target/apps/web/src/components/chat/AutomationProfileDialog.tsx"
  rg -q 'automationRunStorageKey' "$target/apps/web/src/components/chat/AutomationProfileDialog.tsx"
  rg -q 'Reattach' "$target/apps/web/src/components/chat/AutomationProfileDialog.tsx"
  test -f "$target/apps/web/src/components/chat/AutomationProfileDialog.test.tsx"
  if rg -q 'DialogPopup|DialogTrigger' "$target/apps/web/src/components/chat/AutomationProfileDialog.tsx"; then
    echo "detached Symphony modal is still present" >&2
    exit 1
  fi
  rg -q 'data-task-attention' "$target/apps/web/src/components/chat/TaskAttentionView.tsx"
  rg -q 'TaskAttentionView' "$target/apps/web/src/components/ChatView.tsx"
  rg -q 'ComposerBannerStack' "$target/apps/web/src/components/ChatView.tsx"
  rg -q 'ComposerPendingApprovalActions' "$target/apps/web/src/components/chat/TaskAttentionView.tsx"
  test -f "$target/apps/web/src/components/chat/TaskAttentionView.logic.test.ts"
  test -f "$target/apps/web/src/components/chat/TaskAttentionView.test.tsx"
  test -f "$target/apps/web/src/nativeWorkspaceDogfood.test.ts"
  rg -q -- '--orchestra-canvas: #f5f7fa' "$target/apps/web/src/index.css"
  rg -q -- '--orchestra-canvas: #0d0d0d' "$target/apps/web/src/index.css"
  rg -q '"SF Pro Text"' "$target/apps/web/src/index.css"
  rg -q 'bg-sidebar text-sidebar-foreground' "$target/apps/web/src/components/AppSidebarLayout.tsx"
  test -f "$target/apps/web/src/orchestraTheme.test.ts"
  rg -q 'data-workspace-task-tabs' "$target/apps/web/src/components/WorkspaceTaskTabs.tsx"
  rg -q 'data-workspace-context-rail' "$target/apps/web/src/components/WorkspaceContextRail.tsx"
  rg -q 'WorkspaceContextRail' "$target/apps/web/src/components/ChatView.tsx"
  test -f "$target/apps/web/src/components/WorkspaceContextRail.test.tsx"
  rg -q 'aria-label="Choose project"' "$target/apps/web/src/components/Sidebar.tsx"
  rg -q 'useThreadShells' "$target/apps/web/src/components/ChatView.tsx"
  test -f "$target/apps/web/src/components/WorkspaceTaskTabs.logic.test.ts"
  test -f "$target/apps/web/src/components/WorkspaceTaskTabs.test.tsx"
  rg -q 'data-native-subagents' "$target/apps/web/src/components/chat/NativeSubagentsPanel.tsx"
  rg -q 'readNativeSubagent' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  rg -q 'isDirectNativeSubagent' "$target/apps/server/src/provider/Layers/CodexSessionRuntime.ts"
  test -f "$target/apps/web/src/nativeSubagents.test.ts"
  test -f "$target/apps/web/src/components/chat/NativeSubagentsPanel.test.tsx"
  rg -q 'role="tree"' "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.tsx"
  rg -q -F 'load("outputs", stepId)' "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.tsx"
  rg -q -F 'load("history")' "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.tsx"
  rg -q -F 'load("evidence_content"' "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.tsx"
  rg -q 'provenance' "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.tsx"
  test -f "$target/apps/web/src/components/chat/WorkflowRunTree.logic.test.ts"
  test -f "$target/apps/web/src/components/chat/OrchestraLifecycleEntry.test.tsx"
  if rg -q '@fontsource-variable/dm-sans|@fontsource/jetbrains-mono' "$target/apps/web/src/main.tsx"; then
    echo "legacy bundled renderer fonts are still active" >&2
    exit 1
  fi
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
      pnpm test
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
