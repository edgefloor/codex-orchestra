# Codex Orchestra

A decision-complete framework for long-horizon software delivery with **Codex Multi-Agent V2**, designed around bounded hierarchy, minimal context, explicit authority, and risk-derived verification.

The central separation is:

1. `/root` is the global Conductor for phase, policy, capacity, durable state, and Operator checkpoints.
2. The Manager performs only people management and decisions.
3. Team Leaders own bounded squad branches, first-line review, integration, and the hardest technical work.
4. Repository artifacts preserve durable truth; V2 mailboxes carry compact live coordination.

## Start here

- [`DESIGN.md`](DESIGN.md) — complete architecture and rationale.
- [`docs/FIRST-REVISION.md`](docs/FIRST-REVISION.md) — what changed after the V2 documentation and video analysis.
- [`docs/V2-COLLABORATION.md`](docs/V2-COLLABORATION.md) — exact hierarchy, tool, mailbox, fork, permit, and event-loop contract.
- [`docs/RESEARCH-SYNTHESIS.md`](docs/RESEARCH-SYNTHESIS.md) — source-by-source findings and caveats.
- [`docs/VIDEO-ANALYSIS.md`](docs/VIDEO-ANALYSIS.md) — timestamped orchestration lessons and the second-video evidence boundary.
- [`AGENTS.md`](AGENTS.md) — compact repository operating contract.
- [`.codex/config.toml`](.codex/config.toml) — V2, depth, and thread defaults.
- [`.codex/agents/`](.codex/agents/) — persistent role archetypes and leaf worker/reviewer/advisor/verifier classes.
- [`.agents/skills/`](.agents/skills/) — progressive-disclosure operating procedures.
- [`.orchestra/`](.orchestra/) — charter, plans, policies, permits, schemas, templates, and runtime boundary.
- [`tools/orchestra.py`](tools/orchestra.py) — tested reference state/lease/path control-plane utility.

## Bootstrap

```bash
python tools/orchestra.py doctor
python tools/orchestra.py init
python tools/orchestra.py digest
```

Start a fresh Codex session in the target repository:

```text
$orchestrate Ground this repository for: <operator intent>
```

The first binding outputs are `.orchestra/charter/BRIEF.md`, `SCOPE.md`, and `ASSURANCE.yaml`. Implementation waits for accepted grounding and delivery design.

## Runtime shape

```text
Operator
  ↕
/root Conductor — global phase, policy, state, capacity, Operator checkpoints
  ├─ Consultant
  │   └─ optional read-only Explorer/Advisor branch
  ├─ Delivery Architect
  │   └─ optional read-only Explorer/Advisor branch
  ├─ Manager — people and decisions only
  │   ├─ Role Architect
  │   ├─ Team Leader: platform
  │   │   ├─ Context Engineer
  │   │   ├─ Workers
  │   │   └─ Reviewers
  │   └─ Team Leader: product
  │       ├─ Context Engineer
  │       └─ Workers/Reviewers
  └─ Quality Governor
      └─ Verifiers/Advisors
```

The hierarchy is deliberately capped at depth 3. The parent that creates a child owns its wait, join, first-line review, and bounded upward report. Workers and other leaf roles cannot spawn.

## Non-negotiable invariants

- The Manager never plans tasks, writes prompts/capsules, reviews diffs, runs tests, or talks to workers.
- Team Leaders define hire requirements, plan one wave at a time, form approved squads, and own local integration.
- Nested delegation requires both role authority and a current Delegation Permit.
- `/root` receives compact branch/checkpoint reports, not raw worker completion traffic.
- V2 mailboxes are the live bus; repository artifacts and the event store are durable truth.
- `send_message` is passive; `followup_task` wakes; `wait_agent` is the normal parent loop.
- Spawns default to `fork_turns: none`; Context Capsules carry minimal revision-pinned context.
- Write work uses leases, declared domains, isolated worktrees, and serialized integration.
- Reversible ambiguity receives a default; only concrete authority/dependency/contradiction/safety/irreversibility blocks affected work.
- Assurance derives from risk dimensions rather than a maturity label alone.
- Every fan-out has a named join owner, child/concurrency/attempt budget, review capacity, and checkpoint.
- `max` is expensive single-agent reasoning; `Ultra` is multi-agent orchestration topology and is prohibited by default.

## Deployment levels

### Repository-native supervised pilot

The current root Codex session follows `$orchestrate`; authorized parents run bounded V2 branches; deterministic mutations go through `tools/orchestra.py`. Start with one Team Leader and no more than two concurrent writers.

### External Conductor

For unattended, multi-day, or high-stakes operation, implement the same contracts around Codex App Server: transactional events, permits, thread/path reconciliation, leases, worktrees, approvals, secrets, cost/rate budgets, and audit export.

## Scope

This revision includes the complete operating design, role TOMLs, skills, schemas, permit/branch-report contracts, policy files, a V2 collaboration guide, research analysis, and a tested reference control plane. The Python utility demonstrates invariants; it is not represented as a production scheduler.
