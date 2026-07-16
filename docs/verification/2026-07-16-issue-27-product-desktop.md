# Issue #27 retained Product desktop verification — 2026-07-16

The exact T3Code revision `ecb35f75839925dd1ac6f854efeef5c9e291d11b` was prepared with
`scripts/t3code-integration.sh`. An earlier replacement workflow dashboard was rejected and removed.
The accepted MVP retains T3Code's complete normal application and changes only the Codex binary
selected by its existing provider driver.

| Observation | Result | Evidence |
|---|---:|---|
| Normal T3Code shell retained | passed | 4,329-module renderer shows projects, tasks, chat, settings, models, workspaces, and the native task timeline |
| Minimal product seam | passed | `CodexDriver` selects the sealed CLI and verifies its exact release-manifest identity; the existing provider runtime and timeline are extended without replacing desktop, preload, server, or web entrypoints |
| Exact fork typechecks | passed | Contracts, server, web, and desktop complete `tsgo --noEmit` on a clean pinned checkout |
| Provider and timeline tests | passed | 39 provider/runtime tests cover exact Product compatibility, stable native lifecycle projection, and bounded queries; 104 retained-shell/timeline tests cover lifecycle presentation and replay data |
| Production bundles | passed | Full web, server, Electron main, preload, and preview-preload bundles build |
| Real desktop startup | passed | Isolated T3Code home reaches `backend ready` and `main window created`, then shuts down without an orphan listener |
| Native subagent dogfood | passed | Packaged UI created a normal task; sealed Codex emitted `collabAgentToolCall`; child and parent returned `NATIVE_SUBAGENT_READY` |
| Lifecycle recovery | passed | The provider hydrates rollout/StateRuntime-backed replay through native `thread/read` at session start and after turns, with stable event IDs deduplicated per provider lifetime |
| Context-bounded detail | passed | The retained task timeline shows bounded lifecycle summaries; expanding a row requests step and child references, while evidence remains unloaded until a separate bounded reference-only query |
| Real lifecycle dogfood | passed | A packaged desktop task ran a read-only native check as run `1784234522129-ea7f5fe8fdff`; the normal Work Log rendered `Orchestra workflow · completed · invoked`, expanded the `repository` step, and loaded evidence references on demand |
| Reload and provider restart | passed | After quitting and restarting Electron, the task-scoped lifecycle activity remained in the normal timeline; reconnecting the pinned provider replayed the same stable event without a duplicate row |
| Product turn settlement | passed | The pinned Product's authoritative idle transition settles the existing T3 turn exactly once, while an eventual native `turn/completed` is deduplicated; the task returned from Working and accepted a follow-up normally |
| Visible compatibility failure | passed | A missing Product manifest, mismatched manifest SHA, or missing `orchestra/query`/`orchestra/threadItem` capability fails the Codex session with an explicit provider error |

The Product build seals the Codex CLI plus desktop main, preload, local server, renderer, generated
protocol, and evaluator, then runs the native protocol and real Electron startup smokes.

The live pass exposed and fixed two interoperability defects that mocks had missed: Rust `Option`
fields in `thread/read` are encoded as `null`, and a follow-up request cannot be awaited inside the
serialized App Server notification dispatcher. The retained provider now accepts those nullable
protocol fields and forks lifecycle refreshes onto its managed runtime scope. Task-scoped lifecycle
entries use informational timeline tone so they are not mistaken for transient reasoning and hidden.

The privileged native confirmation channel and its user-facing copy are intentionally outside this
issue; issue #28 owns that confirmation surface. Large evidence contents also stay outside the
renderer by design: this surface exposes only bounded references and hashes.
