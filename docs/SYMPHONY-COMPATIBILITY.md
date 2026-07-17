# Symphony compatibility

Orchestra adopts Symphony's useful coding-harness contract while keeping Codex tasks and the native
Rust runtime authoritative. “Adapted” keeps the user-visible capability but expresses it through
native Codex tasks, typed Workflows, and bounded projections. “Excluded” is outside the MVP.

## Profile fields

| `WORKFLOW.md` surface | Class | Orchestra behavior |
| --- | --- | --- |
| Markdown body | Adopted | Renders the Issue-task prompt with the normalized issue and attempt. |
| `tracker.kind`, `endpoint`, `project_slug` | Adopted | Supports the bounded Linear adapter only; endpoint defaults to Linear. |
| `tracker.api_key` / `credential` | Adapted | Resolves an environment reference at use time; stores only its reference and digest. A missing value skips live reads but leaves fixture validation available. |
| `tracker.required_labels`, `active_states`, `terminal_states` | Adopted | Normalize eligibility, refresh, terminal reconciliation, and cleanup. |
| `polling.interval_ms` | Adapted | Supplies retry/reconciliation timing inside the resident task action; it does not start a daemon. |
| `workspace.root` | Adopted | Names the dedicated persistent Issue-worktree root and is containment checked. |
| `hooks.after_create`, `before_run`, `after_run`, `before_remove`, `timeout_ms` | Adopted | Run as bounded native host commands with inspectable receipts. |
| `agent.max_concurrent_agents`, `max_concurrent_agents_by_state` | Adopted | Bound deterministic Issue claims and native child activity. |
| `agent.max_turns`, `max_retry_backoff_ms` | Adapted | Bound typed Workflow invocations and durable continuation/retry schedules. |
| `codex.approval_policy`, `thread_sandbox`, `turn_sandbox_policy` | Adapted | May narrow, but never broaden, the owning Codex task's effective policy. |
| `codex.turn_timeout_ms`, `read_timeout_ms`, `stall_timeout_ms` | Adopted | Bound native invocation, tracker reads, and liveness reconciliation. |
| `codex.command` | Excluded | Orchestra uses the pinned resident Codex runtime and rejects an alternate command/backend. |
| `orchestra.workflow` | Orchestra extension | Selects one repository-contained restricted `.workflow.ts`, parsed and lowered by Rust. |
| `orchestra.effects` | Orchestra extension | Allow-lists typed, policy-gated Tracker effects. |
| Unknown top-level fields | Adapted | Preserved only as bounded extension warnings; they gain no authority. |
| Unknown nested fields | Excluded | Rejected to keep the effective profile deterministic. |

## User-visible behavior

| Symphony behavior | Class | Orchestra MVP behavior |
| --- | --- | --- |
| Linear candidate, terminal, and single-Issue refresh reads | Adopted | Uses fixed bounded queries and normalized values; no raw GraphQL API is exposed. |
| Label/state/blocker filtering, priority order, concurrency, retry/backoff, and stall detection | Adopted | Persisted in the Automation Root Run checkpoint. |
| One workspace per issue and lifecycle hooks | Adopted | Uses a persistent Issue worktree with bounded hook and cleanup receipts. |
| Start and continuous coordination | Adapted | Start is a visible action in the owning Automation task; continuations are durable task work, not a hidden scheduler. |
| Issue execution | Adapted | Creates a canonical native Issue task and runs a typed Workflow inside it. |
| Tracker comments, transitions, and pull-request links | Adapted | Typed effects require the profile allow-list, gate policy, idempotency key, and provider receipt. |
| Pause, resume, refresh, cancel Issue, cancel Run, and inspect | Adapted | Task-scoped App Server requests operate the native checkpoint and project into the retained T3Code dialog. |
| Profile reload | Adopted | Stages a digest-pinned revision, rejects invalid changes, and keeps the last known good profile. |
| Status dashboard | Adapted | The normal T3Code task surface shows a bounded Root Run, queue counts, claims, receipts, and stable IDs. |
| Detailed histories | Adapted | Raw child history stays in its native task; Run Digests, snapshots, and replay contain bounded summaries and stable targeted-expansion IDs. |
| Missing Linear credentials | Adapted | Validation warns; live intake returns `skipped` with a next action, while deterministic fixtures still run. |
| Background daemon, external scheduler, or detached run | Excluded | The owning Codex task and native runtime remain resident authority. |
| Alternate agent backend or `codex.command` | Excluded | Only the pinned Codex product fork is supported. |
| Generic/raw GraphQL, arbitrary tracker mutations, or other trackers | Excluded | Only fixed Linear reads and typed allow-listed effects are available. |
| Browser administration, remote/distributed workers, SSH workspaces | Excluded | Not required by the local coding-harness MVP. |
| Copying child transcripts into parent/UI state | Excluded | Targeted native task expansion is the only detail path. |

The pinned app-server test `workflow_md_linear_fixture_reaches_bounded_desktop_projection` covers
the executable seam. It asserts that a raw Workflow child response is absent from the desktop
projection even though its typed Tracker output completes.
