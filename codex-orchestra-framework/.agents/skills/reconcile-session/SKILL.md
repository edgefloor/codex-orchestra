---
name: reconcile-session
description: Recover a V2 Orchestra session after restart, user steering, timeout, or uncertain child state.
---

1. Load the runtime event cursor, active permits, leases, worktrees, canonical agent paths, and current scope revision/digest.
2. Call `list_agents` once and compare the live tree to the roster. Do not repeatedly poll it.
3. Reconcile each branch from the leaves upward: terminal child, running valid child, stale/invalid child, missing child, or orphaned durable attempt.
4. Accept no result until schema, permit, lease, digest, write boundary, and evidence checks pass. Preserve late results as evidence but never let them transition reassigned work.
5. Interrupt stale, cancelled, runaway, policy-violating, or write-conflicting attempts. Expire orphaned leases and preserve useful worktrees/artifacts.
6. Reuse a resident path with `followup_task` only when its role/config/security boundary and durable handoff remain valid. Otherwise create a fresh attempt with `fork_turns: "none"`.
7. Emit one reconciliation report: recovered paths, interrupted attempts, reissued work, unresolved decisions, and the next safe event-loop action.

**Complete when:** the live V2 tree, durable roster, permits, leases, and worktrees agree—or every disagreement has an explicit safe disposition.
