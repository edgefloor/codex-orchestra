# Issue #61 — Direct public-fork Product assembly

Date: 2026-07-17

## Pinned source tuple

Product assembly now consumes two standalone public repositories at immutable commits. Normal build,
release, and dogfood paths clone these repositories directly; they do not reconstruct either source
tree with a patch or overlay.

| Source | Repository | Fork commit | Fork tree | Upstream base | Upstream tree |
|---|---|---|---|---|---|
| Orchestra Codex | `https://github.com/edgefloor/orchestra-codex.git` | `64bcf75f55745b8304b466bdca1231eb6c5e0620` | `4c5d70a7671f5b27aec0244f7baa707009318788` | `f90e7deea6a715bbd153044af6f475eefa749177` from `https://github.com/openai/codex.git` | `1e83e292b90e89d993703d08f05fb8579601bda2` |
| Orchestra Desktop | `https://github.com/edgefloor/orchestra-desktop.git` | `7e526e83499cd905928a64b80f0a3930424bdb98` | `f664a25a141cfa32aca18327b87a52718fc70a33` | `ecb35f75839925dd1ac6f854efeef5c9e291d11b` from `https://github.com/pingdotgg/t3code.git` | `147cfa26a60049c8b89f836b3c4fdb474906b6a7` |

The Codex fork also seals protocol tree `cab853f856a1543d57d7c7e12b1f904f5e3d2bfb`.
Its 696 protocol files produce digest
`5dfbb6a2a7237b699c83cc125a48e2d3a1f0f59b5ce21909ab325f5a81f10da7` using
`sha256-relative-path-nul-file-sha256-lf-v1`. The desktop's generated protocol identity matches that
tuple.

## Clean-clone evidence

- `scripts/product-source-prepare.sh /tmp/orchestra-issue61-sources` cloned the public fork URLs into
  fresh `codex/` and `desktop/` directories and checked out the two exact detached commits.
- `scripts/product-source-verify.sh /tmp/orchestra-issue61-sources` verified each origin URL, commit,
  commit tree, upstream-base ancestry, upstream-base tree, fork provenance manifest, and retained
  desktop capabilities. It also verified the coordinator and protocol identities.
- The pinned native skill characterization passed against the clean Codex fork at
  `64bcf75f55745b8304b466bdca1231eb6c5e0620`.
- The Product doctor and lifecycle scaffold reject retired integration directories, patch scripts,
  overlays, and active Product scripts that attempt to apply a patch.

## Full Product build evidence

`scripts/product-dev-build.sh /tmp/orchestra-issue61-sources /tmp/orchestra-issue61-product`
completed against the clean-clone tuple on `aarch64-apple-darwin`.

| Gate | Result |
|---|---:|
| Desktop static checks | passed with 0 errors and 12 warnings |
| Monorepo typecheck | 15/15 tasks passed |
| Web test suite | 1,324 tests passed |
| Server test suite | 1,408 tests passed, 7 skipped |
| Desktop test suite | 335 tests passed |
| Production desktop build graph | 4/4 tasks passed |
| Packaged evaluator | 5 fresh-process tests passed |
| Sealed Codex host smoke | App Server initialize/manifest handshake passed |
| Electron launch smoke | two consecutive launches reached backend ready and main window created, then shut down without an orphan listener |

The resulting release inputs and SBOM identify the two fork packages by their immutable public VCS
locations and retain their upstream-base relationships. Product builds therefore have a reviewable,
reproducible fork boundary without depending on coordinator-local source reconstruction.

## Follow-up boundary

Issue #69 separately owns the exact Bun and Zod evaluator provenance, integrity, architecture-smoke,
and boundary checks. Those dependencies remain pinned Product inputs; they are not additional product
fork repositories and do not weaken the direct-fork completion recorded here.
