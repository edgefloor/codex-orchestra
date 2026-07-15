---
status: accepted
---

# Unify Codex tasks and Orchestra execution without conflating their state

The Orchestra product is a drop-in Codex desktop replacement with dynamic workflows and the
nested, observable agent lifecycle intended by Codex V2 subagents. A normal Codex task remains the
user-visible conversation and history container, while Orchestra owns a durable execution graph.
The product combines V1-style distilled result delivery with V2-style live, inspectable nested
agents rather than making the workflow debugger the default interface.

## Runs and ownership

A Workflow invocation occurs inside one Codex task and creates that task's Root Run. Task prompts,
skills, and native tool calls are the MVP entry points; a future UI shortcut may initiate the same
recorded task action but cannot create detached execution. Once invoked, Rust owns deterministic
scheduling while the root task remains the conversational coordinator and observer.

The invoking native tool call remains resident while the Run is active. It returns only when the
Run is terminal or durably suspended for a Decision Gate, Permission Request, human input,
reconciliation, or interruption. Turn interruption and host loss fence active Attempts, request
best-effort child cancellation, and record `invocation_interrupted` suspension; they do not leave a
background scheduler running. A later task-native invocation reconciles and resumes the same Run.

A Codex task has at most one nonterminal Root Run. Run Trees in different tasks may execute
concurrently. A workflow may spawn recursively nested Child Runs, but a Child Run remains owned by
its parent, stays inside the Root Run's ownership boundary, and cannot be detached, reparented, or
outlive its parent. Cancelling a Run cancels all descendants; cancelling the Root Run cancels the
entire tree.

A parent cannot complete while a Child Run is nonterminal. Child output becomes the output of the
spawning Step. Child failure follows that Step's retry and failure policy and does not cancel
siblings unless the parent declares fail-fast behavior. Parent failure and cancellation always
cascade downward.

Every Run has an immutable globally unique `run_id`. Recovery resumes the same Run and Attempt.
Retry after a terminal failure creates a new Attempt and, when the Step spawns a workflow, a new
Child Run. The earlier Child Run remains immutable terminal evidence. A Child Run records its
parent Run, spawning Step, parent Attempt, deterministic child ordinal, and initiating agent.

## Execution graph and identities

The canonical model is an Execution Graph relating Runs, dependency-DAG Steps, Attempts, agents,
Decision Gates, and external effects. Its typed identities are:

- Codex-owned `task_id` and `turn_id`;
- globally unique Orchestra `run_id`;
- Step identity `(run_id, stable_plan_step_key)`;
- Attempt identity `(run_id, step_key, monotonic_attempt_no)`;
- immutable host `agent_id`, optionally bound to exactly one Codex thread ID;
- a declared gate occurrence within an Attempt plus append-only decision revision;
- a declared effect slot within an Attempt; and
- a task event `(task_id, task_local_sequence)` retaining its upstream source IDs.

Display names, task paths, nicknames, object addresses, and array positions are never durable
identities. Workflow agents are canonical Codex children. If the pinned native lineage operation is
unavailable, invocation fails; the product does not counterfeit lineage with linked normal threads.

## State authority, journal, and observation

Repository-local state under `.codex/orchestra/runs/` is authoritative for workflow execution,
outputs, gates, effects, and recovery checkpoints. The Codex product fork extends its existing
rollout-backed thread store and SQLite `StateRuntime` with bounded task sequencing, replay, product
snapshots, attachment discovery, and live protocol recovery. This Desktop projection state is
rebuildable and never completes or mutates a Run independently.

Semantic workflow history enters Codex through its existing extension-owned turn-item seam. The
product fork adds one namespaced Orchestra extension variant under `TurnItem::Extension`, persists
it through the normal `RolloutItem` path, and exposes it as `ThreadItem::Orchestra`. Its payload is
the closed generated `OrchestraThreadItem` discriminated union, with typed Run, Step, Attempt, gate,
effect, and agent members. Each item carries stable identity, bounded display state, and references.
It does not embed an authoritative checkpoint, large evidence, or generic extension JSON, and
Orchestra does not define a parallel rollout record family.

Every lifecycle entity has a stable item ID and monotonically increasing semantic revision. Codex
appends each accepted revision to the rollout and `thread/read` collapses revisions by item ID into
the latest visible `ThreadItem::Orchestra` state. Older revisions remain append-only semantic
history and do not inflate the Task snapshot. An item revision orders updates to that entity; the
task-local protocol sequence independently orders renderer delivery across all task events.

Raw Codex conversation, tool, terminal, diff, and turn events remain only in the native Protocol
stream of the task that produced them. The parent task receives typed Orchestra lifecycle items
with the child task ID and bounded child status or result summary, not copies of the child's raw
events. The renderer reads or subscribes to a child task when that child is expanded. The complete
UI view is therefore a composition of task-scoped streams, while each stream remains canonical and
non-duplicated.

After committing an authoritative repository transition, `orchestra-core` publishes a typed
transition through an explicit in-process host seam. Codex assigns the task-local sequence, appends
the event through its existing persistence interfaces, and projects it. Orchestra cannot write
rollout records, replay tables, or product snapshots directly. A crash between checkpoint commit
and journal append is repaired by reconciling repository state; projection-state loss never
invalidates a Run.

The Rust host appends every accepted task-scoped Codex notification and Orchestra lifecycle
transition to one bounded task-local replay journal in `StateRuntime` before projecting it. Sequence
defines replay and display order, not causality; graph references define causal meaning. Transport
sequence fields are not added to canonical rollout records. Snapshots record their last included
sequence, and reconnect resumes from the following event. Stable upstream identities deduplicate
when available; ambiguous duplicates are retained rather than guessed away. Loss of Desktop
projection state invalidates its cursors and requires a fresh snapshot; it does not rewrite rollout
history or invalidate execution checkpoints, though unreconstructable transient presentation events
may be lost.

A task snapshot composes Codex's existing `thread/read` result with the Orchestra execution
projection at one task-local replay barrier. The Codex thread and rollout store remain the sole
durable source for conversation history; Desktop projection state does not persist a second full
conversation copy. The composed snapshot is a renderer hydration contract, not a new domain or
execution authority. Its Orchestra section contains the current inspectable Run Tree, Attempts,
statuses, gates, digests, and stable evidence references. Full transition history and large tool or
evidence payloads stay in their existing authoritative stores and are retrieved through bounded,
authorized history, evidence, digest-expansion, or attachment reads.

Only task-scoped events enter durable sequencing and replay in the MVP. Account, configuration,
update, and host-health state rehydrates through existing App Server reads or snapshots and then
continues through live notifications. Those global notifications have no durable cursor, and the
product defines no total order across tasks.

Everything in the Execution Graph is observable. The primary UI is the familiar Codex task
timeline with inline, expandable nested agents, live status, concise progress, and click-through to
each agent's conversation, tools, terminal, diffs, plan, and result. Runs organize execution but do
not receive equal default prominence; Attempts, checks, gates, and effects appear progressively.
Terminal, failed, cancelled, and stale agents remain inspectable.

This observation model follows Context economy: the root task and root model coordinate from
bounded lifecycle projections and the Run Digest, while detailed child context is loaded only for
the UI or model operation that explicitly expands it. Convenience never justifies copying whole
child histories into the parent task, snapshot, or prompt.

The root Codex thread observes the whole Run Tree through a bounded deterministic Run Digest derived
from checkpoints and the Execution Graph. It contains current status, recent material changes,
active agents, waiting work, gates, effects, outcomes, and next actions, with stable IDs, a version,
deltas, and targeted expansion. The Codex fork contributes it through the existing extension-owned
World State section and its snapshot-and-diff lifecycle. Digest refreshes therefore replace or diff
current coordination state instead of appending transcript messages. Raw lifecycle events do not
accumulate in model context. Explicit attention notifications for failures, gates, permissions, and
human input, plus distilled child results, remain available when the parent consumes them. Child
agents see only their declared context and permitted graph slice.

Digest generation has a hard byte and token budget and is deterministic for the same authoritative
state, schema, and budget. It retains information in this order: required human actions,
permissions, failures, and unresolved gates; active agents and executing work; newly completed
outcomes and material changes; blocked or waiting work and next actions; then older successful
detail. Stable IDs and counts for omitted items always remain so the consumer can request expansion.

This is a deliberate two-channel observation boundary. The renderer composes task-scoped replay
streams for UI projection and inspection. The root model consumes only the bounded digest,
distilled results, attention events for failures/gates/permissions/human input, and explicit
expansion responses. Desktop projection state never injects its raw journal into model context.

Targeted expansion is implemented once as a bounded, authorized query service in `orchestra-core`.
It selects by stable Execution Graph identities and reads current projections, authoritative
checkpoints, outputs, and evidence references with explicit depth, field, page, and byte limits. A
native Codex task tool adapts it for root-model use; typed App Server methods adapt it for the
renderer. The adapters apply consumer-specific budgets and presentation schemas, but share
selection, authorization, identity, pagination, and stale-revision behavior.

The MVP query surface is a closed set of typed selectors for Runs, Steps, Attempts, agents, gates,
effects, outputs, evidence, and history pages. New selectors require a concrete product consumer and
schema addition. Orchestra does not implement a general graph-query language, arbitrary traversal,
or renderer-supplied projection expressions.

Workflow ownership does not remove native V2 steering. A user or responsible root agent may steer a
workflow-owned child through Codex's existing steer operation. The Orchestra extension recognizes
that the target agent belongs to an Attempt, records a monotonic steering command with initiator,
authority, target, content digest, and expected Attempt revision, then forwards it natively. The
delivery outcome is appended to Attempt evidence. Duplicate command IDs do not deliver twice;
recovery reconciles an ambiguous delivery before accepting another steer. Ordinary non-workflow
children keep unmodified Codex steering behavior.

A Workflow steer augments the child's live conversation; it does not rewrite the immutable initial
Context bundle or execution plan. The Run Digest reports material steering, and output validation
and review evidence retain the steering sequence that influenced the Attempt.

## Lifecycle and recovery

A Run has durable phase `created`, `active`, `suspended`, `cancelling`, or terminal `completed`,
`failed`, or `cancelled`. Suspension carries typed reasons such as Decision Gate, Permission
Request, human input, effect reconciliation, dependency, or explicit pause. Recovery is an
operational mode, not a durable outcome.

Completion requires every required Step and Child Run to complete and every external effect to
have a committed receipt. Failure occurs only after declared retry/failure policy is exhausted or
reconciliation proves failure. Cancellation becomes terminal only after descendants and active or
unknown effects reach safely recorded terminal or reconciled states. Infrastructure loss is not a
semantic workflow failure and does not by itself justify retry.

Each Run Tree has one fenced execution lease with a host owner, heartbeat, and monotonically
increasing epoch. A recovering host advances the epoch before reconciling repository checkpoints,
Codex projection events, threads and turns, Child Runs, and unknown effects. Every state, decision,
effect, and agent-result commit carries the epoch. Late work from a stale owner remains evidence but
cannot commit state or effects.

## Permissions, gates, and effects

A Codex Permission Request controls access to protected capabilities and follows Codex permission
policy. A workflow Decision Gate is a semantic choice governed separately by Gate Policy and
existing authority. Gate Policy supports `ask_human`, `auto_accept`, `auto_reject`,
`delegate_to_agent`, and `inherit`, and may vary by gate kind, risk, or effect.

Organization constraints are enforced bounds that prohibit disallowed resolvers. Within those
bounds, precedence is Root Run override, task setting, product setting, then workflow default. The
effective revision is recorded. Child workflows inherit it and may narrow but not silently broaden
behavior. A new revision applies to gates created afterward; pending gates retain their resolver
unless the user explicitly reapplies policy. Policy never grants undeclared authority.

Child gates bubble through their ownership lineage into the Root Run's task and remain addressed by
immutable gate identity. Multiple gates may wait concurrently while independent work continues.
Free text never implies acceptance. Automatic and delegated resolutions record effective scope,
policy revision, resolver, chosen outcome, authority, and evidence.

For `delegate_to_agent`, the workflow defines goals, boundaries, evidence, authority, and output
schema rather than a fixed reasoning process or personality. An eligible responsible agent may be
selected dynamically, create bounded Child Runs, and adapt its model, reasoning effort,
instructions, context, and topology. Its outcome becomes durable only on commit. Failure before
commit may invoke another agent and produce a different answer; reconsideration appends a new
decision revision. Recovery reuses committed outcomes without requiring reasoning reproducibility.

A committed gate authorizes but does not prove an external effect. Each effect has a stable ID and
records intent before dispatch and a reconciliation receipt afterward. A crash between them leaves
`outcome_unknown`. Recovery reconciles before retry; automatic retry requires a supported
idempotency key or proof that the effect did not occur. Reconsideration never erases or implicitly
reverses an already committed effect.

## Codex task lifecycle

Archiving a task is presentation-only and does not stop its resident invocation or Run Tree.
Deleting a task with a nonterminal Root Run is rejected unless the user explicitly chooses to cancel the Run Tree
and delete. Deleting a terminal task does not delete repository records. Turn interruption durably
suspends and fences the Run as described above; explicit workflow cancellation instead transitions
the selected Run and descendants toward terminal cancellation.

Forking a Codex task copies conversation history but never copies a Run identity or live Run Tree.
An explicit workflow-fork action creates a new Root Run with a new ID and `derived_from_run_id`, and
does not inherit unresolved gates, live children, or effect completion claims. Child Runs never
create top-level sidebar tasks.

## Compatibility and consequences

There is no migration contract for Runs created by the patched-desktop/plugin experiment. Existing
directories may remain as experimental evidence, but the product does not resume or upgrade them.
All product Runs begin with this identity and state model.

This decision supersedes ADR 0001's prohibition on host-local product state while retaining
repository-local execution authority. It also supersedes ADR 0013's assumption that every workflow
approval is necessarily a human action; Permission Requests and Decision Gates now have separate
policy and authority semantics.
