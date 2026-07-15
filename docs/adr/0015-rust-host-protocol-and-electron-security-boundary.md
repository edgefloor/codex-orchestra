---
status: accepted
---

# Extend the pinned Codex App Server protocol as the desktop backend boundary

The T3Code-derived Electron/React shell communicates with `orchestra-host` through one versioned
protocol: the pinned Codex App Server JSON-RPC surface extended in the Codex product fork with
native Orchestra methods, lifecycle notifications, and host-owned snapshot/replay. Codex request
IDs, methods, errors, generated schemas, and events remain the foundation; Orchestra does not wrap
them in a second RPC protocol. Electron main remains a thin product and process-lifecycle shell; it
is not a second backend, event store, permission authority, or Codex client.

The protocol is judged first by whether it preserves the intended Codex V2 subagent model while
exposing the additional typed surface required by dynamic workflows. A mechanism belongs in the
contract only when it supports one of those paths or the shared observation and recovery boundary
between them; speculative general-purpose platform features stay outside the protocol.

This decision is based on the [T3Code seam research](https://github.com/edgefloor/codex-orchestra/issues/15#issuecomment-4982817300)
at pinned revision
[`ecb35f75839925dd1ac6f854efeef5c9e291d11b`](https://github.com/pingdotgg/t3code/tree/ecb35f75839925dd1ac6f854efeef5c9e291d11b)
and the [Codex App Server compatibility research](https://github.com/edgefloor/codex-orchestra/issues/18#issuecomment-4982817271)
at pinned revision
[`f90e7deea6a715bbd153044af6f475eefa749177`](https://github.com/openai/codex/tree/f90e7deea6a715bbd153044af6f475eefa749177).
The former supports retaining the shell behind a replaced client boundary; the latter establishes
that stock App Server notifications have no reconnect cursor or replay and that canonical child
creation is not exposed to clients. Orchestra resolves both gaps by extending the pinned Codex host
in-process, not by wrapping App Server as an external client.

The T3Code-derived Electron/React desktop is also a long-lived product fork, not a temporary UI
transplant. Orchestra retains and selectively synchronizes useful upstream shell and product work,
while owning branding, packaging, the generated client boundary, workflow UX, and security policy.
There is no compatibility requirement with T3Code's replaced Node backend or transport.

## Ownership and topology

The production process and trust topology is:

```text
sandboxed renderer
  -> generated client + one framed MessagePort transport
  -> Electron main byte bridge
  -> inherited orchestra-host stdin/stdout
  -> orchestra-host (pinned Codex host mode)
       -> Codex task, turn, tool, permission, and native V2 agent services
       -> native Orchestra extension, runtime, and repository checkpoints
       -> extended Codex thread store and StateRuntime projections

Electron main <-> separate inherited privileged-control pipe <-> orchestra-host
```

`orchestra-host` is one signed Rust process built from the pinned Codex revision with the Orchestra
extension registered in-process. It owns Codex tasks and turns, native V2 agents, Codex request
correlation, task-native Workflow invocation, Orchestra scheduling, backend authorization, replay
journaling, and product projection. It does not launch or control a second App Server process, and
there is no App Server client, sidecar scheduler, or alternate agent service.
Electron main launches and monitors the signed host, owns its private pipes,
serves packaged assets, and exposes narrowly enumerated OS functions. The preload bridge transfers
one transport `MessagePort` and exposes those OS functions; it does not define a second set of
backend RPC methods. The renderer owns view state only.

The framing and message semantics below are transport-independent, but the MVP uses the same
Electron-owned stdio path in production and development. WebSocket, Unix-socket, remote, mobile,
cloud-relay, multi-user, third-party-client, browser-only, and external-workflow-engine transports
are outside the MVP contract until a concrete client requires one. The extended pinned App Server
surface serves the bundled desktop renderer; dynamic workflows integrate through Orchestra's
compiler and native Rust runtime, while the protocol exposes their control and observation to the
product UI.

## Protocol identity and negotiation

The MVP reuses App Server's `initialize` exchange and adds an exact bundled compatibility tuple to
its parameters and result. The tuple contains renderer and host build IDs, the pinned Codex
revision, generated Codex-plus-Orchestra request/event schema hashes, snapshot schema IDs, required
capabilities, replay-retention boundaries, and effective limits.
These identifiers make drift diagnosable; they do not promise compatibility between independently
upgraded components.

Initialization either confirms the exact tuple and returns the available feature set or returns a
terminal JSON-RPC incompatibility error and closes the data session. A feature name follows the
existing App Server capability model and may identify additions such as `orchestra/run/read` or
`host/replay`. Capability absence disables its UI and request path; it never activates a fallback
backend. No non-initialization request is accepted first, and the host does not silently
down-convert operations or snapshots.

## Framing and envelopes

Frames are length-prefixed, bounded UTF-8 JSON values. Host stdout carries protocol frames only;
logs and human-readable diagnostics go exclusively to stderr. Binary attachments use declared
content-addressed attachment operations rather than unbounded base64 inside envelopes.

Each frame contains one existing App Server JSON-RPC request, response, error, or notification. The
desktop host mode changes stdio framing from line-delimited JSON to length-prefixed JSON, but it does
not change JSON-RPC correlation or introduce a second message ID. An Orchestra request uses the same
shape as a Codex request:

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "orchestra/run/status",
  "params": { "runId": "run_..." }
}
```

JSON-RPC `id` provides request/response correlation. Retry-sensitive mutations additionally carry
an explicit idempotency key; reusing a key returns the recorded result or a typed ambiguity error
without repeating the mutation. Requests and responses have no replay sequence.

Every task-scoped replayable notification retains its original Codex or Orchestra `method` and
`params` and adds one top-level `orchestra` extension object containing its task ID and monotonic
task-local `sequence`. The host appends the notification before delivery. Pipe order preserves
delivery order, but only task-local sequence defines replay order and Execution Graph references
define causality.
Clients ignore unknown top-level extension fields. Unknown notification methods and fields remain
journaled and may be surfaced as raw diagnostics; unknown request methods receive JSON-RPC
`Method not found`. Required fields are never guessed, IDs are opaque, and timestamps are never
ordering authority.

Failures reuse JSON-RPC error responses. Their typed `data` adds a stable safe code, retry class
(`never`, `after_reconnect`, `after_snapshot`, or `after_delay`), and optional bounded details.
Host crashes, transport loss, overload, stale revisions, authorization denial, and Codex failures
remain distinguishable without defining another error envelope.

## Native Codex surface and Orchestra extensions

Codex and Orchestra keep separate command surfaces while projecting into the shared Execution
Graph defined by ADR 0014. Codex commands retain upstream thread, turn, child-agent, approval, and
tool semantics. Orchestra commands add workflow validation, scheduling, gates, recovery, and
effects by building on native Codex capabilities. The common graph unifies observation and product
navigation; it does not impose a synthetic common write API on the two authorities.

A missing capability is added to the pinned Codex integration only when it requires native Codex
identity or lifecycle semantics, such as canonical parent-linked child creation, thread residency,
turn control, collaboration events, or cancellation. Runs, Steps, gates, workflow policies,
evidence, effects, and workflow recovery remain Orchestra-owned and call those native capabilities
through the injected Rust boundary. This rule keeps the Codex product fork synchronized around
native seams without coupling Codex to Orchestra's workflow model.

Native steering remains a Codex command. When its target belongs to an Orchestra Attempt, the
in-process extension adds durable command identity, initiator/authority provenance, expected
Attempt revision, and delivery reconciliation before forwarding through Codex's existing steer
path. It does not create a parallel Orchestra steering protocol.

### Existing Codex methods and lossless events

Renderer-originated Codex commands use generated, revision-pinned request/response schemas and an
explicit per-capability allowlist. The generated client exposes only allowed methods; Rust rejects
unauthorized or host-private App Server methods even if a compromised renderer constructs JSON-RPC
manually. Initialization, capability setup, process supervision, and private Orchestra integration
remain host-only.

The host delivers accepted Codex responses and notifications in their original App Server JSON-RPC
shape without narrowing them through T3Code view-model schemas. Only task-scoped notifications enter
the durable replay journal; responses retain JSON-RPC correlation but have no replay cursor. Known
messages receive typed projections. Unknown task-scoped notifications remain bounded opaque Codex
events, stay journaled, and may be delivered even when no typed projection exists. Exact members,
unknown fields, and Codex identifiers remain compatibility evidence. Secrets identified by the
pinned Codex schema are redacted before diagnostic export, never before an authorized delivery.

Canonical parent-linked V2 lineage is an MVP requirement. The pinned Codex integration exposes one
narrow, parent-task-bound Workflow invocation backed by the existing
`ThreadManager -> OrchestraControl` path. Invocation originates inside the Codex task through an
Orchestra skill/tool call. Rust then uses that native control for child creation, status, events,
final responses, and cancellation. The shell does not start a Run directly, invent child lineage,
or fall back to linked normal threads; if the native operation is unavailable, workflow execution
is disabled with a compatibility diagnostic.

The task's native invocation stays resident while its Run is active and returns only at a terminal
or durable suspended state. Interruption fences active Attempts and suspends for later task-native
recovery; `orchestra-host` does not keep an unowned workflow running in the background.

### Added host and Orchestra methods

Host and Orchestra methods are added to the same generated App Server protocol from Rust-owned
schemas. They extend the product surface without redefining native Codex commands. The initial MVP
families are:

- `host/snapshot/get`, `host/replay/subscribe`, `host/replay/unsubscribe`, diagnostics, and
  content-addressed attachment reads;
- `orchestra/run/status`, pause/resume, cancel, gate/human-input resolution, digest expansion, and
  evidence reads; and
- typed lifecycle events for Runs, Steps, Attempts, agents, gates, effects, promotion, and
  reconciliation.

Persisted semantic lifecycle state reuses Codex's extension-owned item path. The Codex fork adds one
namespaced Orchestra variant to `TurnItem::Extension`, persists it through the existing
`RolloutItem` policy, and maps it to `ThreadItem::Orchestra`. That public variant contains the closed
generated `OrchestraThreadItem` discriminated union for Run, Step, Attempt, gate, effect, and agent
items. Members contain stable identities, bounded display summaries, and references only. The union
is not a generic JSON container and does not duplicate authoritative Orchestra checkpoints or
evidence.

Each `OrchestraThreadItem` carries a stable item ID and monotonic semantic revision. Accepted
revisions append through the rollout path; `thread/read` collapses them by item ID and returns the
latest visible state. Item revision is distinct from the task-local replay sequence: the former
orders semantic updates to one entity, while the latter orders desktop delivery across task events.
Reducers reject a lower item revision and treat the same revision with different content as a typed
integrity error.

Raw Codex events are emitted, sequenced, snapshotted, and replayed only in the native task stream
that produced them. Parent streams contain bounded `OrchestraThreadItem` projections referencing
child task IDs; they never copy child conversation, tool, terminal, diff, or turn events. The
renderer subscribes to child task streams on expansion and may release those subscriptions when the
detail is no longer needed.

Together with native Codex task events, those lifecycle events form the renderer's complete
task-scoped replayable observation surface. They are not copied into the root model context. Rust
contributes the bounded Run Digest through Codex's existing extension-owned World State
snapshot-and-diff seam, so refreshes do not append transcript messages. Distilled child results,
actionable attention events, and requested graph expansions continue through the native task path.

Renderer expansion methods and the root model's native expansion tool are adapters over one bounded
`orchestra-core` Execution query service. App Server owns the generated renderer DTOs and Codex owns
the native tool schema; explicit conversions enter the shared query types. Both adapters enforce
consumer-specific output budgets while preserving the same stable selectors, authorization,
pagination, revision checks, and authoritative reads. Neither adapter rebuilds workflow queries
from Desktop projection state.

The shared MVP query service accepts only generated typed selectors for Runs, Steps, Attempts,
agents, gates, effects, outputs, evidence, and history pages. It does not accept a graph-query
language, arbitrary traversal, or renderer-defined projection. Selector growth follows concrete
Codex task or desktop product needs.

Workflow validation and invocation remain native task tool operations. A future workflow picker may
submit the equivalent recorded task action, but the Host protocol has no detached Run-start
operation.

Mutation requests include the target stable ID and its expected state revision. A stale revision
fails rather than applying to a newer state. Task-native Workflow invocation, resolving a gate,
dispatching an external effect, and other retry-sensitive mutations also require a host-scoped
idempotency key. The same App Server generator emits the Codex-plus-Orchestra TypeScript client and
JSON schemas. Client validation is diagnostic; Rust validates again and remains authoritative.

Wire ownership follows the fork boundary. Codex's `app-server-protocol` crate owns the added
`orchestra/*` and `host/*` request, response, error, event, replay-metadata, JSON Schema, and
TypeScript DTOs. `codex-orchestra-core` owns the durable workflow domain and behavior. Explicit,
tested conversions cross that seam; generated protocol values never become execution plans,
checkpoints, gate records, effect receipts, or recovery authority.

## Snapshots, replay, and Codex storage reuse

The Codex fork reuses its rollout-backed thread store and SQLite `StateRuntime` rather than adding a
Host-store subsystem. Durable Codex history and typed Orchestra lifecycle items use the canonical
extension-owned turn-item and rollout path without task-local transport sequence fields. Task-local
sequences, bounded raw replay tails, immutable Orchestra projection snapshots, attachment indexes,
and query projections extend `StateRuntime` and remain rebuildable. Losing that projection state
invalidates its cursors and requires fresh task snapshots; it never causes rollout history to be
rewritten. `host/snapshot/get` returns a versioned task snapshot composed from the existing Codex
`thread/read` result and the Orchestra execution projection, plus its content digest and last
included task-local sequence. Composition establishes
one replay barrier across both sections without copying the full Codex conversation into a second
durable projection. The Orchestra section contains the current inspectable Run Tree, Attempts,
statuses, gates, digests, and stable evidence references. It excludes full transition history and
large tool or evidence payloads; bounded authorized history, evidence, digest-expansion, and
content-addressed attachment methods retrieve those on demand.
`host/replay/subscribe` atomically establishes a barrier. The request names exactly one task and
supplies its optional task-local cursor. Events after the supplied cursor are replayed, then live
delivery continues without a gap. A duplicate sequence is harmless and reducers must be
idempotent. A missing or compacted cursor
returns `snapshot_required` with the oldest retained cursor; the client discards its projection,
verifies a new snapshot, and subscribes from that snapshot's barrier.

Codex history remains authoritative for visible conversation, and repository checkpoints remain
authoritative for workflow execution and effects. A task snapshot is a rebuildable composed read
model, not a third execution authority. Its Codex section is hydrated from `thread/read`; only its
Orchestra-specific section and replay metadata extend Desktop projection state. On host recovery,
Rust reconciles Codex `thread/read`, raw journal segments, and repository checkpoints before issuing
a new snapshot. It retains ambiguity rather than fabricating order or deduplication.

`orchestra-core` publishes typed transitions in-process only after authoritative checkpoint commits.
The Codex host assigns sequences, journals, and projects them through its existing persistence
interfaces. It records semantic lifecycle items through the rollout path and transport ordering in
the `StateRuntime` replay tail; Orchestra cannot mutate rollout records, replay tables, or product
snapshots. A checkpoint-to-journal crash gap is repaired during reconciliation.

For every active task, `StateRuntime` retains the newest verified Orchestra projection snapshot and
barrier metadata plus all later raw events. The host composes the Codex `thread/read` section on
demand; it never persists a full duplicate task snapshot. Covered segments may be compacted only
after a replacement Orchestra projection is durably verified. For a terminal task, the MVP retains
its newest Orchestra projection and compacts covered raw segments under a bounded Desktop-projection
budget. Eviction is oldest-terminal-task first and never removes repository execution evidence or
Codex history. Exact retention periods, diagnostic-export guarantees, and long-term archival policy
are deferred until product evidence requires them.

Only task-scoped Codex and Orchestra notifications use this replay path. Account, configuration,
update, and host-health state uses existing App Server read or snapshot methods followed by live
notifications. The MVP assigns those global notifications no durable cursor and defines no
cross-task total order.

## Backpressure and resource limits

The MVP has no acknowledgement frames or negotiated per-task delivery windows. A dedicated
bounded writer queue isolates the renderer data pipe from Codex ingestion, Orchestra checkpoint
commits, cancellation, and effect reconciliation. If that queue fills, the host sends
`reconnect_required` when possible and resets the presentation session while continuing to journal.
The renderer resumes from its in-memory cursor when available or requests a fresh snapshot after a
reload or expired cursor.

Replaceable projections such as token counters may be coalesced only when their schema declares
that behavior; raw journal events, terminal transitions, approvals, gates, effects, and responses
are never dropped or coalesced. Explicit acknowledgements and adaptive delivery windows are
deferred until measurements show that cursor replay plus the bounded queue is insufficient.

The pinned limits include frame and attachment size, nesting depth, string and collection length,
requests in flight, subscriptions, connection queue, request rate, replay batch, snapshot size,
history and evidence page size, and operation deadline. Larger artifacts use bounded chunked
attachment reads. Concrete caps are release configuration validated by integration tests rather
than protocol promises. Exceeding a
limit produces a typed rejection or session reset and is audit logged; the renderer cannot raise
host hard caps.

## MVP process transport

Electron main launches one signed `orchestra-host`, owns its inherited stdin/stdout/stderr, and
accepts one renderer data connection. It bridges opaque, length-prefixed frames between host
stdin/stdout and one transferred renderer `MessagePort`; stdout is protocol-only and stderr is
log-only. Main does not parse, cache, authorize, retry, or synthesize data-protocol messages. A new
renderer or host requires a new protocol session.

Pipe privacy is not operation authority. Rust still validates every request against capability,
identity, state revision, workspace scope, policy, rate, and size. Desktop snapshots and replay
remain required because a private pipe does not prevent renderer reloads, process crashes, or gaps
between durable Codex and Orchestra changes and their presentation.

Electron main and the host also receive a separate inherited, renderer-inaccessible privileged
control pipe. Its bounded, direction-authenticated messages carry only native confirmation
challenges and responses; they never carry ordinary backend requests. WebSocket or Unix-socket
transport is deferred until a concrete non-Electron client or browser-only development requirement
appears.

## Electron and renderer security boundary

Packaged windows use `sandbox: true`, `contextIsolation: true`, `nodeIntegration: false`, enabled
web security, no remote module, and a packaged custom origin. Navigation, unrequested windows,
permissions, downloads, and external protocols are denied by default and handled through explicit
allowlists. The production Content Security Policy allows packaged scripts and styles, denies
objects and framing, and has no renderer network `connect-src`; development exceptions are exact
and never ship. No remote content shares the application origin.

The preload is small, frozen, and schema checked. It exposes the single host transport plus
enumerated OS operations such as a native file picker. It exposes no raw Node, filesystem, process,
shell, socket, environment, or Electron API. Paths from OS dialogs are opaque grants scoped to the
request, not ambient filesystem authority.

The renderer is treated as potentially compromised by an injection flaw. Possession of its
MessagePort reaches only the data protocol and cannot access host process pipes, the privileged
control pipe, or Codex stdio. The host independently validates operation, capability, object
identity, state revision, workspace scope, rate, size, and current Codex/Orchestra policy. It never
trusts hidden UI state, disabled controls, renderer-supplied paths, claimed user identity, or a
renderer claim that an approval occurred.

Codex Permission Requests and high-risk external-effect confirmations use a host-issued,
single-use challenge bound to the exact request digest, state revision, consequence text, and
expiry. The host sends the challenge directly over the privileged control pipe. Electron main
renders the bound text in a native confirmation surface and returns the selected response over that
same renderer-inaccessible channel. The host consumes a valid response once. Ordinary conversation
input and workflow choices may originate in the renderer, but they remain typed inputs and cannot
grant undeclared capability or exceed organization policy. This design limits a renderer compromise
without pretending to protect against compromise of the signed Electron main process or the user's
OS account.

## Threat model and authorization invariants

The boundary is designed against hostile task/model output, malicious repository content, renderer
injection, stale or replayed UI actions, malformed or oversized frames, slow consumers,
confused-deputy path requests, pinned Codex schema drift, and host crash or restart.

The following invariants are mandatory:

- The renderer never reaches Codex task/turn services, the Orchestra runtime, repository
  checkpoints, or Codex persistence except through authorized protocol operations.
- A protocol session is private, local, least-privileged, revocable, and bound to one host instance;
  reconnect renegotiates it. Pipe privacy is not operation authorization.
- Only Rust decides whether a backend operation is permitted and whether a mutation can commit.
- Renderer compromise alone cannot approve a protected Codex capability or high-risk external
  effect, read arbitrary local files, open arbitrary network connections, or execute a process.
- Every accepted task-scoped Codex notification and authoritative Orchestra transition is journaled
  before projection; overload may disconnect presentation but may not discard authority events.
- Secrets, auth tokens, full environment values, and unredacted sensitive tool payloads never enter
  diagnostic exports or protocol errors.

Denial of service by the local user, a compromised Electron main process, a compromised host or
Codex binary, kernel compromise, and attacks requiring control of the user's macOS account are
outside this boundary. Packaging, signing, update provenance, and rollback address binary supply
chain risk separately.

## Compatibility and verification policy

The pinned `app-server-protocol` schemas, including its Orchestra additions, are wire-authoritative;
`codex-orchestra-core` remains execution-authoritative. A release uses the existing protocol
generator to emit one TypeScript client, JSON Schema set, golden frames, capability manifest, and
schema hash tuple. CI rejects hand-edited generated bindings and runs golden encode/decode,
wire/domain conversion, unknown-field/event, duplicate-request, stale-revision, snapshot barrier,
reconnect, compaction, overload, malformed-frame, authorization, and redaction tests. Parsers and
reducers are fuzzed with bounded inputs.

The MVP makes no compatibility promise across different host, renderer, or schema builds. Every
release regenerates and tests its bindings, schemas, frames, and capability manifest as one pinned
bundle. Store or snapshot changes require an explicit migration or invalidation decision for that
release; there is no general cross-version migration framework yet.

The pinned Codex revision and its generated protocol schema hash are release-manifest inputs.
Stable Codex operations are the baseline; experimental fields require exact-revision fixtures.
Unknown Codex events remain losslessly journaled, but a missing required native operation disables
the dependent capability rather than being emulated through private Codex storage, linked threads,
or a hidden sidecar. A system-Codex override is outside the MVP.

If an MVP release introduces a one-off Desktop-projection migration, it follows Codex's existing
state migration path and must preserve the previous projection until the replacement snapshot is
verified. It must never rewrite canonical rollout history or repository checkpoints. This protocol
does not promise compatibility with the patched-desktop/plugin experiment or its run formats.

## Consequences

The retained T3Code UI needs an adapter from generated host bindings into its existing reducer and
view-model shapes. The current Effect RPC/WebSocket contract, Node server, T3 event store, and
Node-owned Codex supervision are removed. The pinned Codex host mode owns Codex and Orchestra in one
Rust process. The single byte bridge is intentionally less convenient than adding Electron
handlers, but it keeps backend behavior versioned, replayable, testable, and Rust-authorized.

The renderer prototype must prove exact-bundle negotiation, existing-task hydration, task-native
Workflow invocation through an Orchestra skill/tool, canonical child lifecycle, streamed Codex and
Orchestra events, native approval confirmation, reload from snapshot plus cursor, and slow-consumer
session recovery without `apps/server`, a second App Server process, or detached Run start.
