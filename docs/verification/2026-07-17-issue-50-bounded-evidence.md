# Issue #50 — Bounded Evidence and operational detail

Date: 2026-07-17

## Product contract

- Evidence remains runtime-owned under the authorized task's run storage. The desktop receives
  compact identity, type, provenance, integrity, size, and availability metadata; internal paths
  are not part of the desktop protocol.
- Evidence content is a fixed native `orchestra/query` selector addressed by opaque evidence id.
  The runtime authorizes the task and run before resolving that id and never accepts a renderer
  path.
- Content inspection is explicit and bounded. Text bodies are limited to 32 KiB and the existing
  query response budget; initial task events, run projections, step pages, and evidence pages do
  not include content bodies or child history.

## Implementation

- Native query projections now classify evidence provenance and availability, calculate stable
  identity and integrity references, and expose bounded text content only through
  `evidence_content`.
- Missing or expired identities, unauthorized task access, malformed text, oversized content, and
  unavailable query failures remain distinct. Malformed and oversized files return metadata
  without content.
- The normal workflow run tree lists compact evidence references after a step is opened. Opening a
  reference performs the authorized content query and renders supported text in a bounded native
  detail region. The renderer neither receives nor renders an evidence path.

## Automated evidence

- Native tests cover authorization-before-selection, opaque identity lookup, containment,
  provenance, integrity, malformed text, missing identity, content limits, and response budgets.
- Renderer logic and component tests cover lazy query construction, compact reference display,
  integrity truncation, and missing/expired, unauthorized, malformed, and unavailable states.
- The root workspace passed formatting, 99 executed tests, `git diff --check`, and the
  lifecycle/plugin doctor. Five evaluator-worker tests remain intentionally delegated to their
  pinned Product-worker harness.
- Clean pinned Codex verification passed 78 Orchestra core tests, 16 extension tests with one
  gated live-provider test ignored, 8 Codex core integration tests, 267 app-server protocol tests,
  2 generated-schema fixture tests, and the full app-server compile check at revision
  `f90e7deea6a715bbd153044af6f475eefa749177`.
- The pinned T3Code integration passed 138 web tests and 76 server tests. Web and desktop
  typechecks, canonical patch verification, and production web/server/Electron builds passed at
  revision `ecb35f75839925dd1ac6f854efeef5c9e291d11b`.

## Deferred observation

- Direct authenticated-shell observation of evidence density and expansion remains non-blocking
  evidence debt while the Mac is locked. Authority, bounded loading, error projection, protocol,
  build, and test gates passed.
