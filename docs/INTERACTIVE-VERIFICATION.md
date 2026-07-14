# Interactive verification

Automated checks cannot prove plugin discovery, custom-agent UI behavior, or a real native multi-agent run. Record those observations separately.

## Stage 0 — automated baseline

Run the unit suite, workflow validator, lifecycle doctor, skill validators, and plugin validator. Record exact commands, versions, exit status, and package digest.

## Stage 1 — fresh installation

Install the packaged candidate through the configured local marketplace. Start a new Codex task in a repository that does not contain the plugin source. Invoke `$codex-orchestra:orchestrate` and confirm the installed skill is used without development-only paths.

## Stage 2 — custom agents

Install project or profile configuration through the preview-first lifecycle. Confirm planner, worker, reviewer, and verifier appear and can be selected. Reviewer and verifier must remain read-only; worker must not spawn children.

## Stage 3 — native self-hosting workflow

Use installed N to run `evals/workflows/native-vertical-slice.yaml` against candidate N+1. Confirm the planner step, two parallel read-only steps, isolated worker, deterministic check, independent review, conditional approval, durable state, and final summary. Hash N before and after the run.

## Stage 4 — interruption recovery and promotion

Interrupt after at least one completed step. Start a transcript-free task and invoke `$codex-orchestra:resume-workflow`. Confirm completed evidence is reused and only dependency-ready work continues. Package and install N+1 only after all mandatory checks pass.

## Human-only evidence

Fresh-task discovery, UI selection, live parallelism, interruption recovery, and promotion observations stay `pending` until actually performed.
