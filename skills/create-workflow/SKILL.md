---
name: create-workflow
description: Create a reviewable YAML workflow for a multi-step Codex repository task. Use when work needs explicit dependencies, parallel agents, write isolation, checks, review, approvals, repeat limits, reusable inputs, or durable recovery.
---

# Create a workflow

1. Inspect the repository, its instructions, current source revision, and the user's requested outcome before defining steps.
2. Reuse a named file under `.codex/orchestra/workflows/` only when its purpose and inputs match. Otherwise create a new YAML file from the bundled `assets/templates/WORKFLOW.yaml` shape.
3. Use dependency-linked steps: `agent` for one planner, worker, reviewer, or verifier task; `check` for a deterministic command; and `approval` for a user decision that must pause execution. `needs` chains form pipelines; independent ready steps form a parallel stage.
4. Declare every step's inputs, outputs, attempt limit, read/write scope, worktree policy, and completion condition. Use `${steps.<id>.<output>}` only for declared outputs.
5. Bound every repeat with `max_rounds` and a concrete `until` condition. Never express an open-ended loop.
6. Require isolated worktrees for concurrent writers. Prefer one writer when native isolation is unavailable.
7. Add deterministic checks before model review. Add a user approval after review only when a material finding or irreversible decision requires authority.
8. Validate the definition with `uv run --locked python scripts/workflow.py validate <workflow>` when working in the Orchestra source repository. Else validate it against the bundled workflow schema and semantic rules.
9. Show the workflow and call out projected parallelism, mutation, checks, review, approvals, and bounds before execution.
