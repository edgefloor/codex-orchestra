# Codex Orchestra — V2 First-Revision Design

## 1. Purpose

Codex Orchestra is a governed multi-agent operating system for long-horizon software delivery. It is designed for the failure modes that appear after the first successful demo: scope drift, context accumulation, accidental authority expansion, invisible fan-out, conflicting writes, weak joins, fake blockers, and verification that does not match actual risk.

The framework is not a permanent swarm and not a single super-agent. It is a small stable organization that creates temporary, bounded teams.

The design has four layers:

1. **Operator** — supplies intent, accepts material scope/risk/budget decisions, and accepts milestones/final delivery.
2. **Decision organization** — Consultant, Delivery Architect, Manager, Team Leaders, Context Engineers, Role Architect, and Quality Governor exercise narrow judgment.
3. **V2 collaboration tree** — parent agents spawn, wait for, join, and summarize only their authorized children.
4. **Durable control record** — charter, plans, decisions, permits, capsules, evidence, events, worktrees, and bounded handoffs survive beyond any transcript.

The root Codex session is the global Conductor. It owns project phase, global capacity, policy validation, Operator dialogue, and canonical state transitions. It does **not** plan the implementation or manage individual workers.

## 2. Core design choices

### 2.1 Use the V2 parent-child tree as a context boundary

A flat tree appears safe because only `/root` can spawn. It also makes `/root` the physical parent of every worker, so all completion notifications and task results converge on the Operator-facing context. That creates exactly the long-horizon context pollution the framework is intended to prevent.

The revision uses a bounded hierarchy:

```text
/root                                      depth 0
├── consultant                              depth 1
│   ├── explore_domain                      depth 2, optional
│   └── advise_risk                         depth 2, optional
├── delivery_architect                      depth 1
│   └── explore_interface                   depth 2, optional
├── manager                                 depth 1
│   ├── role_architect                      depth 2, service
│   ├── tl_platform                         depth 2
│   │   ├── context                         depth 3, service
│   │   ├── w_api_a1                        depth 3, leaf
│   │   └── review_api_a1                   depth 3, leaf
│   └── tl_product                          depth 2
│       ├── context                         depth 3, service
│       └── w_ui_a1                         depth 3, leaf
└── quality_governor                        depth 1
    ├── verify_security                     depth 2, leaf
    └── advise_residual_risk                depth 2, leaf
```

`agents.max_depth = 3` permits this shape and no deeper. Leaf roles have no spawn authority. The physical parent is the closest role competent to review and join the child's work:

- grounding evidence joins at the Consultant;
- planning evidence joins at the Delivery Architect;
- Team Leader portfolio reports join at the Manager;
- worker/reviewer results join at the Team Leader;
- verifier evidence joins at the Quality Governor.

Only compact branch reports move upward. Raw worker transcripts do not.

### 2.2 Separate global control from branch-local orchestration

The global Conductor owns:

- phase and milestone state;
- scope revision and alignment digest;
- global thread/capacity ceilings;
- policy/schema/authority validation;
- Operator checkpoints;
- recovery reconciliation;
- applying already-authorized state transitions.

Selected parent agents own only their local branch:

- Consultant: bounded read-only grounding research;
- Delivery Architect: bounded read-only delivery research;
- Manager: Team Leader appointments and Role Architect service;
- Team Leader: Context Engineer, workers, reviewers, and local advisors;
- Quality Governor: verifiers and independent risk advisors.

A parent may call V2 collaboration tools only when both conditions hold:

1. its role is allowed to parent that child archetype; and
2. a current **Delegation Permit** authorizes the purpose, revision, child types, concurrency, attempts, writes, expiry, and stop condition.

This makes nested delegation visible without turning `/root` into a prompt relay.

### 2.3 The Manager is a person manager, not a workflow engine

A valid Manager action changes at least one of:

- accountable owner;
- reporting line;
- Team Leader appointment/status;
- capacity allocation;
- priority;
- escalation path;
- decision owner/deadline;
- continue/pause/re-plan/re-ground recommendation;
- checkpoint recommendation.

The Manager may spawn/wake Team Leaders and its Role Architect because those are people-management acts. It may wait for Team Leader branch reports. It may interrupt a Team Leader only for cancellation, stale scope, policy violation, or runaway behavior.

The Manager may not:

- decompose implementation work;
- author worker instructions or Context Capsules;
- choose architecture or implementation details;
- inspect worker output, diffs, or test logs;
- spawn/message workers, reviewers, verifiers, explorers, or Context Engineers;
- poll the V2 tree;
- approve its own technical answer.

When asked to do technical work, it emits a delegation decision naming the proper owner and decision constraints.

### 2.4 The Team Leader is the squad controller and central implementer

The Team Leader is approximately 50% leadership and 50% execution. The ratio is a guardrail, not time accounting:

- it must not become a pure status relay while short-lived agents hold all technical context;
- it must not disappear into coding while assignments, reviews, integration, and hiring go unmanaged;
- it keeps the hardest, highest-context, or integration-critical task when delegation would cost more context than it saves.

For one bounded wave, the Team Leader:

1. creates the tactical plan and workflow shape;
2. defines technical requirements for any new hire;
3. receives Manager approval and a Delegation Permit;
4. uses its Context Engineer to compile task capsules;
5. spawns approved leaf workers/reviewers;
6. waits for and reviews their results;
7. integrates through one workstream queue;
8. reports only organizational consequences upward.

### 2.5 Persistent personality is a contract, not an immortal chat

A persistent role consists of:

- a version-controlled `.codex/agents/*.toml` authority/personality contract;
- a canonical V2 path;
- a durable roster entry and role memory;
- a reusable resident thread when its configuration and scope remain valid;
- evaluation tests for boundary compliance.

The transcript is not authoritative memory. An idle agent may unload and later restore. A fresh thread is preferred after material role, security, or scope changes.

Assignment-specific expertise uses a generated **Role Card** over a stable archetype. The Role Card may specify operational traits, domain capabilities, exclusions, model/effort, tools, lifetime, deliverables, and independence. It cannot broaden authority. Promote a repeated generated role into a persistent TOML only after at least three waves or cross-milestone continuity, a passing eval, and Manager/Operator approval.

## 3. Role architecture

| Role | Lifetime | Owns | May parent | Explicitly does not own |
|---|---|---|---|---|
| Operator | human | intent, material scope/risk/budget, milestone/final acceptance | none | routine coordination |
| Root Conductor | session/service | global state, policy, phase, capacity, Operator interface | phase owners | product/technical judgment |
| Consultant | project resident | brief, scope, behavior, assumptions, assurance framing | Explorer, Advisor | delivery plan, staffing, code |
| Delivery Architect | phase resident | milestones, workstreams, dependencies, interfaces, integration/evidence plan | Explorer, Advisor | staffing, task-level management |
| Manager | project resident | Team Leaders, staffing, capacity, priority, responsibility, checkpoints | Team Leader, Role Architect | technical planning/execution |
| Role Architect | Manager service | approved Role Cards | none | hiring approval, task scope |
| Team Leader | workstream resident | tactical waves, squad, local decisions, hardest work, review, integration | Context Engineer, Explorer, Advisor, Worker, Reviewer | product scope, own hiring approval |
| Context Engineer | TL service | minimal revision-pinned capsules | none | task meaning, planning, worker selection |
| Quality Governor | project resident | risk-derived gates, verifier branch, verdict | Verifier, Advisor | implementation repairs |
| Explorer | task scoped | bounded repository evidence | none | decisions, edits |
| Advisor | task scoped | independent high-judgment opinion | none | ownership, edits |
| Worker | task scoped | bounded implementation | none | scope, integration, self-approval |
| Reviewer | task scoped | independent target review | none | patching findings |
| Verifier | task scoped | executable evidence for a gate | none | source modification |

## 4. The whole loop

```text
NEW
  ↓
GROUNDING
  Operator + Consultant → BRIEF / SCOPE / ASSURANCE / acceptance behavior
  ↓ accepted grounding revision
DELIVERY_DESIGN
  Delivery Architect → milestones / workstreams / dependencies / interfaces / evidence
  ↓ accepted delivery plan
STAFFING
  Manager → appoint Team Leaders / capacity / initial permits
  ↓ accountable workstreams
WAVE_READY
  Team Leader → tactical plan → approved hires → Role Cards → Context Capsules
  ↓ dependency-ready and isolated assignments
EXECUTION
  Team Leader branch → workers/reviewers → local join and review
  ↓ accepted workstream result
INTEGRATION
  Team Leader → serialized local integration; named cross-workstream integration owner
  ↓ integrated milestone candidate
ASSURANCE
  Quality Governor branch → risk-derived verifier evidence → gate report
  ↓ pass or authorized conditional pass
MILESTONE_CHECKPOINT
  Manager → portfolio/ownership/capacity recommendation
  Root → Operator decision only when required
  ↙ next wave/milestone                 ↘ final milestone
WAVE_READY                              DELIVERY
                                          ↓
                                    OPERATOR_ACCEPTANCE
                                          ↓
                                        CLOSED
```

The project loop is checkpoint-driven. A generated workflow is a local accelerator inside one wave; it is never the entire multi-day program. CI, reviews, merge order, rebases, product decisions, and assurance remain explicit gates.

### 4.1 Grounding gate

The Consultant and Operator establish:

- users and desired outcomes;
- externally observable behavior and examples;
- in-scope boundaries and non-goals;
- constraints and external obligations;
- assumptions/defaults/invalidation conditions;
- assurance dimensions;
- unresolved material decisions.

Questions are reserved for decisions that change behavior, scope, risk, acceptance, budget, or irreversibility. Reversible ambiguity receives a documented default so work can proceed.

### 4.2 Delivery design gate

The Delivery Architect converts the accepted grounding revision into:

- milestone behavior slices;
- workstream ownership seams;
- dependency and interface graph;
- risk-reduction spikes;
- integration order and owner;
- evidence expectations per milestone.

It does not write the full implementation backlog. Team Leaders plan the next bounded wave when dependencies and evidence are current.

### 4.3 Staffing gate

The Manager appoints Team Leaders and assigns capacity. A Team Leader hire request must contain the specific capability gap, required outcomes, why current staff cannot absorb it, expected lifetime, access/independence needs, cost of delay, and budget requested.

The Manager decides whether the organization should fill the gap. The Role Architect compiles the approved Role Card. The Manager issues/revises the Team Leader's Delegation Permit. The Team Leader performs the actual worker spawn, preserving physical parenthood and local review.

### 4.4 Wave readiness gate

Every delegated assignment requires:

- current scope revision and alignment digest;
- accountable Team Leader;
- accepted task and acceptance criteria;
- Role Card when specialization is needed;
- Delegation Permit;
- Context Capsule;
- base revision and isolated worktree for writes;
- non-overlapping write domain;
- output schema and artifact directory;
- validation/evidence expectations;
- stop conditions and escalation owner.

### 4.5 Execution and local join

The parent that spawns a child owns:

- capacity reservation within its permit;
- `spawn_agent`/`followup_task` choice;
- `wait_agent` loop;
- result validation;
- first-line review;
- failure classification;
- compact upward report.

No branch polls agents or a filesystem queue. A V2 completion message is only a lifecycle event; state changes require schema, authority, permit, lease, digest, write-domain, and evidence validation.

### 4.6 Integration gate

Each workstream has one Team Leader-owned integration queue. Parallel writers use isolated worktrees and disjoint declared domains. The Team Leader reviews patches and evidence, resolves local interface issues, and integrates in plan order.

Cross-workstream integration has one named technical owner chosen in the delivery plan. The Manager resolves ownership/priority collisions but does not choose the technical resolution.

### 4.7 Assurance gate

Quality is derived from risk dimensions rather than one maturity label:

- maturity/use case;
- data sensitivity;
- blast radius;
- reversibility;
- availability/operational exposure;
- compliance/external obligations.

The Quality Governor compiles only triggered gates. A prototype handling restricted data still receives data/security controls; an enterprise label does not justify unrelated ceremony. Implementation and independent verification remain separate.

## 5. Communication and context architecture

### 5.1 Three channels

1. **V2 mailbox** — compact live commands, deltas, receipts, and completion activity.
2. **Repository artifacts** — durable charter, decisions, plans, permits, capsules, results, evidence, and handoffs.
3. **Runtime/event store** — transactional state, leases, attempts, paths, and reconciliation data.

Files are not a mailbox. Agents do not poll `messages/` or task folders.

### 5.2 Parent-local communication

The default communication path is child → physical parent → bounded branch report → parent above. Direct cross-branch messaging is reserved for a documented interface or service request and carries an artifact reference rather than full prose.

- `send_message`: passive delta/receipt that may wait; also useful for a compact report to `/root` while it is waiting.
- `followup_task`: new work or immediate review for an existing seat; consumes capacity.
- `wait_agent`: normal event loop at every parent branch.
- `interrupt_agent`: cancellation, stale scope, runaway behavior, write conflict, or policy/safety stop.
- `list_agents`: startup/recovery/path reconciliation, not routine status polling.

### 5.3 Fork policy

- `fork_turns: "none"` is default. Role Card + Context Capsule are the assignment context.
- bounded recent turns are allowed only for an immediate parent-child continuation where the live negotiation is itself required;
- full-history fork is exceptional because it expands context and prevents role/model/reasoning overrides.

### 5.4 Context Capsule

A capsule contains only:

- identity: project/task/capsule, revision/digest, permit, Role Card;
- assignment: goal, definition of done, deliverables;
- boundaries: writable/forbidden paths and non-goals;
- constraints and interfaces;
- `must_read` references and triggered `read_on_demand` references;
- base revision, worktree, dependencies, and stop conditions;
- validation commands and evidence;
- escalation authority and reversible default;
- output schema/artifact directory;
- input, summary, tool-call, and child-agent budgets.

Leaf capsules set `max_child_agents = 0`.

### 5.5 Result and branch-report compression

A leaf result must include an explicit `no_findings` flag and no more than five material findings. It references changes/evidence and names the next action. The Team Leader or other branch parent produces a separate `branch_report` with only organizational or gate consequences. Chronology and raw analysis stay out of upward context.

## 6. Delegation Permit

A Delegation Permit is the enforceable bridge between a decision and a V2 call. It names:

- issuer and authorized parent path/role;
- scope revision and alignment digest;
- wave/task/gate purpose references;
- allowed child archetypes and tasks;
- maximum children, concurrency, and attempts;
- token/time class;
- parallel writer and allowed-domain limits;
- validity window;
- stop conditions;
- exceptional authorization, if any.

A permit is not permission to invent work. It is capacity and authority to execute an already accepted branch plan. It expires on its checkpoint, time limit, exhaustion, revocation, or binding revision change.

## 7. Workflow shapes

| Shape | Use | Join owner | Main hazard |
|---|---|---|---|
| `single_owner` | coupled/high-context work | assignee/TL | unnecessary delegation |
| `parallel_map` | independent read items or disjoint transformations | TL/Architect | duplicate context and write overlap |
| `sequential_pipeline` | canonical output feeds next stage | stage parent | passing raw transcripts |
| `review_loop` | implementer + independent reviewer | TL | reviewer self-patching or endless loops |
| `fanout_verify` | bounded investigators/builders plus judge | TL/QG/Architect | uncontrolled agent count and weak join |

Every non-single workflow names a maximum child count, maximum concurrency, reserved review capacity, join owner, target artifact, and checkpoint. Large repetitive work is a fan-out candidate; small ambiguous irreversible work is a strong-model single-owner candidate.

## 8. Model and reasoning decisions

Routing uses seven axes:

1. cognitive complexity;
2. interface/taste judgment;
3. uncertainty/discovery;
4. blast radius and reversibility;
5. context volume/coupling;
6. tool/capability requirements;
7. throughput/cost and independence.

| Work | Default model | Effort | Rationale |
|---|---|---:|---|
| grounding, architecture, consequential management, gate verdict | `gpt-5.6-sol` | high | ambiguous tradeoffs and high-value judgment |
| Team Leader | `gpt-5.6-sol` | medium, high for critical integration | retains workstream context and taste |
| context/role compilation, exploration, review, verification | `gpt-5.6-terra` | medium | structured repository reasoning at balanced cost |
| bounded mechanical edit | `gpt-5.6-luna` | low | exact reversible transformation |
| ordinary implementation | `gpt-5.6-terra` | medium | default coding route |
| ambiguous/high-consequence implementation or independent advice | `gpt-5.6-sol` | high | difficult reasoning or second opinion |

Effort semantics in this framework:

- **low** — exact transformation;
- **medium** — normal repository work;
- **high** — ambiguity, architecture, security, difficult debugging, consequential decisions;
- **xhigh/max** — rare, single-agent extended reasoning with a clear value/cost justification;
- **Ultra** — a multi-agent workflow topology, not merely “think harder.” It is prohibited in agent archetypes. An exception needs a named owner, child/concurrency/token-time budgets, isolation, stop condition, and recorded authorization.

Routing is a hypothesis. Escalate after a diagnosed failure, contested review, or changed risk—not because a task sounds important. Do not change model mid-attempt; start a new recorded attempt.

## 9. Alignment, drift, and blockers

An alignment digest covers the binding charter, accepted decisions, plan/policy versions, and relevant interface contracts. Capsules and results pin the digest.

Drift classes:

- local implementation defect → Team Leader;
- tactical sequencing/interface within workstream → Team Leader;
- milestone/workstream/system boundary → Delivery Architect;
- ownership/capacity/priority → Manager;
- product behavior/scope/external constraint/assurance assumption → Consultant + Operator;
- assurance evidence/policy interpretation → Quality Governor.

Only affected work is frozen. Re-grounding returns to the earliest invalidated gate, not automatically to the beginning.

A valid blocker states concrete evidence, blocked action, missing authority/dependency, impact, attempted resolutions, reversible default, owner, and deadline. “Unclear,” “need confirmation,” or “might be risky” is not enough.

## 10. Failure and recovery

| Failure class | Response |
|---|---|
| transient transport/rate/process | bounded retry with same meaning |
| stale revision/digest/dependency | invalidate, impact-check, repackage |
| semantic task failure | TL diagnoses and changes task/capsule/route |
| planning failure | Delivery Architect revises affected plan |
| grounding failure | formal re-grounding of affected behavior/risk |
| authority/access | decision request to named owner |
| policy/safety/write conflict | interrupt affected branch, preserve evidence, escalate |

Never replay a semantic failure with the same vague prompt.

On restart or uncertain state, reconcile from leaves upward: durable events/permits/leases/worktrees, then one `list_agents` snapshot, then terminal/running/stale/orphaned classification. Reuse a resident thread only when role/config/security and durable handoff are still valid. Late results remain evidence but cannot transition reassigned work.

## 11. Skills

| Skill | Owner | Purpose |
|---|---|---|
| `orchestrate` | root | global phase/policy/operator loop |
| `reconcile-session` | root | restart/steering/timeout reconciliation |
| `run-bounded-branch` | authorized parent | spawn/wait/join/report local V2 branch |
| `ground-project` | Consultant | decision-complete grounding |
| `architect-delivery` | Delivery Architect | milestone/workstream design |
| `manage-taskforce` | Manager | Team Leaders, staffing, capacity, priorities |
| `design-role` | Role Architect | approved Role Card |
| `lead-workstream` | Team Leader | one tactical wave and squad branch |
| `package-context` | Context Engineer | minimal Context Capsule |
| `execute-assignment` | Worker/TL | bounded implementation and evidence |
| `review-assignment` | Reviewer/TL | independent review and explicit no-findings |
| `verify-milestone` | QG/Verifier | gate evidence and verdict |
| `handle-drift` | correct owner | classify and route drift |
| `handoff-result` | any role | transcript-free continuation |

Agent TOMLs define enduring authority/personality. Skills define progressive-disclosure procedures. Schemas and policies define machine-checkable contracts. Do not duplicate all three into every prompt.

## 12. Security and permissions

- Consultant, Delivery Architect, Manager, Role Architect, Context Engineer, Quality Governor, Explorer, Advisor, and Reviewer default read-only.
- Team Leaders and Workers receive workspace write only in isolated worktrees.
- Verifiers may create build/test artifacts but cannot modify production source.
- Tools, MCP, network, secrets, and data classes are explicit Role Card/capsule fields.
- Credentials never enter prompts, capsules, events, or artifacts.
- The effective sandbox may only narrow inherited permissions, never silently widen them.
- Write tasks require a base revision, lease, declared domain, and serialized integration owner.

## 13. Prototype to enterprise operation

The same organization can run at different assurance levels. The difference is controls, not role theatre.

### Prototype

- one active Team Leader;
- one or two leaf workers;
- smoke/acceptance checks;
- supervised root session;
- no sensitive production data;
- manual worktree/integration review.

### Production

- risk-derived full test/review gates;
- independent reviewer/verifier where triggered;
- transactional event/lease service;
- CI/PR integration and recovery;
- cost/rate budgets and secret isolation.

### Enterprise/high-stakes

- policy-as-code permits and approvals;
- audit export and immutable decision/evidence records;
- strict identity/secret/data boundaries;
- security, migration, operations, compliance, stress/fuzz gates where triggered;
- crash recovery, idempotency, SLOs, abuse/fan-out controls, and adversarial evals.

Do not call every pilot enterprise. Do not skip data/security controls merely because the UI is a prototype.

## 14. Conformance tests

1. Manager given code/task decomposition returns a delegation decision and creates no worker.
2. Manager can parent Team Leaders/Role Architect only.
3. Team Leader can parent an approved worker only with a permit/capsule and cannot exceed depth/budget.
4. Leaf role cannot spawn.
5. Worker completion is joined by the Team Leader; root sees only a branch report.
6. Consultant/Architect/QG read-only fan-out stays inside permit and returns a bounded synthesis.
7. Context Capsule rejects transcript dumps, stale digests, missing permits, or leaf child budget above zero.
8. Overlapping write domains cannot hold concurrent leases.
9. Reviewer returns explicit target and `no_findings=true` or at most five evidence-backed findings.
10. Semantic failure produces a diagnosed repair attempt, not blind replay.
11. Re-grounding invalidates only affected capsules/tasks and returns to the earliest affected gate.
12. Recovery reconciles V2 paths, permits, leases, and worktrees without duplicate work.
13. Prototype/restricted-data and production/public cases derive different, risk-correct gates.
14. End to end: intent → grounding → delivery design → staffing → wave → local join → integration → assurance → acceptance.

## 15. Decision summary

The framework optimizes for alignment per token, not maximum agent count:

- stable roles hold authority;
- generated Role Cards add only task-specific expertise;
- Context Capsules replace transcript inheritance;
- bounded V2 hierarchy keeps detailed completion traffic at the competent parent;
- Delegation Permits make every child visible and budgeted;
- Manager manages people and decisions only;
- Team Leaders manage squads and retain hard technical context;
- workers are leaves;
- workflows accelerate local waves; checkpoints govern the program;
- independent review and assurance are explicit, proportional, and evidence-based.
