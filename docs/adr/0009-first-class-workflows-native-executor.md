---
status: accepted
---

# Use first-class workflows with a native Codex executor

Orchestra workflows are declarative YAML validated by JSON Schema and semantic checks. Dependency chains form pipelines; independent ready steps form parallel stages. Step types cover agent tasks, deterministic checks, and user approvals. Repeats are bounded by an explicit condition and maximum round count.

Version 1 is model-executed: the active Codex agent reads the workflow, selects ready steps, invokes native tools, and records transitions. This is inspired by Anthropic Dynamic Workflows but is not runtime-compatible and does not claim background execution, script-variable isolation, or equivalent scale.

## Consequences

- Reusable definitions live in `.codex/orchestra/workflows/`.
- Each run snapshots its workflow under `.codex/orchestra/runs/<run-id>/`.
- YAML is never executed as arbitrary code.
- The schema is backend-neutral for a future first-party Codex workflow runtime.
