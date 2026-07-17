---
status: accepted
---

# Apply writer ChangeSets transactionally to one Run worktree

A Run owns one shared Git worktree, but agents never mutate it directly. Read-only Steps execute
concurrently against an immutable Stage snapshot; each writer declares a Write scope, acquires its
runtime-owned path and named Write leases before spawning, and receives a private copy-on-write Attempt
overlay restricted to that scope. The writer returns an immutable ChangeSet containing its snapshot
identity, observed read hashes, changed paths and metadata, binary-safe patch, and canonical digest.

The runtime validates every ChangeSet and applies accepted patches under one apply lock in deterministic
Step order. A changed read hash is a stale-input failure and creates a new Attempt against the updated
Run worktree. Checks run only against the aggregate Run worktree after the Stage's ChangeSets have been
applied; Check Steps never share a Stage with writers. Reviewers receive read-only views, and global
formatters, code generation, dependency-lock updates, and other unpredictable write sets require named
exclusive leases.

Agents have no authority to run mutating Git operations, including add, apply, reset, stash, commit, or
Promotion commands; Git mutation belongs exclusively to the runtime. If the host cannot provide an
enforceable Attempt overlay, Orchestra serializes all writers rather than relying on prompts or path
conventions.

## Considered Options

- One Git worktree per writer was rejected because it adds disk and Git overhead while enforcing Write
  scope only after execution and permitting completion-order integration.
- Concurrent writers in one checkout were rejected because prompts and path conventions cannot prevent
  partial visibility, destructive Git interference, or filesystem races.
- Serial execution remains the safe fallback when copy-on-write isolation is unavailable.

## Consequences

- A Stage contains only Steps whose Write leases can coexist; a conflicting writer moves to a later
  Stage and receives a fresh snapshot.
- Disjoint writers may execute concurrently, but read-set validation still detects semantic staleness
  caused by an earlier deterministic apply.
- Run checkpoints and evidence record leases, Stage snapshot identity, ChangeSet identity, validation,
  application order, and stale-input outcomes so recovery never depends on an agent workspace.
