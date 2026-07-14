# Interactive verification

## Stage 0 — automated baseline

Run the Rust workspace suite, lifecycle doctor, plugin validator, and pinned Codex integration build. Record exact revisions and exit status.

## Stage 1 — fresh plugin discovery

Install the packaged candidate, start a task outside the source repository, invoke `$codex-orchestra:orchestrate`, and confirm it requires the native tool surface rather than emulating a scheduler.

## Stage 2 — native tool surface

Using the Orchestra-enabled Codex build, confirm `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, and `orchestra_cancel` appear as native tools.

## Stage 3 — V2 vertical slice

Run `evals/workflows/native-vertical-slice.workflow.ts`. Observe canonical child task paths, explicit model/reasoning, no parent transcript for `fork_turns: none`, parallel activity, V2 completion, isolated changes present in the shared worktree before the sandboxed check, approval pause, isolated and terminal shared-worktree cleanup, and final summary.

## Stage 4 — recovery and self-hosting

Interrupt a run, resume it from a fresh task, and verify checkpoint reconciliation. Use an installed version to validate a source-checkout candidate without changing the installed cache; promote only after all required checks and approval pass.

## Human-only evidence

Tool rendering, real provider execution, visible V2 activity/residency behavior, interactive approval, cancellation timing, fresh-task recovery, and installed-cache identity remain `pending` until observed.
