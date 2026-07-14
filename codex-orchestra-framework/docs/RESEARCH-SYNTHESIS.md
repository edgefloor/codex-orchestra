# Input and Video Synthesis — Revision 1

## Sources considered

- `V2_AGENT_ARCHITECTURE.md` — Codex Multi-Agent V2 paths, mailboxes, lifecycle, forking, config inheritance, residency, and collaboration tools.
- `example-design.md` — initial Symphony Crew roles, state, hiring, alignment, and maturity proposal.
- `manager.zip` — Symphony orchestration repository/spec, Manager boundary notes, Codex agent examples, and skill patterns.
- Theo Browne, **“A proper guide to Fable 5”** (`8GRmLR__OGQ`, July 6, 2026) — full transcript retrieved and analyzed.
- Theo Browne, **“I can't believe they released this”** (`t8hfOyF4ehw`, July 14, 2026) — full caption transcript was not retrievable at revision time; analysis is limited to verified metadata/publisher description and independently verified official Codex semantics.
- Current official Codex documentation for subagents, custom agent TOMLs, model selection, reasoning modes, and harness engineering.

## V2 findings

V2 provides named paths, asynchronous mailboxes, event-driven waits, automatic child-to-parent terminal notifications, resident agent restoration, and explicit fork/override behavior. This changes the architecture in two ways.

First, repository files should be durable contracts/evidence rather than an agent-polled message queue. `send_message`/`followup_task`/completion activity are the live bus; `wait_agent` is the event loop.

Second, physical parenthood is a context-routing decision. A flat tree sends every completion to root. A bounded hierarchy lets the competent parent join detailed work and return a summary upward. Revision 1 therefore replaces root-only spawn authority with role-restricted, permit-bound branch authority.

## First video findings

### Dynamic workflow inside stable governance

The video distinguishes persistent routing policy from task-specific generated workflows/subagents. The reusable idea is:

- keep authority, safety, budgets, output contracts, and checkpoints static;
- generate investigator/reviewer/judge or specialist roles for the actual bounded task;
- evaluate results and escalate rather than assuming the initial route is perfect.

Codex Orchestra implements this with stable TOML archetypes plus generated Role Cards and tactical workflow shapes.

### Route by intelligence, taste, risk, and cost

The video treats mechanical/bulk work differently from UI/API/copy/code-quality judgment. Revision 1 generalizes routing to complexity, interface/taste, uncertainty, blast radius/reversibility, context volume, tool needs, throughput/cost, and independence.

### Bounded fan-out, explicit join

The 16-PR example used one investigator per item and multiple judging perspectives. The transferable pattern is not the raw count. It is independent map items, structured verdicts, reserved judge capacity, contested-case escalation, and a named join owner. The framework caps ordinary dynamic nodes and requires a Delegation Permit.

### Long programs are checkpointed

The transcript explicitly separates a deterministic fan-out/verify workflow from a multi-hour program involving worktrees, CI, reviews, merge order, and product decisions. Revision 1 makes the project loop phase/checkpoint-driven and confines workflows to one wave.

### Skills improve from real failures

Observed failures should refine skills/policies. The framework requires lessons to be general, testable, versioned, and placed in the narrowest source of truth; raw incident chronology is not appended to always-loaded instructions.

## Second video findings and limitation

The verified publisher description characterizes Ultra as repeated maximum-reasoning subagent spawning with potentially rapid allowance consumption. Official Codex docs independently distinguish Max (more compute for one task) from Ultra (subagent delegation) and note that most tasks do not require either.

Safe design consequences:

- treat Ultra as an orchestration topology;
- never hide it in a worker TOML;
- require a named owner and explicit child/concurrency/token-time/isolation/stop budgets;
- prefer an ordinary bounded V2 branch whose paths and results are visible;
- use Max only for a rare single-agent decision where extra reasoning has clear value.

No detailed timestamped claims from the unavailable second transcript are included.

## Attachment-derived role refinements

### Kept

- Operator + Consultant grounding;
- strict Manager boundary;
- Team Leader as leader and executor;
- context specialization;
- generated temporary roles;
- formal hiring;
- re-grounding/drift;
- maturity-aware assurance;
- durable decision/audit artifacts.

### Refined

- **Scope Specialist → Context Engineer:** scope authority stays with Consultant/Operator; the compiler only packages accepted context.
- **Delivery Architect added:** someone other than the Manager must own milestone/workstream design.
- **Quality Governor added:** gate derivation and verdict need an independent owner.
- **Role Architect becomes a Manager-owned service:** it compiles, but does not approve, roles.
- **Bounded hierarchy replaces flat physical topology:** Manager parents Team Leaders; Team Leaders parent their squads.
- **Delegation Permit added:** nested V2 authority is explicit, revision-pinned, and budgeted.
- **Independent Reviewer added:** review is distinct from assurance verification.
- **Risk dimensions replace a single rigor switch:** maturity is one dimension among data, blast radius, reversibility, availability, and compliance.
- **No fabricated blockers:** reversible uncertainty receives a default; only concrete evidence/authority/dependency/safety/irreversibility blocks affected work.

## Final principle

Static authority creates alignment. Dynamic Role Cards and bounded workflows create leverage. Parent-local joins compress context. Durable artifacts and permits make the hierarchy auditable. The result is a managed organization, not an unconstrained swarm.
