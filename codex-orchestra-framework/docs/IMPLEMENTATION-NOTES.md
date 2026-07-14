# Control-Plane Implementation Notes

## V2-native pilot adapter

For supervised use, the root Codex session itself applies the Conductor policy with Multi-Agent V2: `spawn_agent`/`followup_task` for dispatch, `wait_agent` for the event loop, completion notifications for wake-up, `list_agents` for recovery, and `interrupt_agent` for revocation. State mutations still go through `tools/orchestra.py`; role agents never write scheduler state.

This mode proves the organization and communication contract but is not a crash-independent daemon.

## Recommended integration boundary

Use a dedicated **Conductor service** around Codex App Server when you need rich thread control, streamed events, approvals, per-thread/turn configuration, and resumable conversations. For simple CI-style one-shot jobs, the Codex SDK can be a thinner adapter; the organizational contracts remain the same.

The service should generate or vendor schemas from the installed Codex version rather than hand-maintaining protocol types. Treat App Server as a versioned external dependency and keep it behind an adapter.

## Adapter interface

```text
CodexAdapter
  start_thread(role_config, cwd, permissions, service_name) -> ThreadRef
  resume_thread(thread_id, overrides) -> ThreadRef
  start_turn(thread_id, inputs, cwd, approval_policy,
             sandbox_policy, model, effort, output_schema) -> TurnRef
  interrupt_turn(thread_id, turn_id, reason)
  stream_events(thread_id, cursor) -> Event*
  read_thread(thread_id) -> ThreadSnapshot
```

The Conductor stores `thread_id`, current `turn_id`, role/Role Card revision, task/attempt, model/effort, cwd/worktree, permissions, and last event timestamp.

## App Server flow

1. Launch `codex app-server` on stdio/JSONL for a local harness. Use authenticated local transport only when a multi-process deployment requires it.
2. Initialize and record server/version/capabilities.
3. Start a thread with model, cwd, approval/sandbox/permission profile, personality where useful, and a service name.
4. Start each turn with the assignment input, explicit skill input, per-turn cwd/sandbox/model/effort, and the JSON output schema.
5. Stream protocol notifications into raw logs. Normalize only stable fields into Conductor events.
6. Treat a structurally valid final model object as a proposed envelope. Validate authority, task lease, revision/digest, artifact existence, and policy before applying state.
7. Resume persistent role threads by recorded ID. Start a fresh thread when role configuration or security boundary changes materially; attach a bounded handoff rather than injecting the full old transcript.

## Output schemas

Pass the narrowest schema for the expected role action. Do not pass the union of every message type when the turn can only produce one of two. A Manager staffing turn, for example, should accept `staffing_decision | decision_request | delegation_decision`, not worker result fields.

Model-side schema enforcement improves reliability; server-side validation and authorization remain mandatory.

## Custom TOMLs

Each `.codex/agents/*.toml` contains:

- `name`, `description`, `developer_instructions`;
- model and reasoning default;
- sandbox default.

The Conductor may override per turn within approved policy. Role Card data is supplied as turn input or a generated bounded instruction segment; do not modify checked-in archetype files for every hire.

## Skills

Invoke skills explicitly in App Server input using both the textual `$skill-name` reference and the structured skill input/path supported by the installed protocol. This makes procedure selection visible and avoids relying on implicit invocation.

Skills are kept small. Their schemas and policy tables stay in `.orchestra`/`docs` and are read only when a branch requires them.

## Worktree manager

```text
create(task, attempt, base_sha):
  verify repository root and safe worktree parent
  git worktree add -b orchestra/<task>/<attempt> <path> <base_sha>
  run approved idempotent initialization hook
  record path, branch, base_sha, writable paths

finalize(result):
  verify changed paths against write domain
  record commit/patch and dirty status
  retain through Team Leader review/integration

cleanup:
  only after integration, cancellation, or explicit retention expiry
  never recursively delete an unvalidated path
```

The runtime sandbox should make unauthorized paths unwritable where possible; path validation is still required at result acceptance.

## Event store and projections

Use an append-only event table plus transactional projections. Suggested event keys:

- `project.initialized`, `phase.changed`, `scope.revised`;
- `agent.started`, `agent.resumed`, `agent.ended`;
- `task.created`, `task.ready`, `task.claimed`, `task.submitted`, `task.accepted`, `task.rejected`;
- `lease.heartbeat`, `lease.expired`;
- `decision.recorded`, `hire.requested`, `hire.decided`, `role.compiled`;
- `drift.raised`, `capsule.invalidated`;
- `gate.planned`, `gate.verdict`;
- `retry.scheduled`, `attempt.orphaned`.

Every command has an idempotency key. Write event and projection in one SQLite/PostgreSQL transaction. Human-readable summaries are derived after commit.

## Scheduler loop

```python
while running:
    reconcile_app_server_threads_and_leases()
    ingest_and_validate_new_envelopes()
    apply_accepted_decisions()
    invalidate_stale_capsules()
    refresh_dependency_ready_tasks()
    reserve_capacity_partitions()
    dispatch_ready_tasks_by_priority()
    project_status_and_metrics()
    sleep_or_wait_for_event()
```

The scheduler does not ask a model what to poll next. Agent judgment enters only through accepted typed decisions/artifacts.

## Retry policy

- transport/process/rate-limit overload → exponential backoff with jitter and cap;
- turn stall → interrupt, preserve events/worktree, classify before retry;
- missing approval/user input in unattended mode → fail with explicit access/authority event;
- schema-invalid response → one corrective turn on the same attempt when safe, then fail structurally;
- stale digest → reject and repackage, no replay;
- failed acceptance/semantic error → Team Leader diagnosis and new repair attempt/task;
- security/sandbox violation → stop affected work and require explicit review.

## Dynamic tools and MCP

Treat dynamic tools/MCP servers as Role Card permissions. Required servers should fail thread startup rather than silently degrading when their capability is mandatory. Network and external data access should be explicit per task and sensitivity class.

## Pilot versus hardened deployment

### Pilot

- local stdio App Server;
- SQLite/WAL single Conductor;
- one repository and three execution slots;
- manual Operator checkpoints;
- local Git worktrees;
- basic status CLI;
- no automatic external tracker writes.

### Production/higher assurance

Add only as triggered by profile:

- authenticated service transport and host isolation;
- durable database/backup and idempotent command bus;
- secret manager and permission profiles;
- audit export and evidence retention;
- tracker integration with write boundaries;
- cost/rate-limit budgets;
- crash/reconciliation chaos tests;
- security, compliance, tenancy, and disaster-recovery controls.
