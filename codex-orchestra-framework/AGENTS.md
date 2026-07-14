# Repository Operating Contract

## Canonical truth

Read only what the active gate/assignment requires. Binding sources, in order:

1. accepted `.orchestra/charter/BRIEF.md`, `SCOPE.md`, `ASSURANCE.yaml`;
2. accepted decisions and current milestone/workstream plans;
3. current Delegation Permit, Role Card, Context Capsule, and task/gate contract;
4. applicable `.orchestra/policies/` and schemas;
5. repository source/tests/evidence.

Chat history is not canonical. Reference artifacts instead of copying them.

## V2 authority

The physical tree is bounded to depth 3. A role may spawn only child archetypes listed in `.orchestra/policies/collaboration-v2.yaml` and only under a current Delegation Permit.

- root: Consultant, Delivery Architect, Manager, Quality Governor;
- Manager: Team Leaders and Role Architect only;
- Team Leader: branch-local Context Engineer, approved explorers/advisors/workers/reviewers;
- Quality Governor: verifiers/advisors;
- leaf/service roles: no children.

The parent that spawns a child owns its wait, join, first-line review, and bounded upward report. Use `wait_agent`; do not poll agents or filesystem queues. Default to `fork_turns: none`.

## Role boundaries

- Manager performs only people/ownership/capacity/priority/escalation/checkpoint decisions. It never plans tasks, writes prompts/capsules, reviews diffs, runs tests, or contacts workers.
- Team Leader plans one bounded wave, defines hire requirements, forms an approved squad, keeps hard/high-context work, reviews children, and integrates.
- Context Engineer packages accepted task meaning; it does not reinterpret scope or plan.
- Role Architect compiles an approved role; it does not approve hiring.
- Reviewer/Verifier remain independent and never patch their own findings.
- Workers do not change scope, merge themselves, or spawn.

## Assignment discipline

Every delegated task requires current revision/digest, owner, acceptance criteria, permit, capsule, output schema, evidence expectations, stop conditions, and—when writing—base revision, worktree, lease, and declared write domain.

Reversible ambiguity uses the documented default. A blocker needs concrete evidence, impact, missing authority/dependency, attempted resolutions, owner, deadline, and safe default.

## Communication

Use compact typed envelopes. Leaf results include explicit `no_findings` and at most five material findings. Detailed output remains in referenced artifacts. Upward branch reports contain only outcome, evidence refs, capacity/decision consequences, residual risks, and next action.

## Code and integration

Run validation required by the capsule. Do not edit outside declared domains. Do not overwrite unrelated work. Team Leaders integrate through one workstream queue; cross-workstream integration has one named owner.

## Output

Return exactly the schema requested by the parent. Be explicit about completed, partial, blocked, failed, or no-findings. Never claim tests/evidence that were not run or inspected.
