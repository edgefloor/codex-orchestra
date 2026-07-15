---
status: accepted
---

ADR 0014 supersedes this decision's always-human approval model with separate Permission Request,
Decision Gate, Gate Policy, and authority semantics.

# Resolve and snapshot skills before native workflow execution

Skill-backed workflows declare exact skill requirements, typed run inputs, human interactions, and
external-effect authority as execution-plan data. They do not ask a model to discover capabilities
from an unchecked prompt. The Rust runtime continues to own scheduling, state, evidence, and
recovery; agent execution continues only through the active task's native `AgentControl` path.

## Skill identity and requirement closure

A skill requirement identifies one enabled skill in the effective child configuration. Its identity
includes the canonical qualified name, source kind, stable source locator, and plugin or provider
identity when present. Plain names that are missing, disabled, ambiguous, product-incompatible, or
connector-colliding fail validation before any dependent agent starts. A workflow may use a plain
name only when native resolution proves it unique.

The execution plan declares the complete transitive skill closure. References in `SKILL.md` prose do
not create dependencies. Each requirement explicitly names other required skills and the relative
resources whose exact contents the run depends on. Cycles, escaping paths, missing resources, and
unavailable requirements are validation failures. Tool dependencies declared by skill metadata stay
under Codex's native dependency and permission handling, but their declarations are part of the
snapshot manifest.

The native host resolves skills against the same effective configuration and owning filesystem that
the child would use. It returns identity, metadata, instructions, declared resource bytes, and
digests to the runtime before spawn. The runtime persists that bundle under the run evidence and
only then permits native child execution. This extends the narrow injected host capability; it does
not add another dispatcher, process, server, or scheduler.

## Inputs and immutable snapshots

Workflow source declares typed JSON-compatible inputs and defaults. Run invocation supplies values;
environment variables, ambient files, and transcript text are not implicit inputs. The compiler and
runtime validate the values, serialize them canonically, hash them, and persist the resolved object
before selecting a dependency-ready step. Sensitive values are not supported until Orchestra has a
redacted secret-reference contract; workflows must pass identifiers rather than secret material.

Invocation is a native action inside the owning Codex task. A task instruction or skill/tool call
may initiate it; an eventual workflow picker may only submit the equivalent task-bound action. No
renderer or external host starts a detached Run, and no model is responsible for scheduling after
the native runtime accepts the invocation.

The immutable run snapshot consists of the compiled workflow, resolved inputs, source revision,
skill manifest and bytes, and their digests. A new run resolves current skill installations. Resume
never silently reloads an installed skill, changes an input, or substitutes a resource. A requested
change creates a new run or an explicit versioned migration rather than rewriting historical state.

## Human interaction and acceptance authority

Human input and approval are distinct step kinds. A human-input step pauses for a declared free-text
or structured response, validates it, persists it as a step output, and may feed a bounded revision
loop. It does not accept the result. An approval step presents declared outputs or evidence and
records an explicit accept or reject choice. No agent may answer a human-input step, infer approval
from conversation, or promote a patch after rejection.

This supports one-question-at-a-time Wayfinder work, test-seam confirmation for `to-spec`, ticket
breakdown revision for `to-tickets`, and final acceptance after implementation and independent
review without treating every request for revision as cancellation.

## Tracker authority and external effects

Tracker reads are ordinary read-only context. Every tracker mutation declares a narrow external
write scope such as repository, issue operation, and target identity. Explicit workflow invocation
may authorize reversible coordination writes needed to claim the named ticket. Publishing content,
changing dependency topology, closing issues, or performing a batch mutation requires an approval
that presents the generated artifact immediately before the write step.

Tracker work uses the active native agent's ordinary Codex tools and permission policy. Orchestra
does not embed a GitHub client in its core and does not introduce a tracker daemon or sidecar. Each
successful mutation returns a receipt containing provider, repository, operation, stable remote
identities, and enough resulting state to reconcile a retry. Receipts are runtime evidence, not a
substitute for the tracker's source of truth.

GitHub is this repository's primary tracker. Its claim, parent/sub-issue, dependency, frontier, and
resolution operations are defined in `docs/agents/issue-tracker.md`. Tests and offline workflows use
the Git-backed local Markdown contract from the same document; they never mutate live GitHub state.

## Recovery

Checkpoints record input and skill-manifest digests, human responses, approvals, and external-effect
receipts alongside existing step and worktree state. Resume verifies these records before scheduling.
Completed human steps are not asked again. Completed external effects are reconciled by stable remote
identity before retry; already-applied matching state completes idempotently, absent state may be
retried within the original authority, and conflicting state fails for renewed human direction.

Loss of tracker access, a changed target repository, a changed skill requirement, or an input digest
mismatch cannot be treated as a transient agent retry. Patch promotion retains the rules in ADR 0012
and remains downstream of deterministic checks, independent review, and acceptance.

## Consequences

- The next runtime changes add run inputs, skill requirements and snapshots, and resumable human
  input as explicit plan/checkpoint concepts.
- Workflow recipes adapt skill-internal delegation into Orchestra stages when required. In
  particular, Standards and Spec reviews remain independent parallel steps rather than nested child
  delegation.
- Best-effort `$skill` prompt mentions remain useful for ordinary Codex tasks but are insufficient
  for durable workflow execution.
- Human-only UI rendering and provider-backed behavior remain pending until observed in an installed
  candidate.
