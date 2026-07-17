---
status: accepted
---

# Promote verified ChangeSets into the target checkout

Writer Attempts produce isolated ChangeSets that the runtime validates and applies transactionally to
the Run worktree under ADR-0019. Promotion applies the aggregate only after its dependencies,
deterministic checks, review, and acceptance succeed, conflict-safely and idempotently without
overwriting target changes. Rejection or conflict preserves the target and durable candidate evidence
because isolated execution never grants permission to mutate the user's checkout.
