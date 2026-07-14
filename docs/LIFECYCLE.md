# Lifecycle

Plugin configuration installation is preview-first and hash-managed. Upgrade snapshots are reversible, uninstall preserves modified files, and run artifacts are never removed.

The `orchestra-lifecycle` Rust binary owns project/profile installation, upgrades, rollback, uninstall, and capability diagnostics. From a source checkout, invoke it with `cargo run -p codex-orchestra-lifecycle -- <command>`.

The Rust extension is built into the pinned custom Codex binary; it is not copied into the installed plugin directory. Updating Codex requires changing the full revision pin, regenerating the minimal patch against that source, and rerunning the integration build before changing the plugin version.
