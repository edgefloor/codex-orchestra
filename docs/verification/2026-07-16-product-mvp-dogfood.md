# Product MVP dogfood verification

Date: 2026-07-16

## Tuple

- Orchestra: `0.2.0-dev`
- Codex: `f90e7deea6a715bbd153044af6f475eefa749177`
- T3Code: `ecb35f75839925dd1ac6f854efeef5c9e291d11b`
- Target: `aarch64-apple-darwin`
- Manifest: `cccf6dccf9ad5ef3f413ad36fa1eb51b75ad43f8b0b1a6fd275e3da5e353c71e`
- Output: `target/orchestra-product-mvp/`

## Automated evidence

- `scripts/product-dev-build.sh /tmp/orchestra-mvp-normal-sources target/orchestra-product-mvp`
  passed end to end.
- The pinned Orchestra-enabled Codex CLI completed a real App Server `initialize` handshake with the
  exact sealed manifest.
- The complete retained T3Code web, server, and Electron application built and typechecked.
- 35 provider/runtime tests and 40 task-timeline tests passed.
- The real Electron process reached `backend ready` and `main window created`, then shut down
  gracefully without leaving its backend listener orphaned.
- Five fresh-process evaluator tests passed against the packaged Bun worker.

The product build seals the Codex CLI, app-server protocol JSON and generated TypeScript, desktop
main/preload/server/renderer, and evaluator worker into one manifest.

## Manual observation

`scripts/product-dogfood.sh /tmp/orchestra-mvp-normal-sources target/orchestra-product-mvp`
launched the exact tuple and left it running. Visual inspection confirmed the normal T3Code project,
task, chat, model, runtime, workspace, and settings surfaces. In a temporary empty repository, a
normal task spawned exactly one native subagent and returned `NATIVE_SUBAGENT_READY`. Process
inspection confirmed the provider command was
`target/orchestra-product-mvp/codex app-server`; provider events contained the native
`collabAgentToolCall`. No workflow was started in this repository and Orchestra was not used to
self-build.

## MVP boundary

This is an unsigned Apple-silicon development product for local dogfood. Signing, notarization, x86
packaging, production updates, Orchestra-specific lifecycle UI, recovery through a killed provider,
and the renderer-inaccessible privileged confirmation channel remain integration work.
