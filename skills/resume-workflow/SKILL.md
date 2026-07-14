---
name: resume-workflow
description: Resume an interrupted Orchestra workflow from repository state without relying on the previous transcript. Use when a workflow run is incomplete, paused for approval, failed mid-step, or must be recovered after a new Codex task starts.
---

# Resume a workflow

1. Locate the requested run under `.codex/orchestra/runs/`; never infer accepted state from a transcript.
2. Validate the workflow snapshot and confirm its digest matches `state.json`.
3. Compare the recorded source revision, current Git state, step result files, evidence, worktrees, and approvals. Mark stale or missing evidence explicitly.
4. Treat a `running` step without a complete result as interrupted. Retry only when attempts remain and the write scope can be reconciled safely.
5. If the run is waiting for approval, apply only the user's explicit decision and record it before continuing.
6. Recompute dependency-ready steps from completed results; never trust a cached ready list.
7. Route execution through `$codex-orchestra:run-workflow` using the existing run directory.
8. If recovery is unsafe, persist `blocked` with the exact missing decision, evidence, or cleanup action and return that one next action to the user.
