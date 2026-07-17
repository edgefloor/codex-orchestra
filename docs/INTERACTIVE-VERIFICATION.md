# Interactive verification

## Stage 0 — automated baseline

Run the Rust workspace suite, lifecycle doctor, plugin validator, and pinned Codex integration build. Record exact revisions and exit status.

## Stage 1 — fresh plugin discovery

Install the packaged candidate, start a task outside the source repository, invoke `$orchestra:orchestrate`, and confirm it requires the native tool surface rather than emulating a scheduler.

## Stage 2 — native tool surface

Using the Orchestra-enabled Codex build, confirm `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, `orchestra_cancel`, and `orchestra_query` appear as native tools.

## Stage 3 — V2 vertical slice

Invoke `evals/workflows/native-vertical-slice.workflow.ts` through the active parent task's native Orchestra skill/tool path. Confirm the Root Run is owned by that task and no renderer RPC, linked normal thread, or external scheduler creates it. Observe canonical child task paths, explicit model/reasoning, no parent transcript for `fork_turns: none`, parallel activity, V2 completion, isolated changes present in the shared worktree before the sandboxed check, approval pause, accepted promotion into the target checkout as unstaged changes, isolated and terminal shared-worktree cleanup, and final summary.

For the accepted desktop target, also verify exact-bundle initialization without task cursors,
per-task snapshot and cursor subscription, lazy child-task subscription on expansion, typed
`ThreadItem::Orchestra` hydration, World State replacement of the Run Digest, deterministic digest
truncation with stable omission references, and identical bounded query selection through native
task-tool and App Server adapters. Keep these checks pending until the desktop host exists and each
behavior is directly observed.

The disposable issue #20 harness now automates the protocol/reducer subset above with deterministic
fixtures. `scripts/desktop-host-prototype.sh` directly observed exact negotiation, `thread/read`
hydration plus a composed snapshot, replay-to-live delivery, stable lifecycle revisions, lazy child
stream attachment, World State replacement, query parity and denial, reload, restart with an expired
cursor, slow-consumer reset, bundle mismatch, redaction, and separate inherited confirmation-pipe
plumbing. This does not change the production Electron/React, Codex `StateRuntime`, native
macOS confirmation and renderer-isolation proof, and provider-backed rows from `pending`; see the
dated issue #20 record.

## Stage 4 — recovery and self-hosting

Interrupt a resident workflow invocation and verify active Attempts are fenced, child cancellation is requested, and the Run durably suspends rather than continuing in the background. Resume it through a later task-native invocation and verify checkpoint reconciliation. Create a conflicting target edit and verify promotion preserves it, retains the shared worktree, and succeeds after the conflict is resolved and the run is resumed. Use an installed version to validate a source-checkout candidate without changing the installed cache; promote only after all required checks and approval pass.

## Stage 5 — retained desktop Automation seam

Open a normal repository task in the retained T3Code desktop, then open **Automation profile** from
the chat header. Use `examples/automation/WORKFLOW.md` or a repository-local equivalent and record
the Product manifest identity used for the check.

1. Select **Validate and preview**. With `LINEAR_API_KEY` absent, confirm the profile remains valid,
   the missing credential is a warning, and live Linear intake reports `skipped` rather than failing
   or exposing raw GraphQL.
2. Select **Start fixture**. Confirm the Root Run belongs to the current task and the claim displays
   a persistent worktree, native Issue task ID, typed Workflow Run ID, and bounded next action.
3. Select **Inspect** and **Refresh**. Confirm checkpoint revision/reconciliation and bounded queue
   projection update without child transcript text appearing in the dialog.
4. Select **Pause**, then **Resume**. Confirm the visible Root Run moves through `suspended` after
   fenced descendant cancellation and returns to `running` only after reconciliation.
5. Select **Cancel issue** on one nonterminal claim. Confirm the request immediately returns a
   claim-only `suspended` cancellation fence and the Root Run remains inspectable. After native
   descendant and effect reconciliation is confirmed, the claim becomes `cancelled` and cleanup
   becomes eligible. If confirmation is unavailable, confirm the bounded retry state remains visible
   and no new work is dispatched for that claim.
6. Start a fresh fixture if needed, then select **Cancel run**. Confirm the Root Run becomes
   `cancelled` without a detached continuation.

Record these as human-verified only after observing the retained Electron/React application. The
automated acceptance and protocol tests do not substitute for this checklist.

Enter a known root Run ID once when recovering older state. Confirm the retained renderer remembers
that task-local ID and automatically reloads the bounded native status projection after a renderer
reload; it must not persist or reconstruct child histories.

Retained evidence for the Symphony-compatible product seam is recorded in
[`verification/2026-07-17-issue-42-symphony-acceptance.md`](verification/2026-07-17-issue-42-symphony-acceptance.md).

## Human-only evidence

Tool rendering, real provider execution, visible V2 activity/residency behavior, interactive approval, cancellation timing, fresh-task recovery, and installed-cache identity remain `pending` until observed.
