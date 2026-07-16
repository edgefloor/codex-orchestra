# Issue #28 desktop boundary hardening

Status: the earlier direct renderer-to-host implementation was superseded after product review. The
coding-harness MVP now retains pinned T3Code `ecb35f75839925dd1ac6f854efeef5c9e291d11b`.

Implemented in the accepted product path:

- normal sandboxed T3Code renderer and local-server lifecycle retained;
- exact product Codex CLI injected through the existing provider driver;
- Codex/Rust remains authoritative for native task and workflow operations;
- provider and timeline regression coverage plus graceful desktop smoke cleanup.

Evidence:

- 35 focused provider/runtime tests and 40 timeline tests passed;
- desktop and web typechecks passed;
- full pinned web/server/Electron bundles passed;
- real Electron startup and graceful shutdown passed;
- provider-backed native subagent completed in the normal task UI.

The old MessagePort queues and inherited-fd confirmation were prototype behavior, not evidence for
the retained product path. Orchestra-specific confirmation isolation, lifecycle replay under a
provider crash, and hostile-renderer authorization tests remain open and must be implemented at
real T3Code/Codex seams rather than claimed from the discarded dashboard.
