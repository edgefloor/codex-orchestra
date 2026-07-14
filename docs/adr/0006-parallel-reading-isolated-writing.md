---
status: accepted
---

# Parallelize information gathering and isolate mutation

Read-only exploration, review, test analysis, and synthesis may fan out when their questions are independent. Concurrent writers are allowed only with demonstrably disjoint write domains and native workspace isolation; otherwise Orchestra uses one writer in the shared checkout. A native subagent is not assumed to have an isolated worktree merely because it is a separate thread.

Every Workstream names one integration owner and a serial integration order. Results identify their base revision, changed files, validation, and residual risk. A Reviewer reports findings without patching the reviewed implementation; repair is a separate Attempt with a new or revised Context Capsule.

## Consequences

- Native worktree capability is verified before parallel mutation.
- Review and verification capacity cannot be consumed entirely by implementation fan-out.
- Write conflicts are prevented through assignment design, not repaired through optimistic merging alone.
