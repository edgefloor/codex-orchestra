---
name: run-workflow
description: Run a validated restricted TypeScript workflow through the native Rust Orchestra extension.
---

# Run a workflow

1. Call `orchestra_validate` with the repository-relative `.workflow.ts` path.
2. Summarize its parallel stages, writers, checks, repeats, and approvals.
3. Call `orchestra_run`. The Rust runtime owns compilation, DAG scheduling, exact context hashing, V2 spawning, retries, checks, worktrees, checkpoints, and summaries.
4. If it pauses, report the exact approval request and run id. Do not invent or pre-record a decision.
5. Read authoritative state with `orchestra_status` and report the promotion status on terminal outcomes.

Do not manually spawn workflow agents or write run state on the model's behalf.
