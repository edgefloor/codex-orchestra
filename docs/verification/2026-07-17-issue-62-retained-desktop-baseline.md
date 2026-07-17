# Issue 62 — retained desktop baseline and directional workspace reference

Issue: <https://github.com/edgefloor/codex-orchestra/issues/62>

## Sealed fork

- Orchestra Desktop revision: `06b7540d8c6897cc37f6d2e463fadfe63be7c918`
- Orchestra Desktop tree: `391f2d0a476e8b6c021c8912a10b91b2e2827dd0`
- Repository: <https://github.com/edgefloor/orchestra-desktop>
- Retained upstream base: `ecb35f75839925dd1ac6f854efeef5c9e291d11b`

## Capability preservation

`docs/retained-desktop-capabilities.json` now seals 15 explicit capability groups: annotations, approvals, authentication, Browser/Preview, composer, diff/review, environments, files/editor, models, panels, recovery, settings, terminal/context, updates, and VCS. Each group names non-empty implementation and test files retained in the standalone desktop fork.

The verifier rejects missing or reordered groups, unsafe or missing paths, an upstream-base mismatch, and empty implementation/test files. It is available as `verify:retained-capabilities` and runs in the desktop fork's CI check job.

## Directional reference

The approved reference is versioned under `docs/design-reference/orchestra-workspace/` in the desktop fork.

- Source URL: <https://orchestra.demystify.hu/orchestra-workspace>
- Captured HTML bytes: `174950`
- Captured HTML SHA-256: `285b5d0a0cd2edd45a0a81f5f816f8dd3276d8f49b6e6c3b828dbc04e5638b79`
- Desktop capture: `1280x720`, SHA-256 `e6f7f195316e83aea9e32bc333863a54c7efeb25173d6829c50731c0b8098a1d`
- Narrow-desktop capture: `900x506`, SHA-256 `efebc08bf992ac0422957457ddaba413eaafb61ea0db086d52a0da5f0727e018`

`reference.json` seals the source identity, capture metadata, brand-note digest, screenshot hashes and JPEG dimensions. The verifier recomputes each local digest and reads the JPEG dimensions. The captures were visually checked after archival; their active theme is correctly recorded as light.

The archive states this precedence explicitly: native behavior, retained capabilities, coherent fusion, then reference pixels. It excludes landing/marketing, mobile and widths at or below 820px, prototype data/actions as product authority, detached workflow control planes, and pixel parity that conflicts with native behavior.

## Verification

- `node scripts/verify-retained-capabilities.mjs` — passed in the desktop fork.
- `vp fmt --check` — passed across 2,117 desktop-fork files.
- `vp check` — passed with 0 errors and the 12 previously observed warnings.
- `cargo test -p codex-orchestra-lifecycle direct_fork_pins_are_explicit_and_patch_assembly_is_retired` — passed.
- `scripts/product-source-prepare.sh target/product-sources-issue62` — cloned both public forks at the sealed tuple and passed source provenance, upstream ancestry, tree, generated protocol, and retained-capability verification.

The full fused Product gate was not rebuilt for this documentation/verification-only desktop change; it had passed immediately before issue 62 during issue 60 verification. This loop reran the narrower fork provenance and baseline checks affected by the new commit.
