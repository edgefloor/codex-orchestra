# Codex Orchestra

Codex Orchestra creates and executes declarative, repository-native workflows through the active Codex agent and native collaboration tools.

## Language

**Workflow**: A reusable YAML definition containing inputs, limits, dependency-linked steps, checks, review, and approvals.

**Run**: One execution of a workflow against a repository and source revision.

**Step**: The smallest workflow unit. A step is an agent task, deterministic check, or user approval.

**Stage**: The set of dependency-ready steps that may run together. Independent read-only steps can run in parallel; dependency chains form a pipeline.

**Task input**: The self-contained instructions, resolved values, source revision, scope, limits, and output contract passed to an agent step.

**Result**: Structured output from one step attempt, including declared outputs, changed files, evidence, and residual risk.

**Check**: A deterministic command whose invocation, exit status, and output are recorded as evidence.

**Review**: Independent model judgment performed by an agent that did not implement the reviewed change.

**Approval**: A user decision that pauses the run before material scope, risk, irreversible action, or acceptance can proceed.

**Attempt**: One bounded execution of a step against a specific workflow and source revision.

**Run summary**: The transcript-independent record of outputs, changes, checks, findings, approvals, residual risk, and the next action.

## Runtime boundary

The active Codex agent is the version 1 model-executed workflow runtime. YAML is declarative data; it never executes as code. Native subagents, messages, waits, worktrees, commands, and review agents perform the work. Repository artifacts preserve accepted state.

This provides reviewable and resumable workflow semantics, but not Anthropic's background JavaScript runtime, script variables, or hundreds-of-agents scale. Workflow contracts stay backend-neutral so a future first-party Codex workflow runtime can execute them without changing user-authored definitions.
