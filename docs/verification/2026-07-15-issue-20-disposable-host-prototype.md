# Issue #20 disposable desktop-host prototype — 2026-07-15

> Superseded product conclusion (2026-07-16): the harness remains valid evidence about a possible
> direct byte bridge, but the replacement workflow dashboard built from it failed product review.
> The MVP retains the normal T3Code application and its existing Codex provider adapter. Nothing in
> this record is production evidence for the current desktop path unless repeated there.
>
> Retirement note (2026-07-17): issue #30 removed the disposable source and commands after native
> Codex and retained T3Code tests superseded the applicable assertions. This dated record is preserved
> as unsupported legacy evidence and is not a current reproduction guide.

This record covers the deterministic architecture gate for
[issue #20](https://github.com/edgefloor/codex-orchestra/issues/20). It tests the retained
T3Code-derived client boundary against one direct, length-framed Rust host process without paid
model calls, a Node backend, a second App Server, WebSockets, or detached Run creation.

- Orchestra base revision: `f7c9293b7858844059c09aca8264dd5a8d27d1e5`
- Codex revision: `f90e7deea6a715bbd153044af6f475eefa749177`
- T3Code revision: `ecb35f75839925dd1ac6f854efeef5c9e291d11b`
- Host build ID: `orchestra-host-issue-20-prototype`
- Renderer build ID: `t3code-ecb35f75-orchestra-prototype`
- Protocol schema ID: `codex-app-server+orchestra-prototype-v1`
- Protocol schema SHA-256: `e87ff11b6d0c02e466b3b225c67faff9c84c90b8d04b7f4ee5bcc086e89c20a5`
- Snapshot schema ID: `task-snapshot-prototype-v1`

## Prototype question and boundary

Can T3Code's reusable snapshot/reducer model sit above generated extended App Server bindings while
normal Codex task detail remains native, Orchestra state is a bounded typed projection, and Electron
main remains an opaque process/byte bridge?

The host is intentionally an in-memory fixture. Its `thread/read` payload models the pinned App
Server result, and its projection/journal models the intended `StateRuntime` extension. The harness
uses a real child process, stdin/stdout, inherited fd 3, and Node `MessageChannel`; it does not claim
that fixture storage is the production Codex implementation.

## Automated evidence

Command: `scripts/desktop-host-prototype.sh`

| Observation | Result | Evidence |
|---|---:|---|
| One host and one renderer connection | passed | Electron-main harness spawned one Rust child and transferred only one `MessagePort` data endpoint to the renderer adapter |
| Bounded framing and stdout discipline | passed | 4-byte big-endian bounded UTF-8 JSON frames round-tripped; oversized/truncated input is typed; diagnostics appeared only on stderr |
| Exact compatibility tuple | passed | Exact tuple initialized with no task cursors; changed renderer build returned terminal `incompatible_bundle` |
| Existing-task hydration | passed | `thread/read` hydrated the parent task and `host/snapshot/get` composed the task with an Orchestra projection at barrier 1; SHA-256 verified |
| Replay to live fixture | passed | The in-memory host recorded a parent subscription at cursor 1 and delivered the next sequence 2 update; real `StateRuntime` atomicity remains pending |
| Typed lifecycle revision | passed | Stable `orchestra-step-1` advanced from semantic revision 1 to 2; lower revision was ignored and divergent same revision failed integrity validation |
| Parent/child projection contract | partial | Parent fixture referenced `task-child` without child detail and expansion performed `thread/read` plus child-only replay; native V2 creation was not exercised |
| Run Digest reducer contract | partial | Renderer `orchestra.runDigest` changed from revision 1 to 2 through a keyed `replace`; the Codex World State contributor was not exercised |
| Shared bounded query contract | partial | Two fixture adapters received identical selection bytes and unauthorized scope was denied; the real `orchestra-core` query service was not called |
| Renderer reload | passed | A replacement renderer port hydrated snapshot barrier 2 and subscribed from that cursor |
| Host restart/expired cursor | passed | New host instance rejected the old cursor with `snapshot_required`; a fresh snapshot recovered at its barrier |
| Overload policy branch | partial | Eight bounded-tail events were recorded before a typed reset instruction; a genuinely blocked host writer queue was not exercised |
| Unknown event and redaction | passed | Unknown sequenced event remained in bounded diagnostics; exported token field was `[REDACTED]` |
| Privileged confirmation plumbing | partial | Challenge traversed inherited fd 3 to the main-bridge module and the renderer client API received only its `MessagePort`; actual renderer-process inaccessibility still requires Electron process isolation |

## Retained Electron process follow-up

`scripts/electron-host-prototype.sh /path/to/pinned-t3code` uses the exact pinned T3Code checkout and
its Electron 41.5.0 runtime. A real hidden `BrowserWindow` runs with `sandbox: true`,
`contextIsolation: true`, and `nodeIntegration: false`; its renderer confirmed that `process` and
`require` are absent. Electron main transfers one `MessagePort`, owns the Rust child stdio, and alone
answers the inherited fd 3 confirmation challenge. The renderer successfully initialized, read
`task-parent`, and received the confirmation result through protocol frames.

The pinned T3Code desktop smoke baseline also passed before the bridge test. This closes the
renderer-process-isolation observation, but does not exercise retained React lifecycle components or
a genuinely blocked host writer queue.

Focused results: 5 Rust unit tests and 2 renderer reducer tests passed before the end-to-end trace.

## Gaps that remain pending

- The fixture protocol types have not yet been absorbed into the pinned Codex
  `app-server-protocol` generator or handled by the real `MessageProcessor`.
- Replay tails and projection snapshots are in memory here, not the pinned Codex SQLite
  `StateRuntime`; the restart case therefore proves typed invalidation/recovery, not durable replay.
- The harness does not yet render through the retained React lifecycle components.
- The parent/child fixture uses canonical Codex IDs and separation rules, but no provider-backed V2
  child was created during this deterministic run.
- fd 3 and renderer process separation are proven in Electron. Native macOS confirmation UI,
  signing, packaging, final CSP/preload hardening, and hostile-renderer testing remain unobserved.
- The fixture World State replacement proves reducer semantics, not the actual Codex extension-owned
  prompt contribution API.

These are product-fork absorption tasks, not reasons to add a fallback backend or alternate store.

## Verdict

**Historical prototype result only.** Keep the transport/reducer findings as disposable evidence,
but do not use them to replace T3Code's normal task/chat product. The accepted MVP retains T3Code's
server and Codex provider supervision while keeping Codex/Rust authoritative for task and workflow
semantics.

The architecture is not falsified by the fresh pinned Codex overlay build, retained Electron smoke,
transport, reducer, recovery, context-economy, process isolation, or privileged-channel behavior.
The remaining real protocol, `StateRuntime`, rollout, native-child, World State/query, retained-React,
and blocked-writer work is production fork absorption, not another architecture decision. Issue #12
decomposes it into implementation slices; this disposable fixture is deleted only after those native
tests supersede its evidence.
