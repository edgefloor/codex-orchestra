---
status: accepted
---

# Compile workflow source through one pinned hermetic contract

The target authoring surface pins one inert Agents-compatible facade that Rust parses and lowers from
a closed module graph without executing workflow source as JavaScript. Exact synchronous Zod behavior
runs only from the schema-only Validation bundle in a pinned one-request worker; Runs consume the
canonical plan, JSON Schema remains guidance, and evaluator caches are disposable. The MVP treats
repository workflow definitions like other local coding-harness inputs and proves deterministic,
bounded failure behavior rather than making the worker a hostile-code security boundary.
