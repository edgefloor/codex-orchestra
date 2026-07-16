---
tracker:
  kind: linear
  project_slug: orchestra
  api_key: $LINEAR_API_KEY
  required_labels:
    - automation
  active_states:
    - Todo
    - In Progress
  terminal_states:
    - Done
    - Cancelled
workspace:
  root: ../../.codex/orchestra/automation-worktrees
agent:
  max_concurrent_agents: 1
orchestra:
  workflow: ../../crates/orchestra-core/fixtures/automation-issue.workflow.ts
  effects:
    - tracker.comment
---

Implement {{ issue.identifier }}: {{ issue.title }}

Work only on the claimed issue. Keep the final response concise and include the verification you ran.
