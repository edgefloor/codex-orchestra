# Codex Orchestra Native Plugin Scaffolding Plan

## Purpose

Scaffold Codex Orchestra as a completely Codex-native orchestration layer and use the existing Orchestra framework to conduct its development.

This plan covers scaffolding only. It does not authorize implementation beyond establishing the package structure, configuration templates, native workflow surfaces, and a minimal self-hosting validation path.

## Product boundary

Codex Orchestra will consist of:

- an installable Codex plugin containing skills and supporting assets;
- optional repository-scoped Codex configuration and custom agent TOMLs;
- optional global Codex profile configuration and custom agent TOMLs;
- native Codex threads, subagents, worktrees, collaboration tools, and approval flows;
- repository-grounded artifacts for plans, decisions, context, results, verification, and recovery;
- optional native hooks where deterministic lifecycle enforcement is useful.

The core architecture will not include:

- an MCP server;
- a Codex App Server client;
- a sidecar or daemon;
- an external scheduler or workflow service;
- a separate durable control plane.

## Bootstrap invariant

Orchestra version N develops Orchestra version N+1.

The installed version conducting a development run must never modify its own cached installation. It works against the source repository for the next candidate version. A candidate is installed and tested in a fresh Codex task only after validation, while the preceding known-good version remains available as a fallback.

## Phase 1: Preserve and inventory the seed framework

1. Treat `codex-orchestra-framework/` as the seed conductor rather than the target package layout.
2. Record known incompatibilities, especially conflicting or version-sensitive Codex configuration.
3. Inventory the existing:
   - skills;
   - custom agent TOMLs;
   - policies;
   - schemas;
   - templates;
   - tests;
   - documentation;
   - reference utility behavior.
4. Classify each item as:
   - migrate unchanged;
   - normalize for plugin packaging;
   - retain as design reference;
   - replace;
   - remove.
5. Do not move or delete seed files during this phase.

### Exit criteria

- A migration inventory exists.
- Known configuration assumptions are explicitly marked for capability validation.
- The seed framework remains runnable as the fallback conductor.

## Phase 2: Establish the target package layout

Scaffold the following target shape alongside the seed framework:

```text
codex-orchestra/
├── .codex-plugin/
│   └── plugin.json
├── skills/
├── hooks/
├── assets/
│   ├── schemas/
│   ├── policies/
│   └── templates/
├── config/
│   ├── project/
│   │   ├── config.toml
│   │   └── agents/
│   └── global/
│       ├── orchestra.config.toml
│       └── agents/
├── tests/
└── docs/
```

Repository-local run state will use a separate installed-project structure:

```text
<target-repository>/.codex/orchestra/
├── charter/
├── plans/
├── decisions/
├── context/
├── tasks/
├── results/
├── verification/
├── recovery/
└── overrides/
```

Generated run state must never be stored inside the installed plugin directory.

### Exit criteria

- Package source and mutable project state have distinct locations.
- The plugin manifest is valid.
- No external runtime integration is present.

## Phase 3: Scaffold the minimal skill surface

Create one primary entry point:

```text
$codex-orchestra:orchestrate
```

Scaffold supporting skills or skill resources for:

- grounding a project;
- creating an engagement charter;
- planning delivery;
- forming a taskforce;
- leading a workstream;
- packaging context;
- executing an assignment;
- reviewing an assignment;
- verifying a milestone;
- recovering an interrupted run;
- handing off or closing a run.

Initially normalize and reuse the existing skills. Avoid redesigning the entire operating model during scaffolding.

### Exit criteria

- The primary skill can be discovered and invoked from an installed development plugin.
- Its instructions route work through native Codex collaboration facilities.
- Supporting resources resolve from within the plugin package.

## Phase 4: Scaffold the Codex configuration kit

### Repository installation

Provide templates for:

```text
<repository>/.codex/config.toml
<repository>/.codex/agents/*.toml
```

This mode activates Orchestra defaults automatically when Codex opens a trusted repository.

### Global selectable profile

Provide templates for:

```text
~/.codex/orchestra.config.toml
~/.codex/agents/*.toml
```

This mode is selected explicitly with:

```bash
codex --profile orchestra
```

### Global default

Document how a user can intentionally merge Orchestra settings into:

```text
~/.codex/config.toml
```

Do not silently replace user-owned global configuration.

### Configuration requirements

- Preserve user-owned settings.
- Detect and report conflicting keys.
- Keep custom agents individually selectable in supported Codex interfaces.
- Validate configuration against the installed Codex version.
- Avoid relying on undocumented defaults.

### Exit criteria

- Project, profile, and global-default modes are clearly distinct.
- Agent TOMLs load from their intended scope.
- Configuration precedence is tested and documented.

## Phase 5: Scaffold installation and lifecycle workflows

Provide Codex-native, skill-guided procedures for:

- installing the configuration into a repository;
- installing a global Orchestra profile;
- intentionally making Orchestra the global default;
- checking compatibility with `doctor`;
- upgrading installed templates;
- detecting locally modified files;
- uninstalling without deleting user-owned work;
- rolling back to the previous known-good version.

Any bundled scripts must remain transparent implementation helpers invoked through the native skill workflow. They must not become a background service or alternate orchestration runtime.

### Exit criteria

- Each operation previews its intended changes.
- Existing configuration is preserved or explicitly reconciled.
- Upgrade and uninstall behavior are reversible.

## Phase 6: Create the first self-hosting vertical slice

Use the seed Orchestra to conduct one bounded development engagement whose objective is to improve the candidate Orchestra plugin.

The minimum topology is:

```text
Operator
  → Consultant
  → Team Leader
  → Worker
  → Reviewer
  → Operator checkpoint
```

The slice must demonstrate:

1. grounding the Orchestra source repository;
2. producing an engagement charter;
3. planning one small candidate improvement;
4. delegating implementation through native Codex agents;
5. reviewing the result independently;
6. recording results and verification evidence in repository artifacts;
7. installing the candidate as a development plugin;
8. opening a fresh Codex task using the candidate;
9. retaining the seed version as fallback.

### Exit criteria

- A known-good Orchestra version successfully conducts work on its successor.
- The candidate can be installed and invoked in a fresh task.
- Failure of the candidate does not damage the conducting version.

## Phase 7: Validate the scaffold

Validate:

- plugin manifest structure;
- marketplace installation and cached-plugin behavior;
- skill discovery and namespacing;
- project configuration parsing;
- global profile selection;
- custom-agent discovery and manual selection;
- configuration precedence;
- separation of plugin files from mutable run state;
- native subagent delegation;
- recovery from an interrupted task using repository artifacts;
- upgrade and rollback behavior;
- one end-to-end self-hosting scenario.

Do not expand to the complete role catalog, unattended execution, external integrations, or production hardening until this vertical slice passes.

## Proposed implementation order

1. Seed migration inventory.
2. Target directory scaffold.
3. Minimal plugin manifest and development marketplace entry.
4. Primary `orchestrate` skill.
5. Minimal custom-agent set.
6. Project configuration template.
7. Global profile template.
8. Repository run-artifact structure.
9. Install, doctor, upgrade, uninstall, and rollback workflows.
10. Self-hosting vertical slice.
11. Fresh-task installation validation.
12. Decision checkpoint before further implementation.

## Initial minimal agent set

Only scaffold the agents required for the first self-hosting slice:

- Consultant;
- Team Leader;
- Worker;
- Reviewer.

Additional Manager, Delivery Architect, Context Engineer, Quality Governor, specialist, and verifier roles should be migrated only after the minimal topology demonstrates that the separation creates measurable value.

## Stop condition

Scaffolding is complete when:

- the plugin and configuration skeletons exist;
- the primary skill is installable and invocable;
- the minimal custom-agent set is discoverable;
- repository-native run artifacts have a defined home;
- installation and rollback paths are validated;
- the seed Orchestra completes one self-hosting vertical slice;
- unresolved architectural questions are recorded for an Operator decision.

At that point, stop and review the evidence before implementing the full framework.
