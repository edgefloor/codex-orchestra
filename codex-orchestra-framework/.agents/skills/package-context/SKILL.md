---
name: package-context
description: Context Engineer procedure for compiling a minimal revision-pinned assignment capsule.
---

1. Confirm the task is dependency-ready and already has a Team Leader owner, acceptance criteria, worker archetype/Role Card, Delegation Permit, output schema, and write domain.
2. Resolve binding charter, decisions, plan fragments, interfaces, base revision, and policy version; pin `scope_revision` and `alignment_digest`.
3. State goal, definition of done, deliverables, constraints, non-goals, writable/forbidden paths, dependencies, worktree/base revision, stop conditions, and escalation authority without re-planning.
4. Select the minimum `must_read` refs required before action. Put branch-only material under `read_on_demand` with explicit triggers. Reference rather than paste large content.
5. Add validation commands, evidence requirements, artifact directory, output schema, reversible default, and input/summary budgets.
6. Remove transcript history, obsolete decisions, broad repository tours, duplicated global policy, and speculative implementation advice.
7. Validate against `.orchestra/schemas/context-capsule.schema.json`. Emit a capsule or bounded decision request to the Team Leader.

**Complete when:** a fresh leaf agent can complete exactly the assignment without inventing requirements or reading irrelevant context.
