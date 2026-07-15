# Desktop native vertical slice — 2026-07-15

This record covers a fresh task created in the isolated-profile
`/Applications/ChatGPT-Orchestra.app` candidate. The user confirmed that this task was running from
that application while the task exposed and invoked the native Orchestra tool surface.

- Codex revision: `f90e7deea6a715bbd153044af6f475eefa749177`
- Plugin version: `0.2.0+codex.f90e7dee`
- Source revision: `f33405e9cc90e64d2d7323cc6a47ec17c3702f0c`
- Candidate SHA-256 before app-bundle signing:
  `7891a0289320cc578f66fff4ebfc0da76105ab4fcf133ab4bb7a7832a3034991`
- Installed signed CLI SHA-256:
  `7947058c5896640062cbd16c6d0717dc6269846f6162adcb86fcbbc070b08067`
- Run: `1784125288872-9a20ab5fea48`

## Automated and runtime evidence

| Observation | Result | Evidence |
|---|---|---|
| Patched bundle signature | passed | `codesign --verify --deep --strict` passed for `/Applications/ChatGPT-Orchestra.app` |
| Bundled CLI identity | passed | app-server process used `/Applications/ChatGPT-Orchestra.app/Contents/Resources/codex`; CLI reported `codex-cli 0.0.0` as expected for the pinned source build |
| Post-promotion source gates | passed | `cargo fmt --all --check`, 51 workspace tests, and `orchestra-lifecycle doctor` passed |
| Fresh pinned integration | passed | Updated core: 37 tests; extension: 3 tests; pinned Codex core Orchestra module: 7 tests; `codex-app-server` check passed at `f90e7deea6a715bbd153044af6f475eefa749177` |
| Restricted workflow validation | passed | Native `orchestra_validate` accepted `evals/workflows/native-vertical-slice.workflow.ts` |
| Provider-backed native run | passed | Native `orchestra_run` created run `1784125288872-9a20ab5fea48` under this task's parent thread |
| Parallel dependency-ready agents | passed | `inspect-runtime` and `inspect-tests` were concurrently `running` with canonical child task paths |
| Declared dependency output | passed | `inspect-runtime.findings` was persisted and supplied to the isolated `implement` step |
| Isolated implementation | passed | Four changed core files remained outside the target checkout before approval |
| Sandboxed shared-worktree check | passed | `cargo test --workspace`: 37 core and 14 lifecycle/scaffold tests passed |
| Approval pause and resume | passed | Run paused at `accept`; explicit user choice `accept` was persisted in `approvals/accept.json` and passed to native `orchestra_resume` |
| Promotion | passed | Runtime returned `status: completed`, `promotion: applied`; the verified patch appeared as unstaged target-checkout changes |
| Worktree cleanup | passed | Both run-owned `implement` and terminal `shared` worktrees were removed after promotion |
| Durable evidence | passed | Workflow, state, summary, outputs, approval, check result, skill manifest, implementation patch, and promoted patch remain under `.codex/orchestra/runs/1784125288872-9a20ab5fea48/` |

The promoted patch fixes the three findings produced by the inspection step: downstream step-output
template resolution, cancellation coordination for running check steps, and unique temporary paths
for concurrent atomic checkpoint writers. The run's test evidence includes dedicated regressions for
each behavior.

## Human UI evidence

| Observation | Result | Notes |
|---|---|---|
| Fresh patched-app task | passed | User explicitly confirmed this task was running from `ChatGPT-Orchestra.app` |
| Five native Orchestra tools | passed | This task exposed and invoked exactly `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, and `orchestra_cancel` |
| Canonical V2 task paths and parallel activity | passed | Native child paths and concurrent step residency were observed during the run |
| Explicit model/reasoning rendering | pending | Plan values were verified through the native tool result, but dedicated UI rendering was not separately observed |
| Approval interaction | passed | The user supplied `accept` after the native pause, and resume recorded that exact choice |
| Cancellation timing | pending | No provider-backed cancellation run was performed |
| Rejection leaves target unchanged | pending | Not exercised in this run |
| Promotion conflict preserves target and resumes | pending | Not exercised in this run |
| Transcript-free recovery from a fresh task | pending | Not exercised in this run |
| Installed-cache identity during self-hosting | pending | Not exercised in this run |

## Verdict

The patched desktop candidate passes the fresh-task native-tool and provider-backed vertical-slice
acceptance criteria, including approval-gated promotion and run-owned worktree cleanup. Stage 4
recovery, cancellation, conflict, rejection, and installed-cache identity checks remain pending.
