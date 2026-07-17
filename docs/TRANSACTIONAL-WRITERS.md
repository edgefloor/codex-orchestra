# Transactional writers

Orchestra will use one runtime-owned Run worktree without allowing agents to mutate it directly.
Read-only Steps run concurrently against an immutable Stage snapshot. Writers declare a Write scope,
acquire path or named Write leases, and work in private copy-on-write Attempt overlays.

Each writer returns an immutable ChangeSet containing its snapshot identity, observed read hashes,
changed paths and metadata, binary-safe patch, and digest. The runtime validates these ChangeSets and
applies them under one lock in deterministic Step order. A changed read hash makes the ChangeSet stale
and starts a new Attempt against the updated Run worktree.

```text
Stage snapshot
   ├─ reader A ───────────── findings
   ├─ writer B ─ COW overlay ─ ChangeSet
   └─ writer C ─ COW overlay ─ ChangeSet
                                  │
                         validate leases, scope,
                         read hashes, and patch
                                  │
                    runtime-owned deterministic apply
                                  ▼
                       shared Run worktree
```

This provides four guarantees:

- Agents never see another agent's partial edits.
- Write scope is enforced by isolation, not prompts.
- Mutating Git operations belong exclusively to the runtime.
- Conflicts become explicit stale-input failures rather than filesystem races.

Checks run only against the aggregate Run worktree after a Stage is applied. Reviewers are read-only,
while formatters, code generation, lockfile updates, and other unpredictable write sets require named
exclusive leases. If enforceable overlays are unavailable, Orchestra serializes all writers.

The current per-Step Git-worktree mechanism remains a transitional safety adapter. The authoritative
decision is [ADR-0019](adr/0019-transactional-writer-changesets.md), with canonical terminology in
[`CONTEXT.md`](../CONTEXT.md).
