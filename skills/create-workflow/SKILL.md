---
name: create-workflow
description: Author a restricted Orchestra .workflow.ts definition for native compilation and execution.
---

# Create a workflow

1. Start from `assets/templates/WORKFLOW.workflow.ts` and import only from `@codex-orchestra/workflow`.
2. Use literals, arrays, objects, static templates, declared step-output references, and approved calls: `workflow`, `agent`, `parallel`, `pipeline`, `check`, `approval`, `worktree`, and bounded `repeat`.
3. Give every agent an explicit model and, when useful, reasoning effort and service tier. Default `fork_turns` to `none`; full-history forks cannot override inherited model or reasoning.
4. Declare exact context sources, output names, dependencies, attempts, write scopes, and isolated worktrees for concurrent writers.
5. Keep child delegation disabled unless the workflow explicitly requires it and repository configuration permits it.
6. Reject functions, methods, dynamic imports, filesystem/network/process APIs, `eval`, environment access, and side effects.
7. Call `orchestra_validate` and fix all compiler or semantic errors before execution.
