---
status: accepted
---

# Compile restricted TypeScript into a native V2 Rust runtime

Workflow source is an authoring-only restricted TypeScript shape compiled into a Rust execution plan;
Rust owns validation, scheduling, checkpoints, recovery, and evidence rather than executing arbitrary
JavaScript or asking a model to simulate the scheduler. Agent work uses the owning task's native V2
control plane, and the task-bound invocation remains resident until terminal completion or durable
suspension so Orchestra never becomes detached background execution.
