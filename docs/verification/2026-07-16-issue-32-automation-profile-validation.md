# Issue 32 — Automation profile validation and preview

## Result

The MVP now validates a Symphony-compatible `WORKFLOW.md` through the active Codex task and previews its selected native Orchestra workflow in the retained T3Code task view. The renderer supplies only a task id, repository-relative profile path, fixture issue, and attempt; Codex derives the repository root and inherited policies.

Rust owns front-matter parsing, defaults, strict issue/attempt prompt rendering, secret-reference redaction, policy narrowing, workspace containment, restricted workflow compilation, input compatibility, allowed tracker effects, canonicalization, and the profile digest. `codex.command`, broader policy, unsafe roots, unknown required values, unsupported effects, and incompatible workflow inputs fail validation.

The pinned Codex App Server exposes generated `automation/validate` request and response types. T3Code routes its typed environment RPC through the task's existing Codex adapter and renders a bounded summary in the normal chat header dialog; the complete canonical profile remains collapsed by default.

## Automated evidence

- Root workspace: `cargo fmt --all --check`, 76 tests passed, and `orchestra-lifecycle doctor` passed with four skills and native capabilities.
- Fresh pinned Codex worktree at `f90e7deea6a715bbd153044af6f475eefa749177`: the integration patch applied; 55 Orchestra core tests, 7 extension tests, and the focused missing-profile containment test passed. The generated protocol includes `AutomationValidateParams`, `AutomationValidateResponse`, and `automation/validate`.
- Fresh pinned T3Code worktree at `ecb35f75839925dd1ac6f854efeef5c9e291d11b`: contracts and web typechecks passed, the production web build passed, 42 web tests passed, and 35 Codex adapter/runtime tests passed.
- The desktop request-shape tests assert that repository root and policy cannot be supplied by the renderer.

## Boundaries

- Validation uses fixture issue data and performs no tracker mutation.
- Inline tracker credentials are represented only by a digest; environment credentials retain only their variable name and digest.
- A real packaged Electron render remains human-only evidence and is not claimed here.
