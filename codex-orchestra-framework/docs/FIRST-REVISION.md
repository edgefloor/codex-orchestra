# First Revision — V2 and Video-Driven Changes

## Executive decision

The initial design correctly separated Manager judgment from orchestration mechanics, but it over-corrected into a flat V2 tree. In V2, terminal notifications go to the physical parent. Making every worker a direct `/root` child therefore pushes worker lifecycle and result traffic into the Operator-facing context.

Revision 1 changes the runtime to a **bounded hierarchy with delegated branch controllers**:

- `/root` controls phase, policy, global capacity, durable state, and Operator checkpoints;
- Manager physically parents Team Leaders and its Role Architect, but no technical leaf agents;
- Team Leaders physically parent Context Engineers, approved workers, reviewers, and bounded technical advisors;
- Delivery Architect/Consultant may parent bounded read-only evidence agents;
- Quality Governor parents verifiers;
- all leaf roles are non-spawning;
- depth is capped at 3 and every nested spawn requires a Delegation Permit.

This both matches the intended organization and reduces context movement.

## Changes from the prior repository

### Runtime topology

**Before:** `max_depth = 1`; root spawned every seat/attempt; worker completion was relayed through root.

**Now:** `max_depth = 3`; the closest competent parent owns child lifecycle/join; root receives bounded branch reports.

### Manager boundary

The Manager may now use V2 tools, but only as people management:

- spawn/wake/replace/release Team Leaders;
- spawn/wake the Role Architect;
- wait for Team Leader portfolio reports;
- interrupt a Team Leader for cancellation, stale scope, policy, or runaway behavior.

It cannot spawn/message workers or perform technical dispatch.

### Team Leader authority

The Team Leader now performs the hiring sequence implied by the Operator's intent:

1. defines technical requirements for the missing hire;
2. asks the Manager for the organizational commitment/capacity;
3. receives an approved Role Card and Delegation Permit;
4. uses a Context Engineer for the exact assignment;
5. spawns the approved worker as its child;
6. reviews and integrates the result locally.

### Explicit delegation control

A new `delegation-permit.schema.json` makes nested authority and budget machine-checkable. It binds the parent path, allowed child types, purpose refs, scope revision/digest, children/concurrency/attempts, write policy, expiry, and stop conditions.

A new `branch-report.schema.json` limits upward communication to one bounded outcome, at most five material findings, evidence refs, capacity/decision consequences, residual risks, and next action.

### Independent review

A `reviewer_terra` archetype and `$review-assignment` skill implement a narrow, independent review handoff. Results require either explicit `no_findings: true` with target/checks, or at most five actionable findings. Reviewers never patch their own findings.

### Reasoning policy

The policy now distinguishes:

- `max`: extended reasoning by one agent;
- `Ultra`: a child-spawning workflow topology.

Ultra is never embedded in leaf archetypes. It requires exceptional recorded authorization and explicit child, concurrency, token/time, isolation, and stop budgets.

### Persistent/generated roles

Persistent identity is formalized as TOML + canonical path + durable roster/memory + eval, not transcript length. Generated Role Cards overlay a stable archetype. Promotion to a new persistent TOML requires recurrence/continuity and a passing eval.

## Video-derived operating rules

### From “A proper guide to Fable 5”

- Route by capability, interface/taste judgment, and cost rather than model prestige.
- Strong orchestration can direct cheaper specialized execution, but outputs must be inspected and escalated when the quality bar is missed.
- Bounded fan-out with explicit investigators/reviewers/judges is useful for independent items.
- A multi-day implementation program should not be one giant generated workflow; it needs worktrees and explicit CI/review/merge/product checkpoints.
- Custom skills should improve from observed failures, but lessons belong in the narrowest durable policy/eval rather than an ever-growing global prompt.

### From “I can't believe they released this”

The full caption transcript was not retrievable through the available surfaces at the time of this revision. The verified publisher description says Ultra can repeatedly spawn maximum-reasoning subagents and consume allowance rapidly. Official Codex documentation independently describes Max as extended single-task reasoning and Ultra as subagent delegation. The framework therefore treats Ultra as visible orchestration, not a harmless slider.

For the full timestamped synthesis and evidence limits, see [`VIDEO-ANALYSIS.md`](VIDEO-ANALYSIS.md).

## Why this is not uncontrolled recursion

The hierarchy is finite and role-restricted:

- raw depth cap: 3;
- role-to-child allowlist;
- permit-bound child/concurrency/attempt limits;
- global reserved control/quality slots;
- write-domain/worktree constraints;
- leaves cannot spawn;
- parent-local wait/join;
- compact upward reports;
- explicit stop/expiry/revision invalidation.

The revision uses V2 hierarchy to compress context while retaining a deterministic policy envelope.

## Validation status — 2026-07-14

This revision was checked with:

- 18 unit/contract tests covering hierarchy, Manager isolation, permit enforcement, schema/template validity, V2 fork semantics, leases, write-domain isolation, stale results, review, reconciliation, phase transitions, and idempotency;
- `python tools/orchestra.py doctor` with zero errors and zero warnings;
- an internal Markdown link check with 13 relative links resolved;
- a disposable control-plane smoke run through `init`, `status`, `digest`, and `phase get`.

These checks validate the repository contracts and reference control plane. They do not substitute for the live Codex V2 capability probe documented in [`CODEX-CONFIGURATION.md`](CODEX-CONFIGURATION.md).
