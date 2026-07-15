# Native skill characterization — 2026-07-15

This record characterizes the narrow boundary between Orchestra agent prompts and Codex skill
loading at the exact revision in `integration/codex/UPSTREAM_REVISION`. It records observed
behavior; it does not grant the Orchestra runtime a new skill-resolution contract.

## Observed contract

- Orchestra submits each agent prompt as one `UserInput::Text`. A literal `$wayfinder` mention
  therefore reaches Codex's ordinary explicit-skill selection path without a second dispatcher.
- `policy.allow_implicit_invocation: false` removes a skill from the model-visible available-skills
  catalog. It does not prevent an unambiguous explicit `$name` or exact linked-path invocation.
- A disabled skill, an unknown plain name, an ambiguous plain name, or a name colliding with a
  connector selects no skill. The plain-name cases do not fail the turn. An exact linked skill path
  disambiguates duplicate names, unless that path is disabled or missing.
- Explicit selection injects the selected `SKILL.md` body as a contextual user fragment. Referenced
  files are not recursively snapshotted or injected; the skill instructions direct the agent to
  read them later through the owning filesystem. Missing referenced files therefore fail only if
  and when the agent tries to read them; child creation and skill injection do not validate them.
- Codex's `HostSkillsSnapshot` freezes discovery metadata and filesystem mappings. The instruction
  body is read from the skill path when the explicit mention is processed. The config-aware skill
  cache key does not include skill file content, so a new or renamed skill is invisible until cache
  invalidation. Editing the body at an already-discovered path affects a later invocation because
  that body is reread. Deleting it produces an injection warning and no injected body; it does not
  fail child creation.
- Machine-readable dependencies cover tools declared in `agents/openai.yaml`. References from one
  skill's prose to another skill are not a validated transitive dependency graph. The explicit
  selector scans the child task input, not the subsequently injected skill body, so a missing prose
  dependency is neither selected nor rejected automatically and the turn continues without it.

## Matt Pocock skill sample

The installed `wayfinder`, `to-spec`, `to-tickets`, and `implement` skills set
`policy.allow_implicit_invocation: false` in `agents/openai.yaml`; `code-review` currently does not.
Their legacy `disable-model-invocation: true` frontmatter is not the field read by this pinned Codex
loader. Wayfinder's prose refers to `grilling`, `domain-modeling`, `research`, and `prototype`, and
implement refers to `tdd`; only `prototype` was installed during this observation.

## Narrowest viable integration seam

Keep skill selection on the native Codex turn path. A future Orchestra workflow step should carry
an exact skill identity, ask the native host to resolve it against the child's effective skill
catalog, and persist the resolved instruction and referenced-resource bytes before spawning. It
must fail before agent work on missing, disabled, ambiguous, or unsatisfied transitive requirements.
Prepending an unchecked `$name` to a prompt is sufficient for best-effort invocation but is not a
durable or validated workflow contract.

## Automated evidence

Run:

```bash
scripts/characterize-pinned-skills.sh <pinned-codex-checkout>
```

The script refuses the wrong upstream revision or a stale Orchestra overlay, then covers the
Orchestra text-input handoff, explicit selection of a non-implicit skill, missing and ambiguous
names, turn-level instruction injection, policy loading, explicit mention selection, and
config-aware cache reuse. It also pins source assertions for the remaining negative behavior that
cannot be observed through a public host API without changing the boundary being characterized.

| Behavior | Executable or pinned evidence |
| --- | --- |
| Native child handoff | Overlay tests prove the Orchestra prompt remains `UserInput::Text`; pinned `agent/control/spawn.rs` proves that initial input enters the ordinary turn path. |
| Explicit non-implicit selection | Overlay selection test plus the `collect_explicit_skill_mentions` test group. |
| Instruction injection | Pinned `core` integration test `user_turn_includes_skill_instructions` and the `session/turn.rs` injection assertions. |
| Referenced resources | Pinned `core-skills/src/injection.rs` reads only the selected `SKILL.md`; no recursive resource loader participates in turn injection. |
| New, renamed, or deleted cache entries | `skills_for_config_reuses_cache_for_same_effective_config` creates a new skill after caching and observes unchanged discovery metadata; the invocation-time body-read assertion defines edit/delete behavior at an existing path. |
| Transitive skill references | Pinned metadata models only tool dependencies, while explicit selection runs on task input before skill-body injection. |

Human-only observations of skill UI visibility remain pending until performed in the installed
candidate.
