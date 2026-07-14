---
name: orchestrate
description: Create, run, or resume a reviewable Codex-native workflow. Use when a user asks Orchestra to coordinate a multi-step repository task, delegate bounded work, recover an interrupted run, or apply review and verification stages.
---

# Orchestrate a workflow

Act as the user-facing router. Plugin files are read-only; all workflow definitions and run state belong in the target repository under `.codex/orchestra/`.

1. Inspect `.codex/orchestra/runs/` for an incomplete run that matches the request.
2. If one exists, route to `$codex-orchestra:resume-workflow`.
3. If the user supplied a workflow or named a reusable workflow, route to `$codex-orchestra:run-workflow`.
4. Otherwise route to `$codex-orchestra:create-workflow`, show the generated workflow, and obtain any required approval before routing to `$codex-orchestra:run-workflow`.

Use only native Codex collaboration and ordinary repository tools. Do not start an MCP server, App Server client, daemon, sidecar, or external scheduler. The active Codex agent decides which dependency-ready steps execute next.

Complete when the workflow is closed with a run summary or paused with an exact user decision or recovery action.
