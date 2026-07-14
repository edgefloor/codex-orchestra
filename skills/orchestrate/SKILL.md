---
name: orchestrate
description: Route creation, validation, execution, status, cancellation, and recovery through the native Orchestra V2 tools.
---

# Orchestrate a workflow

This skill is a user-facing layer over the Rust extension. It is not the scheduler.

1. Inspect `.codex/orchestra/runs/` for a matching incomplete run.
2. Resume an existing run with `orchestra_resume`; pass an approval decision only when the user explicitly supplied it.
3. Validate an existing `.workflow.ts` with `orchestra_validate`, then run it with `orchestra_run`.
4. Otherwise use `$orchestra:create-workflow`, validate the result, show material mutations and approvals, and run only when authorized.
5. Use `orchestra_status` and `orchestra_cancel` for runtime-owned state and cancellation.

Never emulate missing native tools with SDK threads, MCP, `codex exec`, an App Server client, a daemon, a sidecar, or model-mediated scheduling. If the native tools are absent, explain that the Orchestra-enabled pinned Codex build is required.
