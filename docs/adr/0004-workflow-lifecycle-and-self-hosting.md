---
status: superseded-by-0009
---

# Preserve checkpoints and version-to-version self-hosting

Runs persist state after each step transition and user approval. A failed or interrupted run resumes from its workflow snapshot, results, evidence, and Git state.

Orchestra develops itself by having installed version N run a workflow against candidate N+1. The active installed cache is never modified. The candidate is validated and installed separately while N remains available for recovery.

ADR 0009 replaces this decision's earlier assumption that no first-class generated workflow should exist.
