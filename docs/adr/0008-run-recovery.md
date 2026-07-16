---
status: accepted
---

# Diagnose failures and recover from durable evidence

Orchestra retries unchanged work only after diagnosing a transient failure; stale inputs create a
new Attempt and semantic failures require revised instructions. Recovery reconciles checkpoints,
source revision, outputs, evidence, worktrees, approvals, and late results because durable recorded
state—not transcript continuity—determines what may commit next.
