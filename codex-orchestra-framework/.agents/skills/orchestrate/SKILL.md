---
name: orchestrate
description: Run the global Codex Orchestra loop with bounded hierarchical Codex Multi-Agent V2 branches.
---

# Global loop

The root is the Operator-facing Conductor. It owns global phase/state, portfolio capacity, policy validation, and Operator checkpoints. It does not impersonate the Manager or technical owners.

1. Run `python tools/orchestra.py doctor` and `digest`. After restart/uncertainty, use `$reconcile-session` before dispatch.
2. Determine the earliest unsatisfied gate in `docs/STATE-MACHINE.md`. Read only the current projection and the narrow artifact for that gate.
3. Spawn/reuse the relevant direct phase owner—Consultant, Delivery Architect, Manager, or Quality Governor—with `fork_turns: "none"` by default and a compact message pointing to canonical refs/output schema.
4. Before nested delegation, record a Delegation Permit that constrains parent path, allowed child archetypes, task/wave/gate refs, scope revision/digest, child/concurrency/write/token-time budgets, expiry, and stop condition.
5. Let the closest competent parent run its bounded branch: Consultant/Architect for read-only evidence, Manager for Team Leaders/Role Architect, Team Leader for its squad, Quality Governor for verifiers. Root must not relay every worker message.
6. Use `wait_agent` as the root event loop. Mailbox receipts and direct-child completions end the wait; do not poll. User input is steering: classify impact before interrupting work.
7. Validate each upward envelope, authority, revision/digest, permit, artifacts, and gate evidence before committing state. Branch reports are compact; detailed child output remains referenced.
8. Apply only deterministic consequences already authorized by role/Operator decisions: phase transition, capacity reservation, lease/worktree state, stale invalidation, or dispatch.
9. On failure, classify transient, stale context, semantic/task, planning, grounding, authority/access, or safety/policy. Blind retry only transient failures.
10. Stop for the Operator only at material scope, accepted risk, budget/capacity ceiling, irreversibility, milestone acceptance, or final acceptance. Present one decision, options, recommendation, evidence, and reversible default.

**Complete when:** the next phase/checkpoint is validly passed or the Operator has one decision-ready brief. Load `docs/V2-COLLABORATION.md` for runtime semantics and `DESIGN.md` only when a rule remains unclear.
