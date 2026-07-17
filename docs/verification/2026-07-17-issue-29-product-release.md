# Issue #29 — Product release, update, and recovery

Date: 2026-07-17

## Implemented contract

- The retained T3Code packager and `electron-updater` path remain the only desktop distribution
  authority. The updater is pinned exactly to 6.8.3 and differential downloads are disabled, so the
  pinned Codex fork, evaluator, Product helper, server, renderer, and desktop main move as one
  full-application release.
- The macOS packager emits separate arm64 and x86_64 candidates, embeds the sealed Product tuple,
  requires Apple signing and notarization inputs for publishable output, and provides a deliberately
  non-publishable unsigned rehearsal mode.
- Candidate sealing requires both architectures and non-waivable repository, protocol, evaluator,
  state, Codex-home, distribution, machine, licensing, and human-evidence gates. Every archive,
  manifest, gate record, license, dependency inventory, notice, SPDX document, and corresponding
  source artifact is content-addressed in the release record.
- Publication requires the exact sealed asset identities, immutable artifact and evidence status,
  and content-addressed update metadata plus its signature. A boolean-only publication assertion is
  insufficient.
- The Product install state records staging, side-by-side projection generations, first-launch
  activation and commit, two retained predecessors, one bounded automatic rollback, explicit
  reverse transitions, snapshot-schema rollback barriers, and transition receipts under an
  exclusive Codex-home maintenance lease. Canonical Codex rollouts and repository Run checkpoints
  are outside the paths this controller may replace.
- Electron consumes the exact Product manifest and schema tuple from signed update metadata before
  `quitAndInstall`, retains the current application bundle, and aborts the staged transition if
  installation cannot begin. On the successor launch it activates the staged tuple, waits for the
  native backend health signal, commits the first launch, or restores and relaunches the retained
  predecessor after one bounded failure.
- Product-facing desktop and renderer copy now consistently says Orchestra. T3Code names remain
  only where they are compatibility identifiers, environment variables, or the legacy user-data
  migration source.
- The bundled Authoring plugin baseline is inactive, has an explicit lifecycle, and cannot install
  native execution.
- The release build generates an SPDX 2.3 JSON SBOM and third-party notices from both locked Cargo
  graphs and the production pnpm license inventory. The exercised fixture contained 2,903 unique
  dependency package identities.

## Automated evidence

- `cargo test --workspace`: passed 103 active tests, including a macOS desktop lifecycle integration
  test that retains an application bundle, activates its successor, then restores the predecessor
  and records the bounded rollback. The five evaluator integration cases remain assigned to their
  pinned-worker gate.
- `cargo fmt --all --check`, `git diff --check`, and `orchestra-lifecycle doctor`: passed.
- Product release preflight against fresh Codex `f90e7deea6a715bbd153044af6f475eefa749177` and
  T3Code `ecb35f75839925dd1ac6f854efeef5c9e291d11b` sources: passed.
- The final retained T3Code build passed desktop and web typechecks, 335 desktop tests, 1,322 web
  tests, the wider T3Code suite, and the production web, server, Electron main, preload, and
  preview-preload builds. Product host and two-launch Electron smoke tests passed against manifest
  `be70c86700011ee7ec432d841eea5e51d03d178bbed6d02e69cc9d4df15a8dcd`.
- The rebuilt Electron app was captured after the final branding pass.
  [Running Orchestra window](assets/2026-07-17-orchestra-electron-window.png) shows the native
  `t3code://app/` window, normal project/task shell, Orchestra Alpha wordmark, and no replacement
  workflow dashboard. The separately authenticated
  [live renderer capture](assets/2026-07-17-orchestra-running.png) verifies the same shell through
  the desktop-managed local endpoint.
- Real unsigned arm64 and x86_64 packaging rehearsals each produced an Orchestra DMG, full-app ZIP,
  and updater blockmaps. Bundle identity is `com.edgefloor.orchestra`; the app name and executable
  are rebranded; all three native executables report the expected architecture; and Codex,
  evaluator, Product helper, and the inactive plugin baseline match their per-architecture sealed
  manifest hashes under renderer-inaccessible application resources. The full-app ZIP SHA-256
  identities are `0a9f8220f2a02879aaf35b955be749ba63f1ca799e0f23b8425ce5944d5c40e3`
  (arm64) and `490396136b1b196b56834eb2d104d0375027a2662f03ed81b1f4102b224d57ca`
  (x86_64).

## Production publication deferred

The production publication gate intentionally remains fail-closed. No Apple signing certificate, notarization API
credentials, or signed update-feed key was available in this environment, so code-signing, Hardened
Runtime verification, notarization, stapling, signed metadata publication, and the final two-machine
human gate are not claimed.

Those credentials and first public candidate are not required to dogfood the MVP. They are tracked
separately in issue #56. The repository keeps the real publication gates and unsigned-rehearsal
boundary intact; revising the MVP milestone does not turn unsigned output into a publishable release.
