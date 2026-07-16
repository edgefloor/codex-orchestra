---
status: accepted
---

# Maintain Codex as a long-lived pinned product fork

Orchestra maintains a revision-pinned Codex product fork because stock plugins cannot register the
required Rust extension and Orchestra must share native task, permission, tool, and V2-agent
lifecycle rather than integrate through an external service. The fork extends Codex-owned seams for
native identity and protocol while Orchestra core retains workflow plans, scheduling, gates,
evidence, effects, and recovery; upstream changes are selectively adopted, but eliminating the fork
is not an architectural objective.
