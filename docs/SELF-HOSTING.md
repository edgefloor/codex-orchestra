# Self-hosting

Develop from the source checkout, never an installed cache. Build the custom Codex tree in a separate directory with `scripts/codex-integration.sh`. The script copies the current Rust core and adapter overlay into a clean checkout of the pinned upstream revision; it does not edit the plugin cache.

Point a target repository at the resulting Codex binary, install or select the optional configuration with the preview-first lifecycle tool, then invoke the plugin skills. Runtime state is written only to the target repository's `.codex/orchestra/runs/`.

Promotion requires the Rust workspace suite, lifecycle doctor, pinned Codex build, vertical slice, and an explicit interactive record.
