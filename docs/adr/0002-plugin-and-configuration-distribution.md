---
status: accepted
---

# Distribute behavior as a plugin and activation as optional configuration

The plugin distributes Orchestra skills and supporting assets. Custom agent TOMLs and multi-agent defaults are a separate optional configuration kit that may be installed into a trusted repository, installed as a selectable global `orchestra` profile, or deliberately reconciled into the user's global default. Plugin installation alone must not silently change project or user configuration.

Configuration lifecycle operations are preview-first, preserve user-owned files, refuse unresolved conflicts, track only files Orchestra created, and support reversible upgrade, rollback, and uninstall. Configuration avoids hard-coded models and experimental flags; `doctor` probes the installed Codex version and strict configuration behavior. This rejects the legacy invalid combination of version-sensitive Multi-Agent V2 settings and fixed model routes.

## Consequences

- Project mode uses `.codex/config.toml` and `.codex/agents/*.toml` in trusted repositories.
- Profile mode uses `$CODEX_HOME/orchestra.config.toml` and is selected explicitly.
- Making Orchestra the global default always requires explicit user intent and manual conflict resolution.
- One canonical custom-agent source set must feed both project and global installation targets.
