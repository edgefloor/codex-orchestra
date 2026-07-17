# Self-hosting

Develop from source checkouts, never an installed cache. Run `scripts/product-source-prepare.sh <destination>` to clone the exact public Orchestra Codex and Orchestra Desktop revisions, then `scripts/product-source-verify.sh <destination>` to verify their upstream ancestry and cross-repository identities. Normal development never applies a patch or copies an overlay.

Point a target repository at the resulting Codex binary, install or select the optional configuration with the preview-first lifecycle tool, then invoke the plugin skills. Runtime state is written only to the target repository's `.codex/orchestra/runs/`.

Promotion requires the Rust workspace suite, lifecycle doctor, pinned Codex build, vertical slice, and an explicit interactive record.
