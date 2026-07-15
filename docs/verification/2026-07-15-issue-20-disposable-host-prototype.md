# Issue #20 disposable desktop-host prototype — 2026-07-15

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

Focused results: 5 Rust unit tests and 2 renderer reducer tests passed before the end-to-end trace.

## Gaps that remain pending

- The fixture protocol types have not yet been absorbed into the pinned Codex
  `app-server-protocol` generator or handled by the real `MessageProcessor`.
- Replay tails and projection snapshots are in memory here, not the pinned Codex SQLite
  `StateRuntime`; the restart case therefore proves typed invalidation/recovery, not durable replay.
- The harness adapts T3Code's pure reducer/snapshot concepts but does not build the retained React
  components or Electron application.
- The parent/child fixture uses canonical Codex IDs and separation rules, but no provider-backed V2
  child was created during this deterministic run.
- fd 3 plumbing and API separation are proven, but main and renderer modules share one Node harness
  process. Actual Electron renderer inaccessibility, native macOS confirmation UI, signing,
  packaging, CSP, preload hardening, and hostile-renderer testing remain unobserved.
- The fixture World State replacement proves reducer semantics, not the actual Codex extension-owned
  prompt contribution API.

These are product-fork absorption tasks, not reasons to add a fallback backend or alternate store.

## Verdict

**Keep with changes.** Keep the retained T3Code product components, pure reducer/cache concepts,
Electron lifecycle shell, and the direct extended App Server seam. Replace Effect RPC/WebSocket,
Node-owned Codex supervision, and T3 persistence with generated bindings and Rust-authorized task
projection/replay. Adapt lifecycle rendering around stable typed items and subscribe to child tasks
only when expanded.

The architecture is not falsified by transport, reducer, recovery, context-economy, or privileged
channel behavior. Issue #20 should remain open until the same trace runs against the actual pinned
Codex protocol/`StateRuntime` integration and retained Electron/React fork; this disposable fixture
should then be deleted and its validated reducer rules absorbed.
