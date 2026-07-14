# Bootstrap in a Target Repository

## 0. Confirm V2 behavior

Copy the framework, start a fresh Codex session, and run the capability probe in `docs/CODEX-CONFIGURATION.md`. Confirm named paths, mailbox behavior, depth 3, parent-local completion, and visible routing fields before relying on the hierarchy.

## 1. Install and validate

Copy `.codex/`, `.orchestra/`, `AGENTS.md`, `tools/`, and the relevant docs into the repository.

```bash
python tools/orchestra.py doctor
python tools/orchestra.py init --project-id <id>
python tools/orchestra.py digest --record --actor conductor
```

Keep `.orchestra/runtime/orchestra.db*` and worktrees out of version control.

## 2. Capture Operator intent

Invoke:

```text
$orchestrate Ground this repository for: <intent>
```

Root spawns/reuses `/root/consultant` with `fork_turns: none` and refs to repository evidence. The Consultant may run only a small read-only evidence branch under a grounding permit.

Grounding produces:

- `BRIEF.md`;
- `SCOPE.md`;
- `ASSURANCE.yaml`;
- acceptance behavior/examples;
- assumptions/defaults/invalidation conditions;
- material decision queue.

## 3. Accept grounding and delivery design

After Operator acceptance, record the revision/digest and wake/spawn `/root/delivery_architect`. It produces milestones, workstreams, dependencies, interfaces, integration order, Team Leader capability needs, and evidence expectations.

## 4. Staff the first workstream

Root wakes/spawns `/root/manager` with the accepted delivery plan and portfolio limits. The Manager:

1. appoints one initial Team Leader;
2. spawns/reuses `/root/manager/role_architect` only when an approved role must be compiled;
3. issues an initial Team Leader Delegation Permit;
4. waits for compact Team Leader reports.

Start with one Team Leader and no more than two concurrent writers.

## 5. Run one bounded wave

The Team Leader:

1. plans one wave;
2. keeps the central/high-context task where practical;
3. spawns/reuses its branch-local Context Engineer;
4. submits hire requests for missing capability/capacity;
5. after Manager approval/Role Card/permit, spawns workers/reviewers as its own children;
6. waits, reviews, integrates, and reports upward.

Root should not receive raw worker results.

## 6. Verify and accept

After integration, root wakes/spawns the Quality Governor with the Assurance Profile, change surface, and milestone evidence contract. The Governor spawns permitted verifiers, joins evidence, and returns one gate report.

The Manager recommends the checkpoint disposition from portfolio evidence. Root asks the Operator only for material scope/risk/budget/irreversibility or milestone/final acceptance.

## 7. Pilot limits

For the first supervised run:

- one active Team Leader;
- one workstream wave;
- at most three leaf children in the wave;
- at most two writers;
- no Ultra;
- explicit worktrees and manual integration review;
- human-visible permits and phase transitions.

## 8. Production adapter checklist

Before unattended use, add transactional event/permit state, path/thread reconciliation, worktree lifecycle, idempotency, secret isolation, cost/rate budgets, approval handling, CI/PR integration, audit export, monitoring, and adversarial evals according to the project's Assurance Profile.
