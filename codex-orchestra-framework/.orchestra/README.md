# `.orchestra` contract

This directory separates durable organizational truth from transient V2 execution.

## Version controlled

- `charter/` — Operator/Consultant grounding bundle.
- `plan/` — Delivery Architect milestones and workstreams.
- `decisions/` — accepted decisions and waivers.
- `roster/` — stable archetypes and lifecycle rules.
- `policies/` — collaboration, routing, assurance, blocker, retention, and concurrency rules.
- `schemas/` — typed envelopes, permits, capsules, results, and reports.
- `templates/` — authoring examples, not runtime state.

## Runtime only

- `runtime/orchestra.db` — transactional project/event/path/lease projection.
- `runtime/artifacts/` — attempt and evidence artifacts.
- `worktrees/` — isolated write workspaces.
- `logs/` — raw runtime protocol logs.

Root/external Conductor is the sole committer of global runtime state. Authorized parent agents may call V2 collaboration tools inside Delegation Permits, but report commands/receipts upward; they do not independently redefine global state.

Increment `scope_revision` whenever accepted product behavior, scope, or assurance changes. Recompute the alignment digest and invalidate only affected permits/capsules/tasks.

Key contracts:

- `policies/collaboration-v2.yaml` — role-to-child authority, lifecycle, fork, message, and fan-out policy;
- `schemas/delegation-permit.schema.json` — nested authority/capacity boundary;
- `schemas/context-capsule.schema.json` — minimal assignment context;
- `schemas/result-envelope.schema.json` — bounded leaf result with explicit no-findings;
- `schemas/branch-report.schema.json` — compressed parent-to-parent outcome;
- `schemas/collaboration-command.schema.json` — auditable command/receipt.
