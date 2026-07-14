---
status: accepted
---

# Parallelize reading and isolate writing

Dependency-ready read-only steps may run concurrently. Concurrent writers require disjoint write scopes and isolated worktrees; otherwise they run serially in one checkout. A native subagent is not assumed to own an isolated workspace.

Results record source revision, changed files, checks, and residual risk. Reviewers report findings without patching the reviewed source; repairs are separate worker attempts.
