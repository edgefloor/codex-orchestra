---
status: superseded by ADR-0019
---

# Parallelize reading and isolate writing

Dependency-ready read-only steps may run concurrently. Concurrent writers require disjoint write scopes and isolated worktrees; otherwise they run serially in one checkout. A native subagent is not assumed to own an isolated workspace.

Results record source revision, validated outputs, checks, and residual risk. Independent assurance steps report findings without patching the reviewed source; repairs are separate attempts.
