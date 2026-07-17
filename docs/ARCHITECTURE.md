# Orchestra product-fork architecture

This is the issue #12 synthesis of accepted decisions #11, #13, #16, #19, and #21 and the research
and prototype evidence linked from issue #10. It defines ownership and implementation boundaries for
the macOS coding-harness MVP; detailed domain language remains in `CONTEXT.md` and decisions remain in
the concise ADRs.

## Product shape

Orchestra is a long-lived fork of Codex plus a long-lived T3Code-derived product fork. The MVP keeps
the normal T3Code Electron, React, and local-server application and points its existing Codex
provider at one exact Orchestra-enabled Codex CLI. The independently versioned Authoring plugin
supplies skills, documentation, and configuration, but never native execution.

The retained T3Code server is a UI BFF and provider adapter, not an agent or workflow authority. It
supervises `codex app-server` and maps the native protocol onto T3Code's existing task/chat API. There
is no second agent service, external scheduler, detached Run start, Host-store, or alternate agent
dispatcher.

## Runtime ownership

| Concern | Authority |
| --- | --- |
| Conversation, turns, typed lifecycle history | Codex task and rollout |
| Scheduling, Steps, Attempts, gates, effects, recovery | native Orchestra Rust runtime |
| Execution checkpoints and validated outputs | target repository `.codex/orchestra/runs/` |
| Task-local replay tail and rebuildable projections | existing Codex `StateRuntime` |
| Agent creation, status, steering, wait, cancellation | owning task's native V2 `AgentControl` |
| Desktop protocol authorization | Rust host using Codex policy and Orchestra authority |
| Window, local-server launch, updates, native dialogs | Electron main |
| UI RPC, provider supervision, task/chat projection | retained T3Code server |
| Task, subagent, and Run rendering | retained/adapted React renderer |

A task has at most one nonterminal Root Run. A workflow invocation is a resident native action in that
task and returns only when the Run terminates or durably suspends. Child Runs use canonical parent-linked
V2 children; interruption fences Attempts and suspends rather than leaving background work.

## Invocation and observation

1. A user or agent invokes Orchestra through the parent Codex task.
2. Rust resolves inputs and skills, compiles the closed workflow graph, and records a Workflow artifact.
3. The native runtime schedules ready Steps through the task's `AgentControl`.
4. Codex rollouts receive bounded typed Orchestra lifecycle item revisions.
5. Repository checkpoints record authoritative execution state, outputs, gates, evidence, and effects.
6. `StateRuntime` records only task-local delivery cursors, replay tails, and rebuildable projections.
7. The T3Code provider adapter hydrates `thread/read`, composes a Task snapshot at one replay barrier,
   and projects it through the normal task/chat surface.
8. The root model receives a replaceable bounded Run Digest through Codex World State. Raw child detail
   stays in the child task and is read only on targeted expansion.

The fixed typed Execution query service is shared by native task-tool and App Server adapters. Each
consumer has its own output budget, but identity, authorization, selection, and pagination semantics
are common. There is no general graph-query language.

## Protocol and desktop boundary

Codex App Server remains the sole native agent/workflow protocol and is extended with Orchestra
methods, lifecycle notifications, capability negotiation, snapshots, replay, and bounded queries.
For the MVP, T3Code's existing server is the client of that protocol and the renderer keeps T3Code's
existing local UI transport:

```text
sandboxed T3Code renderer -> retained local T3Code server -> pinned `codex app-server`
```

The Product integration seam is intentionally one environment-controlled provider-binary override; it does not
replace the T3Code shell with a workflow dashboard. Rust still authorizes Orchestra operations, and
the local server cannot manufacture a detached Run. Only task-scoped notifications have durable
cursors; there is no global replay sequence. Slow consumers reconnect through snapshot plus
task-local replay.

The retired issue #20 direct MessagePort/framed-host design remains only as a dated historical
evidence record, not product transport or runnable source. A renderer-inaccessible control channel is
still required before exposing an Orchestra-specific privileged native confirmation, but that
channel will be added at the narrowest retained T3Code seam when the confirmation UX is implemented.

## Compilation and validation

Rust resolves, parses, and lowers the pinned `.workflow.ts` graph without executing workflow source as
JavaScript. The supported Agents SDK surface is an inert authoring subset; Runner, providers, sessions,
tracing, realtime, callbacks, and SDK execution are excluded.

A pinned one-request worker evaluates only the schema-only exact-Zod Validation bundle. Canonical JSON,
artifact hashes, provenance, deterministic issues, bounded IPC/time, and crash classification follow
`WORKFLOW-COMPILATION.md`. Repository workflow definitions are trusted like local build/test code for
the coding-harness MVP; hostile-code XPC isolation is production hardening rather than an MVP claim.

## Failure and recovery

- Renderer reload: reconnect to the retained local server, hydrate the task, then resume task-local
  projection delivery.
- Codex provider crash: the retained provider supervisor restarts it; Orchestra fences the previous
  Execution lease, reconciles repository checkpoints and Codex history, then resumes explicitly
  through the owning task.
- Expired cursor: discard rebuildable projection state and request a fresh snapshot.
- Child interruption: preserve evidence, best-effort cancel descendants, and durably suspend the Run.
- External effect ambiguity: reconcile its recorded receipt before retrying.
- Promotion conflict: preserve the target checkout and retained worktree for explicit resume.
- Product mismatch or unsupported artifact tuple: fail closed without rewriting history.

## Threat model

The renderer, local UI server, workflow outputs, child-agent text, repository content, and protocol
inputs are untrusted with respect to Orchestra authority. Rust validates identities, limits, state
transitions, and protected effects; transport privacy does not grant authority. Neither the renderer
nor T3Code server may create detached Runs, forge committed gates, or mutate repository checkpoints.

Crashes, duplicate delivery, stale hosts, expired cursors, partial writes, ambiguous effects, and
malformed outputs are expected operational faults. Atomic checkpoints, stable identities, semantic
revisions, task sequences, Execution-lease fencing, effect receipts, and fail-closed compatibility
make them recoverable. Product artifacts and upstream changes are supply-chain inputs controlled by
exact fork and upstream pins, signed Release manifests, reproducible gates, and rollback.

For the coding-harness MVP, repository workflow definitions are trusted at the same level as local
build scripts. The exact-Zod worker is bounded operationally but is not a sandbox for malicious code.
That limitation must remain visible until a later product requirement funds hostile-code isolation.

## Release and compatibility

Codex, the desktop, native host, generated protocol, Orchestra runtime, evaluator, schemas, capabilities,
and effective limits ship as one architecture-specific signed Product release. The sealed Release
manifest records exact source and artifact identities. Native components update and roll back together;
the Authoring plugin lifecycle is independent.

The field-by-field and behavior-by-behavior Symphony mapping is maintained in
[`SYMPHONY-COMPATIBILITY.md`](SYMPHONY-COMPATIBILITY.md).

Updates preserve the user-owned Codex home, canonical rollouts, and repository Run checkpoints.
Rebuildable projections use versioned generations. Direct Developer ID distribution, separate arm64
and x86_64 builds, full-app updates, rollback barriers, signing, notarization, notices, SBOM, and fork
sync follow ADR-0017. Every dependency, including the updater, is pinned exactly in an actual release.

## Maintained fork source model

The disposable evaluator, host, MessagePort, Electron, and reducer harnesses were removed after their
claims gained native Product parity. Their dated verification records remain historical evidence.
The reviewed Product source lives directly in the public `orchestra-codex` and
`orchestra-desktop` hard forks. Product preparation clones immutable fork commits and verifies their
upstream bases plus cross-repository runtime and protocol identities. The coordinator preserves no
patch assembly, fixture protocol, duplicate store, Node workflow authority, detached tool, or
alternate scheduler.

## Decision reconciliation

ADR-0004 is superseded by ADR-0009, and ADR-0009 is superseded by ADR-0010. ADRs 0011 through 0017
then refine the long-lived fork, verified promotion, skill contract, unified state, desktop protocol,
compilation/validation, and release boundary without replacing ADR-0010's native Rust runtime. No
accepted decision authorizes the earlier separate Host store, child App Server process, detached
scheduler, runtime JavaScript compiler, or temporary-fork exit strategy; any prose implying those
shapes is historical evidence rather than current architecture.

## Context-economy acceptance rule

Every implementation slice must identify its canonical authority, bounded default projection, hard
budget, stable expansion identities, and targeted expansion path. Parent tasks, model prompts,
renderer snapshots, and product diagnostics must not copy detail merely for convenience.

## Implementation frontier

The production implementation decomposition is:

1. [#22](https://github.com/edgefloor/codex-orchestra/issues/22) — reproducible pinned forks and Release manifest.
2. [#23](https://github.com/edgefloor/codex-orchestra/issues/23) — generated App Server Orchestra protocol.
3. [#24](https://github.com/edgefloor/codex-orchestra/issues/24) — rollout lifecycle, `StateRuntime` replay, and snapshots.
4. [#25](https://github.com/edgefloor/codex-orchestra/issues/25) — resident invocation, native V2 children, World State, and queries.
5. [#26](https://github.com/edgefloor/codex-orchestra/issues/26) — Rust compilation and exact-Zod worker.
6. [#27](https://github.com/edgefloor/codex-orchestra/issues/27) — retained Electron/React product integration.
7. [#28](https://github.com/edgefloor/codex-orchestra/issues/28) — recovery, backpressure, confirmations, and hardening.
8. [#29](https://github.com/edgefloor/codex-orchestra/issues/29) — packaging, updates, migration, rollback, and release gates.
9. [#30](https://github.com/edgefloor/codex-orchestra/issues/30) — prototype retirement after native parity.

These issues may refine local code shape, but they must not reopen ownership, state authority,
invocation residency, native V2 children, protocol direction, Product compatibility, or the absence
of alternate control planes.
