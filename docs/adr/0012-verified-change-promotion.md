---
status: accepted
---

# Promote verified isolated changes into the target checkout

Isolated agent changes remain patches until dependent deterministic checks and approvals complete in the shared run worktree. The first declared approval choice continues an accepted run; any other declared choice rejects it. Approval decisions must match a unique, nonempty declared choice.

After acceptance, the runtime writes the aggregate staged shared-worktree diff to `evidence/changes/promoted.patch`, checks it against the target checkout, and applies it without staging target files. The checkpoint records whether promotion was applied, pending, or unnecessary; rejection records it as unnecessary.

Promotion never overwrites a conflicting target change. A conflict fails the run, preserves the target checkout, and retains the shared worktree and durable patch so a later resume can retry. Successful promotion is idempotent across interruption, after which terminal cleanup removes the shared worktree.
