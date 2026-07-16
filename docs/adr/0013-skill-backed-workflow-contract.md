---
status: accepted
---

# Resolve and snapshot skills before native workflow execution

Workflows declare exact skill requirements, typed inputs, and external-effect authority; Rust
resolves the complete skill closure through native Codex capabilities and snapshots its instructions,
resources, inputs, revision, and digests before child execution, so recovery uses that snapshot
rather than mutable installations or prompt discovery. Human input is data, not acceptance, and
external mutations require declared scope and durable reconciliation receipts.
