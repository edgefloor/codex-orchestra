# Interactive verification runbook

This runbook verifies Orchestra as an installed Codex product. It deliberately separates reproducible commands from observations that require a human in the Codex UI. Run stages in order. Keep version N installed and immutable while evaluating candidate N+1.

## Evidence contract

Create `.codex/orchestra/verification/<date>-interactive.md` in the source repository from `assets/templates/INTERACTIVE-VERIFICATION.md`. Record `pass`, `fail`, `blocked`, or `pending` for every check. A pass needs an observation or an evidence path; command exit status alone is insufficient for UI behavior. Redact credentials and private prompt content. Preserve exact versions, candidate revision/digest, timestamps, command output paths, screenshots when useful, and all negative observations.

The promotion decision is `pending` until Stages 1–3 and the mandatory behavioral scenarios pass. A failure does not authorize repair during the verification run: record it, identify the earliest affected gate, and return to version N or a separate repair attempt.

## Stage 0 — automated baseline

Run from the candidate root and paste the summary into the evidence record:

```bash
uv sync --locked --dev
uv run --locked python ~/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py .
uv run --locked python -m unittest discover -s tests -v
uv run --locked python scripts/lifecycle.py doctor
codex --version
```

Also record a source digest (`git rev-parse HEAD` when Git-backed, otherwise a SHA-256 inventory), the plugin manifest version, and whether the tree is dirty. Verify that the manifest declares neither MCP nor apps, `.codex/orchestra/` is absent from the plugin, and no skill depends on the outer scaffolding wrapper, legacy framework, test fixtures, or a developer home path.

Automatable result: manifest/config parsing, skill discovery, lifecycle behavior, native capability probe, and tests. Human result: none.

## Stage 1 — clean-room install and fresh-task invocation

1. Copy the candidate into a disposable local marketplace using only distributable plugin content. Do not point the check at the working tree if that would allow development-only files to mask a missing asset.
2. Add that non-default marketplace with `codex plugin marketplace add <marketplace-root>`, then install the candidate with `codex plugin add codex-orchestra@<marketplace-name>`. Record the installed cache path and its file inventory. Never edit the cache.
3. In a disposable target repository, preview and apply repository configuration:

```bash
uv run --locked python <candidate>/scripts/lifecycle.py project --target <target>
uv run --locked python <candidate>/scripts/lifecycle.py project --target <target> --apply
```

4. Close the installation task. Open a completely new Codex task rooted at the disposable target.
5. Invoke naturally: `Use Orchestra to ground and plan a small, bounded change in this repository.` Do not name a skill in the first prompt.
6. Confirm that Codex discovers Orchestra, routes to its installed skill, reads installed assets, and creates the run directories under `<target>/.codex/orchestra/`. Confirm no file is created inside the plugin cache.
7. Temporarily move or make unavailable the source checkout and outer legacy/scaffolding files, then repeat the new-task invocation. Restore them after the observation.

Automatable evidence: marketplace/install output, cache inventory, target config hashes, run-directory inventory. Human-only evidence: fresh task boundary, natural routing, UI-visible plugin identity, coherent first response, and absence of hidden reliance on the prior task.

## Stage 2 — manual custom-agent selection

Exercise project mode first, then repeat in a clean target with the global selectable profile. Do not overwrite an existing global default.

```bash
uv run --locked python <candidate>/scripts/lifecycle.py profile --codex-home <disposable-codex-home>
uv run --locked python <candidate>/scripts/lifecycle.py profile --codex-home <disposable-codex-home> --apply
codex --profile orchestra
```

For Consultant, Team Leader, Worker, Reviewer, and Verifier, record:

- whether the agent is visible and manually selectable by its expected name;
- displayed description/metadata and whether the role prompt is observably active;
- inherited base/profile/project settings and sandbox/capability behavior;
- switching away and back without stale role authority;
- one native delegation to the role and the returned join behavior;
- whether skill behavior, agent identity, and native Codex controls feel like one system.

Use a harmless probe matched to each role. Reviewer and Verifier must not edit product source. Worker must not spawn. Consultant must stop before solution design. Team Leader must keep authority within one named workstream. Record unsupported UI capabilities as `blocked` with the Codex build/interface, never as a pass.

Automatable evidence: TOML parse, installed file hashes, strict config/profile probes. Human-only evidence: visibility, selection, displayed metadata, switching, inherited behavior, delegation experience, and native UI coherence.

## Stage 3 — live N to N+1 self-hosting engagement

Choose one small candidate improvement whose acceptance can be verified in one session. Record N, candidate N+1, source revision, charter, risk, and stop conditions before delegating.

1. N grounds the source tree and persists a Context Capsule.
2. The Operator accepts a charter and one bounded plan.
3. N forms the smallest capable taskforce, reserving independent review/verification capacity.
4. Each assignment names its authority, write domain, attempt/child budget, Join Owner, output path, and stop condition.
5. Work proceeds through native delegation/worktrees. The conductor waits, joins, and persists results and checkpoints.
6. Reviewer reports evidence-backed findings or explicit no findings. Verifier executes prescribed gates and records residual risk.
7. Interrupt after a durable checkpoint. Start a fresh task and recover without transcript dependence.
8. Validate and install N+1 without modifying N's cache. Exercise N+1 in another fresh task.
9. Compare N and N+1 with `evals/scenarios/self-hosting-promotion.md`; the Operator records promote, reject, or extend-evaluation.

During the run, count spawns, roles, repeated context, duplicate work, operator interventions, retries, and artifacts. Quote concrete examples of ceremony, incoherent planning, redundant roles, or smooth behavior. These are observations, not assumptions.

## Stage 4 — permanent behavioral evaluation

Run the scenarios in `evals/scenarios/` against the same candidate. Use their exact setup, perturbation, observables, and pass criteria. Store one result per scenario below `.codex/orchestra/verification/evals/<candidate>/`. Every scenario result records the prompt/capsule, source revision, agent topology, decisions, artifacts, findings (including explicit no-findings), command evidence, deviations, residual risk, and verdict.

Promotion requires all mandatory invariants, no unresolved material finding, successful clean-room/fresh-task behavior, successful agent selection in a supported UI, recovery from a checkpoint, and no regression against N in the promotion scorecard. Improvements in speed or fewer artifacts cannot compensate for an authority, isolation, recovery, or assurance failure.
