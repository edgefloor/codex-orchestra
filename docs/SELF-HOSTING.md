# Self-hosting vertical slice

Installed Orchestra N runs `evals/workflows/native-vertical-slice.yaml` against the source tree for candidate N+1.

1. A planner proposes one bounded improvement.
2. Two read-only agent steps inspect documentation and tests in parallel.
3. One worker implements the combined result in an isolated worktree.
4. A deterministic check runs in the worker's recorded candidate workspace and revision.
5. An independent reviewer examines that same candidate workspace.
6. A conditional approval pauses for the user when a material finding exists.
7. Every transition is reconstructable from `.codex/orchestra/runs/<run-id>/`.
8. Only an accepted candidate workspace is packaged and installed as N+1; N's cached files must remain byte-identical.

Promotion requires deterministic validation plus a fresh Codex task using the installed candidate. A failed candidate is removed while the known-good version remains available.
