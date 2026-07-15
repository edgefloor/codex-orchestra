# Interactive verification

## Stage 0 — automated baseline

Run the Rust workspace suite, lifecycle doctor, plugin validator, and pinned Codex integration build. Record exact revisions and exit status.

## Stage 1 — fresh plugin discovery

Install the packaged candidate, start a task outside the source repository, invoke `$orchestra:orchestrate`, and confirm it requires the native tool surface rather than emulating a scheduler.

## Stage 2 — native tool surface

Using the Orchestra-enabled Codex build, confirm `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, and `orchestra_cancel` appear as native tools.

## Stage 3 — V2 vertical slice

Invoke `evals/workflows/native-vertical-slice.workflow.ts` through the active parent task's native Orchestra skill/tool path. Confirm the Root Run is owned by that task and no renderer RPC, linked normal thread, or external scheduler creates it. Observe canonical child task paths, explicit model/reasoning, no parent transcript for `fork_turns: none`, parallel activity, V2 completion, isolated changes present in the shared worktree before the sandboxed check, approval pause, accepted promotion into the target checkout as unstaged changes, isolated and terminal shared-worktree cleanup, and final summary.

For the accepted desktop target, also verify exact-bundle initialization without task cursors,
per-task snapshot and cursor subscription, lazy child-task subscription on expansion, typed
`ThreadItem::Orchestra` hydration, World State replacement of the Run Digest, deterministic digest
truncation with stable omission references, and identical bounded query selection through native
task-tool and App Server adapters. Keep these checks pending until the desktop host exists and each
behavior is directly observed.

## Stage 4 — recovery and self-hosting

Interrupt a resident workflow invocation and verify active Attempts are fenced, child cancellation is requested, and the Run durably suspends rather than continuing in the background. Resume it through a later task-native invocation and verify checkpoint reconciliation. Create a conflicting target edit and verify promotion preserves it, retains the shared worktree, and succeeds after the conflict is resolved and the run is resumed. Use an installed version to validate a source-checkout candidate without changing the installed cache; promote only after all required checks and approval pass.

## Human-only evidence

Tool rendering, real provider execution, visible V2 activity/residency behavior, interactive approval, cancellation timing, fresh-task recovery, and installed-cache identity remain `pending` until observed.
