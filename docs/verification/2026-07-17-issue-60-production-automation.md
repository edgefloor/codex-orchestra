# Issue 60 production Automation and steering verification

Verified on 2026-07-17 against the standalone hard-fork tuple:

- Orchestra Codex `1c6ed0131acc148772d878260c76963440057f40`
- Orchestra Desktop `abfd6f37758f4437e5d903ad4a2b5df418e28b26`
- canonical Orchestra core `eaab5f864442e5f83b8158cd12596125591cbfdc`

## Delivered

- `automation/start` opens or reattaches the active task's single resident Automation root, resumes
  a provider-fenced suspended root, reads bounded live Linear intake, and dispatches eligible Issues
  through native Codex tasks.
- `automation/steerIssue` authorizes the owning task and exact claim, persists a submitted receipt
  before native `AgentControl::send_input`, then persists a bounded delivered or failed outcome.
- Live comment, transition, and pull-request effects use the runtime-owned two-phase receipt path.
- The desktop no longer exposes the fixture runner as a production WebSocket method. Symphony Start
  sends only `threadId` and `profilePath`; running Issue claims expose bounded guidance and the latest
  durable steering receipt.
- Generated App Server bindings are sealed into the desktop fork with 701 files and digest
  `79cc318a85d863cc6c3b56d4a793874407a3e8cdff4a63ccc8b3671b6cf9c9e4`.

## Verification

- Codex: core Automation tests 26 passed; extension 16 passed and 1 explicit live-mutation test
  ignored; App Server protocol 268 passed before the final closed-request test was added, with that
  test and both generated-schema fixture tests passing independently; App Server Automation
  projection acceptance passed; all changed native crates compiled.
- Desktop focused: contracts 182 passed; server start/steer/auth/recovery tests 78 passed; Automation
  renderer tests 8 passed; all 15 workspace typechecks passed; static check reported 0 errors and 12
  existing warnings.
- Fused Product gate: source and generated provenance passed; web 1,325 passed; server 1,415 passed
  with 7 skipped; desktop 335 passed; all six sealed Bun/Zod evaluator tests passed; framed host
  handshake passed; two Electron launches reached backend-ready and main-window-created.

The live Linear mutation test remains opt-in and was not run because no user-selected disposable
Issue and mutation credential were supplied. Missing live credentials fail explicitly; fixture-only
execution remains available inside native tests but is absent from the production desktop RPC/UI.
