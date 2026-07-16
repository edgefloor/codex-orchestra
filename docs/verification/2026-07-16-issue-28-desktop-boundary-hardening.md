# Issue #28 retained-path recovery and hardening

Status: verified in the retained pinned T3Code `ecb35f75839925dd1ac6f854efeef5c9e291d11b`
and Product Codex path.

## Implemented boundary

- T3Code remains an adapter over Product Codex App Server; it does not own Orchestra runtime state.
- A provider exit records `session/exited`, marks the session unusable, and clears its active turn.
- The next normal task action starts Product Codex with the previous native resume cursor. No
  detached Orchestra Run or alternate recovery control plane is introduced.
- Replay payloads are capped at 64 events. Projection text accepts the native 4096-byte body budget
  plus its optional truncation marker; oversized and malformed payloads are rejected by the shared
  contract.
- Provider diagnostics are bounded and redact token, password, secret, API-key, and bearer-shaped
  values before reaching UI-visible state.

## Real application evidence

In the normal Electron task UI, Product Codex PID `58972` completed a task action and was then sent
`SIGTERM`. T3Code emitted `session/exited` for the failed protocol stream. The next user message
started Product Codex PID `61326` with the retained native resume cursor and returned the requested
`AFTER_CRASH_2` response. The hydrated Work Log still contained exactly one prior Orchestra
lifecycle entry; recovery did not duplicate the projection.

This abrupt process termination also exercises a truncated stdout/protocol stream. Recovery occurs
at the native App Server session boundary instead of attempting to repair partial JSON writes in
the adapter.

## Fault and authority coverage

- Codex `StateRuntime` tests prove persisted replay and idempotent duplicate event revisions.
- Automation lease epoch and revision tests reject stale provider results after pause.
- App Server protocol tests reject client-supplied lease epochs and reconciliation outcomes.
- The renderer has no privileged Orchestra confirmation or decision RPC. Native Codex remains the
  authority; a direction-authenticated confirmation channel should be added with the first concrete
  privileged UI, not exposed speculatively in this MVP.
- No ACK/window protocol, alternate transport, scheduler, daemon, or recovery dashboard was added.

## Verification

- pinned patch verification passed against a clean T3Code worktree;
- 104 web tests and 74 server/provider tests passed;
- contracts, server, and web typechecks passed;
- the full pinned web/server/Electron build passed before the final recovery-only adjustment;
- the final recovery adjustment was rechecked by the clean test and typecheck suites and by the
  real Electron crash/recovery exercise above.
