---
status: accepted
---

# Diagnose failures and recover from durable evidence

Retry unchanged work only after diagnosing a transient failure. Stale inputs require a new attempt; semantic failure requires revised instructions. Late results remain evidence but cannot complete reassigned work.

Recovery reconciles the workflow snapshot, digest, source revision, step results, evidence, worktrees, and approvals before selecting the next dependency-ready step. Framework lessons are promoted only when general, testable, versioned, and placed in the narrowest durable source.
