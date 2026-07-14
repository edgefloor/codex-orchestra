---
status: accepted
---

# Bound delegation and pass task inputs instead of transcripts

Every agent step declares instructions, resolved inputs, source revision, read/write scope, completion condition, outputs, attempt limit, and result path. Fresh context is the default, and native spawns request `fork_turns: none` when supported.

The parent that spawns an agent waits for it and records its structured result. Agent completion is only a transport signal; declared outputs and evidence determine step completion.

## Consequences

- Agent steps do not spawn children unless explicitly enabled.
- Raw transcripts and broad repository dumps are not delegated.
- Attempt and parallelism limits reserve capacity for review and verification.
