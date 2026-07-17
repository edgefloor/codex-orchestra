# Orchestra

## Domain language

**Workflow source**: An Agents SDK-compatible `.workflow.ts` module parsed and lowered by Rust from a pinned module graph without executing it as JavaScript.

**Workflow compilation**: Deterministic Rust parsing and lowering of a closed workflow module graph into a canonical execution plan and schema artifacts before a Run exists.

**Execution plan**: The validated internal Rust representation produced by workflow compilation and consumed by the runtime without executing workflow TypeScript again.

**Validation bundle**: The recorded, hashed source that defines the schemas available to a Run. Its source is authoritative; evaluator-specific compiled forms are disposable caches.

**Workflow artifact**: The immutable, content-addressed compilation result that binds an Execution plan and Validation bundle to their source graph, guidance schemas, and exact Product compatibility.

**Canonical value**: A JSON-compatible value whose representation is portable across workflow compilation, validation, checkpoints, recovery, and evaluator revisions.

**Custom value type**: A stable, versioned type identity with a JSON wire schema, deterministic validation issues, and an optional transformation into a canonical value. A type without a canonical JSON representation is compile-time-only.

**Codex task**: The user-visible Codex conversation and history container. A task has at most one nonterminal Root Run, while Run Trees in different tasks may execute concurrently.

**Run**: One runtime-owned execution of a plan against a repository revision and parent Codex task.

**Workflow invocation**: A user- or agent-initiated action inside a Codex task that asks native Orchestra to create or resume a Root Run from a validated workflow and resolved inputs. Its native tool call remains resident until the Run terminates or durably suspends; it never becomes detached background execution.

**Automation**: The issue-driven Orchestra mode that selects tracker work and coordinates authored Workflows from a visible Codex task.

**Automation task**: The Codex task that owns one active Automation Root Run for a repository and tracker project.

**Automation profile**: A versioned declaration of issue eligibility, coordination policy, workflow selection, and prompt guidance for Automation.

**Automation Root Run**: The resident Root Run owned by an Automation task and responsible for its Issue claims.

**Issue task**: The persistent Codex task representing one claimed tracker issue within an Automation Root Run.

**Issue claim**: The exclusive, lease-scoped ownership of one tracker issue and its current Automation attempt.

**Tracker effect**: A typed, policy-gated tracker mutation scoped to the current Issue claim and recorded by durable identity and receipt.

**Root Run**: The workflow entry-point Run created by a Workflow invocation and owned by that invocation's Codex task.

**Child Run**: A Run spawned and owned by another Run. It cannot be detached, reparented, or outlive its parent; retry creates a new Child Run, while recovery resumes the existing one.

**Run Tree**: A Root Run and all of its transitively owned Child Runs. Cancelling a Run cancels all of its descendants, so cancelling the Root Run cancels the entire tree.

**Execution Graph**: The canonical graph relating Runs, Steps, Attempts, agents, Decision Gates, and external effects. Step dependencies may form a DAG, while Run and agent ownership provide tree projections for navigation.

**Run Digest**: A bounded, deterministic, LLM-friendly projection of a Run Tree's current state and recent material changes. It is derived from the Execution Graph and injected through Codex's extension-owned World State section so newer snapshots replace or diff older state instead of appending transcript messages. Under its hard budget it preserves, in order, required actions and failures; active work; new outcomes and material changes; blocked work and next actions; then older successful detail. Stable IDs and omission counts make truncated detail expandable. It supports targeted expansion and is never execution authority.

**Step**: One agent, check, or Decision Gate action with dependencies, attempt bounds, optional repeat bounds, context, outputs, and worktree policy.

**Attempt**: One execution of a Step within a Run. Retry creates the next Attempt; crash recovery resumes the existing Attempt.

**Workflow steer**: A native Codex steering instruction addressed to an agent owned by an Orchestra Attempt. Orchestra records its target, initiator, authority, sequence, content digest, and delivery outcome before and after forwarding it through Codex's existing steer operation.

**Stage**: A conflict-free batch of dependency-ready Steps executed against one Stage snapshot, up to `max_parallel`. Write leases may defer an otherwise-ready writer to a later Stage.

**Stage snapshot**: The immutable, content-addressed repository view captured from a Run worktree for one Stage. Every Attempt in that Stage observes the same repository bytes.

**Run worktree**: The single runtime-owned mutable repository view for a Run. It accumulates validated ChangeSets and is the repository state against which aggregate checks run before Promotion.

**Attempt overlay**: A private copy-on-write repository view for one writer Attempt, derived from its Stage snapshot and writable only within its reserved Write scope. It cannot mutate the Run worktree directly.

**Write scope**: The declared repository paths a writer Step is authorized to change. It is enforced by the runtime's workspace isolation rather than treated as prompt guidance.

**Write lease**: A runtime reservation of a Write scope or named exclusive mutation class for one Attempt. Conflicting leases cannot be held concurrently.

**ChangeSet**: An immutable candidate transaction produced by a writer Attempt, binding its Stage snapshot and observed read hashes to its changed paths, file metadata, binary-safe patch, and canonical digest. It remains evidence until the runtime validates and applies it.

**Context bundle**: Exact bytes materialized from declared files, line ranges, revisions, diffs, and dependency outputs, with a SHA-256 digest.

**Run input**: A typed, run-specific value resolved before scheduling, canonically serialized, hashed, and persisted independently of the parent transcript.

**Skill requirement**: An exact enabled skill identity plus its declared transitive skills and resources, resolved through the native host and snapshotted before an agent starts.

**Human input**: A durable free-text or structured response that resumes a paused workflow without granting acceptance authority.

**Native host**: The narrow Codex capability for parent-linked V2 spawn, status, wait, final response, cancellation, and sandboxed command execution.

**Product fork**: A long-lived, independently shipped fork that selectively incorporates upstream changes while owning its product semantics and compatibility. Orchestra maintains product forks of Codex and the T3Code-derived desktop rather than treating either integration as a temporary patch or disconnected rewrite.

**Product release**: One architecture-specific signed macOS app in which the desktop, native host, generated bindings, pinned integrations, schemas, capabilities, and effective limits form an exact lockstep compatibility unit.

**Release manifest**: The sealed machine-readable identity of one Product release and its compatibility-relevant source, artifacts, schemas, capabilities, limits, signing requirements, and state transitions.

**Authoring plugin release**: An independently versioned distribution of skills, documentation, configuration templates, and editor assets. It never installs or replaces native execution.

**Host protocol**: The pinned Codex App Server JSON-RPC protocol extended in the Codex Product fork with Orchestra methods, lifecycle notifications, and snapshot/replay recovery. It is the sole native agent/workflow interface consumed by the retained T3Code provider adapter; T3Code's local server may adapt it for the normal renderer but never becomes execution authority. Transport privacy does not grant operation authority.

**Protocol stream**: The replayable event partition for one Codex task, containing that task's Codex and Orchestra events under one monotonically increasing local sequence. The MVP has no global or cross-task replay stream; sequence defines display and replay order within the task, while graph references define causality.

**Task snapshot**: A rebuildable desktop read model composed at one task-local replay barrier from Codex's existing `thread/read` state and the current Orchestra execution projection. It includes the inspectable Run Tree, Attempts, statuses, gates, digests, and stable evidence references, but not the full transition history or large payloads. It does not duplicate the full Codex conversation or become authority for workflow execution.

**Orchestra lifecycle item**: A member of the closed generated `OrchestraThreadItem` union, carried through Codex's existing extension-owned `TurnItem::Extension` rollout path and exposed as one namespaced `ThreadItem::Orchestra` App Server variant. It records stable workflow identity and bounded display state, never an authoritative checkpoint or an untyped JSON payload.

**Lifecycle item revision**: The monotonic semantic revision for one stable Orchestra lifecycle item. Each revision is appended to the Codex rollout; `thread/read` collapses revisions to the latest visible state. It is independent of the task-local Protocol stream sequence, which orders desktop delivery across different items.

**Context economy**: The design rule that canonical detail remains in its native authoritative task or store, while each consumer receives the smallest bounded projection needed for coordination and may request targeted expansion. Context is not duplicated across parent tasks, child tasks, renderer snapshots, or model prompts merely for convenience.

**Execution query service**: The bounded, authorized `orchestra-core` read API over Execution Graph identities, current projections, checkpoints, and evidence references. Its MVP surface is a closed set of typed selectors for Runs, Steps, Attempts, agents, gates, effects, outputs, evidence, and history pages—not a graph query language. Codex task tools and App Server methods are separate adapters over this one service, with consumer-specific budgets and presentation schemas rather than duplicated query logic.

**Protocol capability**: A Codex or Orchestra method/event feature reported by the extended App Server initialization exchange for the pinned host bundle and current environment. Absence disables the dependent product behavior and never activates an alternate backend.

**Checkpoint**: Atomic runtime state recording attempts, rounds, statuses, context hashes, validated outputs, evidence, and gate resolutions.

**Permission Request**: A request to use a protected capability. It follows Codex permission policy and is distinct from a workflow decision.

**Decision Gate**: A typed semantic choice in a Run whose resolution is governed by an effective Gate Policy and existing authority.

**Gate Policy**: A revisioned, scope-derived mapping from gate kinds, risks, or effects to `ask_human`, `auto_accept`, `auto_reject`, `delegate_to_agent`, or `inherit`. Root and task settings may override workflow defaults within enforced organization bounds, and Child Runs may narrow but not broaden the inherited behavior.

**Gate Resolution**: A committed, revisioned outcome for a Decision Gate with its effective policy, authority, resolver, and evidence provenance. Recovery reuses committed outcomes; explicit reconsideration appends a new revision rather than rewriting history.

**Desktop projection state**: Bounded, rebuildable replay, product-snapshot, and attachment data implemented by extending Codex's existing thread store and `StateRuntime`. Canonical semantic history remains in rollout records; task-local transport sequences and raw replay tails live only in `StateRuntime`. It is not a separate storage subsystem or authority for workflow execution and effects.

**Execution lease**: The single-writer ownership of a Run Tree, identified by a host instance and monotonically increasing fencing epoch. Late work from an older epoch may be retained as evidence but cannot commit state or effects.

**External effect**: A declared mutation outside the target checkout, such as publishing or closing an issue, bounded by explicit authority and recorded with a reconciliation receipt.

**Promotion**: Conflict-checked application of the aggregate verified isolated-worktree patch into the target checkout after every step and approval succeeds.

**Run summary**: The transcript-independent terminal or paused record under `.codex/orchestra/runs/<run-id>/summary.md`.

## Invariants

- TypeScript workflow source is never executed as JavaScript; Rust parses and lowers it, and a Run executes only its recorded execution plan.
- Runtime schema validation accepts and returns only canonical values; evaluator-specific objects never cross into checkpoints.
- Model and tool guidance uses recorded JSON Schema, while exact runtime acceptance uses the recorded validation bundle.
- A Workflow invocation occurs inside and is owned by its parent Codex task; no renderer, sidecar, or external scheduler creates a detached Run.
- Active workflow execution requires a resident task invocation. Interruption fences active Attempts and durably suspends the Run for reconciliation and later task-native recovery.
- The runtime, not a model, owns scheduling and durable state.
- Design choices optimize context as well as implementation reuse: preserve detail once in its native authority, send bounded summaries by default, and expand on demand for the specific UI or model consumer.
- The product UI may compose the complete observation surface from task-scoped replay streams; the root model receives only a bounded replaceable Run Digest, distilled results, attention events, and targeted expansion.
- The Run Digest uses Codex's native extension-owned World State snapshot-and-diff seam. Digest refreshes do not append transcript messages; actionable failures, gates, permissions, and human-input requests remain explicit attention events.
- A Run Digest has a hard context budget and deterministic priority order. It always preserves stable expansion identities and omission counts; identical authoritative state and budget produce identical digest bytes.
- Workflow-owned agents remain natively steerable. Orchestra durably mediates their Codex steer operations so conversation state, Attempt evidence, and recovery cannot diverge silently.
- Coordination, committed decisions, and effects are durable; agent reasoning, model settings, and Child Run topology may adapt within recorded policy and authority bounds.
- Agent steps use the active task's V2 `AgentControl`; there is no alternate dispatcher.
- `fork_turns` defaults to `none`; exact declared context replaces the parent transcript.
- Models and reasoning settings are step data, not fixed role personalities.
- Repository-local checkpoints are authoritative for workflow execution, outputs, gates, and effects. A Task snapshot composes existing Codex thread state with Orchestra projection data; Codex Desktop projection state may retain replay, snapshot, and attachment data but cannot duplicate conversation authority or complete or mutate a Run independently.
- Semantic Orchestra history uses Codex's native extension-owned turn-item and rollout path. Lifecycle items are typed and bounded; generic JSON blobs and a parallel rollout record family are not protocol seams.
- Orchestra lifecycle items have stable IDs and append-only semantic revisions. Codex `thread/read` returns the latest revision per item; older revisions remain rollout history and are not copied into Task snapshots.
- Raw Codex events stay in their native task's Protocol stream. A parent task receives only typed Orchestra lifecycle projections referencing child task IDs; clients read or subscribe to a child task when they need its conversation, tools, or detailed events.
- Targeted model and renderer expansion shares one Orchestra-owned Execution query service. Native Codex tool and App Server adapters apply their own budgets and output shapes without changing selection, authorization, identity, or pagination semantics.
- The MVP exposes fixed typed expansion selectors and adds another only for a concrete product need. It has no general graph-query language.
- Resumed runs use their recorded inputs, skill snapshots, human responses, and external-effect receipts rather than re-resolving mutable ambient state.
- Writer Attempts produce ChangeSets rather than mutating the Run worktree; only the runtime applies validated ChangeSets, and only the verified aggregate reaches the target checkout after successful checks and acceptance. Rejection or a Promotion conflict never overwrites target files.
- Codex and the T3Code-derived desktop are intentional Product forks. Their upstream revisions are explicit and pinned, native primitives are reused first, and divergence stays concentrated in reviewed seams.
- Native Product components update and roll back as one exact release. Plugin/configuration lifecycle remains independent, and neither path rewrites canonical Codex history or repository Run checkpoints.
