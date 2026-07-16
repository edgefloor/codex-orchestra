# Issue #27 retained Product desktop verification — 2026-07-16

The exact T3Code revision `ecb35f75839925dd1ac6f854efeef5c9e291d11b` was prepared with
`scripts/t3code-integration.sh`. An earlier replacement workflow dashboard was rejected and removed.
The accepted MVP retains T3Code's complete normal application and changes only the Codex binary
selected by its existing provider driver.

| Observation | Result | Evidence |
|---|---:|---|
| Normal T3Code shell retained | passed | 4,322-module renderer shows projects, tasks, chat, settings, models, workspaces, and the native task timeline |
| Minimal product seam | passed | Patch changes `CodexDriver` only; `ORCHESTRA_CODEX_PATH` supplies the sealed CLI without replacing desktop, preload, server, or web entrypoints |
| Exact fork typechecks | passed | `@t3tools/web` and `@t3tools/desktop` complete `tsgo --noEmit` |
| Provider and timeline tests | passed | 35 server provider/runtime tests and 40 web timeline tests pass |
| Production bundles | passed | Full web, server, Electron main, preload, and preview-preload bundles build |
| Real desktop startup | passed | Isolated T3Code home reaches `backend ready` and `main window created`, then shuts down without an orphan listener |
| Native subagent dogfood | passed | Packaged UI created a normal task; sealed Codex emitted `collabAgentToolCall`; child and parent returned `NATIVE_SUBAGENT_READY` |

The Product build seals the Codex CLI plus desktop main, preload, local server, renderer, generated
protocol, and evaluator, then runs the native protocol and real Electron startup smokes.

Pending: expose Orchestra-specific lifecycle items in the retained timeline, exercise workflow
recovery through the provider supervisor, inspect large referenced evidence, and implement/review
the privileged native confirmation channel and copy.
