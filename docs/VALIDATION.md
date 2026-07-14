# Validation

Automated validation covers:

- plugin manifest and supported bundle structure;
- all skill frontmatter and UI metadata;
- TOML configuration and native Codex capability probes;
- workflow JSON Schema and semantic rules;
- valid and invalid workflow fixtures;
- lifecycle preview, install, upgrade, rollback, and uninstall;
- retired terminology in user-facing documentation and skills;
- mutable-state exclusion from the plugin package.

Run:

```text
uv run --locked python -m unittest discover -s tests -v
uv run --locked python scripts/workflow.py validate evals/workflows/native-vertical-slice.yaml
uv run --locked python scripts/lifecycle.py doctor
```

Interactive verification additionally covers fresh-task discovery, custom-agent selection, an installed self-hosting run, interruption recovery, and candidate promotion.
