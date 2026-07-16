---
status: superseded by ADR-0010
---

# Execute declarative workflows through the model

This historical decision used model-owned scheduling over declarative YAML workflows because no
native executor existed. ADR 0010 replaced the model, YAML, and Python execution path with Rust while
retaining declarative authoring, bounded repeats, dependency-linked parallelism, deterministic
checks, explicit approvals, and repository snapshots.
