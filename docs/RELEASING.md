# Product releases

Orchestra releases the pinned Orchestra Codex fork, evaluator, Product helper, and Orchestra Desktop fork as
one macOS Product. The Authoring plugin remains independently versioned; its bundled baseline is
inactive and cannot install native execution.

The release path extends the desktop fork's retained Electron packaging and updater. `electron-updater` is
pinned exactly, differential downloads are disabled, and each architecture
receives a complete application archive. This keeps one updater in the desktop instead of adding a
parallel Sparkle control path.

## Candidate phases

Prepare exact sources and run the credential-free preflight:

```sh
scripts/product-source-prepare.sh /tmp/orchestra-product-sources
scripts/product-release.sh preflight /tmp/orchestra-product-sources
```

Preflight verifies Bun's installed version/revision, the public Bun and Zod release tags, the
published Zod package source and integrity, and the evaluator worker/package/lock digests before any
candidate is built.

Build both macOS architectures:

```sh
scripts/product-release.sh build /tmp/orchestra-product-sources target/release-candidate
```

Publishing candidates require the Apple certificate, notarization API inputs, team ID, and
provisioning profile consumed by the Orchestra Desktop packager. A local rehearsal may set
`ORCHESTRA_ALLOW_UNSIGNED=1`, but unsigned output cannot satisfy the release gate.

Each architecture embeds `codex`, `orchestra-validate-worker`, `orchestra-product`, the exact
Product manifest, and an inactive plugin-baseline record under the application resources. Verify
signed archives independently. Both build and archive verification execute the same exact-Zod,
provenance, forbidden-capability, malformed-input, and limit smoke against the architecture-specific
worker:

```sh
scripts/product-release.sh verify-arch target/release-candidate aarch64-apple-darwin
scripts/product-release.sh verify-arch target/release-candidate x86_64-apple-darwin
```

## Non-waivable evidence

The release evidence file must cover repository, protocol, evaluator, state, Codex-home,
distribution, machine, licensing, and human-evidence gates. Every gate references an immutable
candidate-relative evidence file and its SHA-256. Each target references its sealed manifest and
full archive and records successful code-signing, Hardened Runtime, renderer isolation,
notarization, stapling, and machine verification.

After evidence is complete, seal the release record:

```sh
scripts/product-release.sh seal target/release-candidate release-evidence.json
```

The publication gate is separate. It refuses updater metadata until the publication evidence lists
the exact sealed identities of every archive, manifest, notice, SBOM, corresponding-source file,
and gate record, and until the update metadata and signature identities match their files:

```sh
scripts/product-release.sh publication-gate target/release-candidate publication-evidence.json
```

Only then may release automation publish the update metadata. Candidate, signing, notarization,
verification, and publication remain distinct auditable phases even when one maintainer performs
all of them.

## Update and rollback state

`orchestra-product update-init` and `update-transition` maintain the Product install record under an
exclusive Codex-home maintenance lease. Staging allocates a side-by-side projection generation;
activation retains the predecessor; a verified first launch commits it. One failed pre-commit
launch may automatically return to the retained predecessor. Later reverse transitions require an
explicit snapshot-schema override when the rollback barrier changes.

These transitions do not modify canonical Codex rollouts or repository
`.codex/orchestra/runs/` checkpoints. Two predecessors remain available, while rebuildable
projections are selected by generation.

## Source and licensing material

The build emits all top-level licenses, both fork provenance records, Product pins, both Cargo
dependency graphs, the production pnpm license inventory, generated third-party notices, an SPDX
2.3 JSON SBOM with immutable fork packages and upstream annotations, and a corresponding-source
archive containing the exact Product repository and prepared pinned forks.
Release evidence must additionally confirm LGPL replacement/relink material where applicable and
final product rebranding before the licensing gate may pass.
