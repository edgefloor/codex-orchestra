---
status: accepted
---

# Separate plugin distribution from native execution and configuration

The independently versioned plugin distributes skills, documentation, editor assets, and
configuration templates, while native workflow execution ships only in the lockstep Product release
defined by ADR 0017. Plugin installation never changes native execution or user configuration;
configuration lifecycle remains an explicit, preview-first, conflict-refusing operation because
project and Codex-home files remain user-owned.
