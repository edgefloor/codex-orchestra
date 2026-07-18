# Issue #66 native Electron shell acceptance

## Published fork identity

- Orchestra Desktop source under review: `fa24bf561bcd807e71c3aa786c83c0d8277307b6`, tree
  `622c518832281fb157283a62b54090e57fe2e53b`.
- Orchestra Desktop evidence commit: `7417e043938e07183037ec251eac999be8baed81`, tree
  `eb6506d24a0e48328e396640026a4af98452ec62`.
- Orchestra Codex pin: `7fddc000e0531657002ec4fac59f5edbabb4695b`, tree
  `b1306b462645553d975b21f872ddc4ca75310f20`.
- Both standalone forks are published on their repository `main` and
  `codex/orchestra-bootstrap` branches.

## Native-shell evidence

The Desktop harness launches the production Electron main, preload, server, and web artifacts with
isolated homes. It creates the project and task through the native backend and drives the production
Browser panel's real `<webview>` guest. The sealed manifest records 28 passing runtime assertions,
including native route reload recovery, Browser history/reload/failure/recovery, observed guest
security preferences, an actual rejected invalid-partition attachment, screenshot capture, and
owned process-group/listener cleanup.

Two clean captures against the exact source produced the same semantic assertion set. The 1024×768
screenshot was byte-identical across runs. The 1440×900 screenshot changed only because the visible
address bar contains a freshly allocated loopback port. Both final real Electron screenshots were
directly inspected and record native-backed project/task state at desktop and narrow widths.

Evidence:

- `docs/acceptance/orchestra-native-shell/manifest.json` in the Desktop fork.
- `native-browser-1440x900-dark.png` SHA-256
  `b84ec2415162831bfd15dea855e4c05c91a3818ba5e46ce47a5c96fce9eb7026`.
- `native-workspace-1024x768-dark.png` SHA-256
  `828a584ce17c0a8d7781000d706a2c615d07299fcdb45bc566f69bb7001aff51`.

## Verification

Desktop source verification before evidence publication passed:

- independent Spec review and independent Standards review, with no unresolved P1/P2 findings;
- 51 Desktop test files / 342 tests;
- 183 Web test files / 1,432 tests;
- 161 Server files passed, 2 skipped; 1,415 tests passed, 7 skipped;
- all package typechecks, production Desktop build, Desktop smoke, retained-capability verifier, and
  repository check (nine retained warnings, zero errors);
- focused native-shell/verifier tests and two clean native captures.

Coordinator verification after advancing the public Desktop pin passed:

- fresh-clone `scripts/product-source-prepare.sh` and `scripts/product-source-verify.sh`;
- `cargo test --workspace`: 124 active tests passed; six pinned evaluator-worker tests delegated;
- `scripts/evaluator-test.sh`: all six Bun 1.3.14 / Zod 4.4.3 sealed-worker tests passed;
- `cargo fmt --all -- --check`, `git diff --check`, and lifecycle/plugin doctor.

The coordinator suite exposed two stale Cycle 5 derived identities: exact fork literals in the
lifecycle scaffold and the plugin Codex version suffix still named `973a40f7`. They were advanced to
the already-sealed `7fddc000` Codex identity. No validation rule or documented standard changed.

## Remaining boundary

This evidence is deterministic and native, but the visible provider state truthfully remains
unauthenticated. A live authenticated Codex-provider/MCP broker dogfood observation remains open in
#66. Signed/notarized physical-machine distribution remains #56.
