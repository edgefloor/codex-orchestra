use crate::context::materialize_context;
use crate::state::RunStore;
use crate::{
    Action, AgentHandle, AgentStatus, ExecutionPlan, NativeHost, RunCheckpoint, RunStatus,
    SpawnRequest, Step, StepOutputs, StepStatus, validate_plan,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum RunOutcome {
    Completed(RunCheckpoint),
    Paused(RunCheckpoint),
    Failed(RunCheckpoint),
    Cancelled(RunCheckpoint),
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("workflow validation failed: {0}")]
    Validation(String),
    #[error("run storage failed: {0}")]
    Storage(#[from] std::io::Error),
    #[error("native host failed: {0}")]
    Host(String),
    #[error("runtime task failed: {0}")]
    Join(String),
}

struct ActiveRun {
    cancelled: Arc<AtomicBool>,
    handles: Arc<Mutex<HashMap<String, AgentHandle>>>,
}

pub struct OrchestraRuntime<H: NativeHost> {
    host: Arc<H>,
    active: Arc<Mutex<HashMap<String, ActiveRun>>>,
}

impl<H: NativeHost> Clone for OrchestraRuntime<H> {
    fn clone(&self) -> Self {
        Self {
            host: Arc::clone(&self.host),
            active: Arc::clone(&self.active),
        }
    }
}

impl<H: NativeHost> OrchestraRuntime<H> {
    pub fn new(host: H) -> Self {
        Self {
            host: Arc::new(host),
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(
        &self,
        repository: &Path,
        parent_thread_id: &str,
        plan: ExecutionPlan,
    ) -> Result<RunOutcome, RunError> {
        let errors = validate_plan(&plan);
        if !errors.is_empty() {
            return Err(RunError::Validation(
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
        }
        let plan_bytes = serde_json::to_vec(&plan).expect("plan serializes");
        let hash = format!("{:x}", Sha256::digest(&plan_bytes));
        let run_id = new_run_id(&hash);
        let revision = git_revision(repository)?;
        let (store, checkpoint) = RunStore::create(
            repository,
            &run_id,
            &plan,
            &hash,
            parent_thread_id,
            revision,
        )?;
        self.execute(store, plan, checkpoint).await
    }

    pub async fn resume(&self, repository: &Path, run_id: &str) -> Result<RunOutcome, RunError> {
        self.resume_with_approval(repository, run_id, None).await
    }

    pub async fn resume_with_approval(
        &self,
        repository: &Path,
        run_id: &str,
        approval_decision: Option<&str>,
    ) -> Result<RunOutcome, RunError> {
        let (store, plan, mut checkpoint) = RunStore::open(repository, run_id)?;
        if matches!(
            checkpoint.status,
            RunStatus::Completed | RunStatus::Cancelled
        ) {
            return Ok(if checkpoint.status == RunStatus::Completed {
                RunOutcome::Completed(checkpoint)
            } else {
                RunOutcome::Cancelled(checkpoint)
            });
        }
        let mut approval_decision = approval_decision;
        for step in &plan.steps {
            let state = checkpoint
                .steps
                .get_mut(&step.id)
                .expect("snapshot and state agree");
            if state.status == StepStatus::WaitingApproval
                && let Some(decision) = approval_decision.take()
            {
                state.approval_decision = Some(decision.to_string());
                state.status = StepStatus::Completed;
                store.approval(&step.id, decision)?;
                continue;
            }
            if matches!(
                state.status,
                StepStatus::Running | StepStatus::Retrying | StepStatus::WaitingApproval
            ) {
                if state.attempts >= step.max_attempts
                    && !matches!(step.action, Action::Approval(_))
                {
                    state.status = StepStatus::Failed;
                    state.error = Some("interrupted after attempt budget was exhausted".into());
                } else {
                    state.status = StepStatus::Pending;
                }
            }
        }
        checkpoint.status = RunStatus::Running;
        checkpoint.next_action = "resume dependency-ready steps from checkpoint".into();
        store.save(&checkpoint)?;
        self.execute(store, plan, checkpoint).await
    }

    pub async fn status(&self, repository: &Path, run_id: &str) -> Result<RunCheckpoint, RunError> {
        let (_, _, checkpoint) = RunStore::open(repository, run_id)?;
        Ok(checkpoint)
    }

    pub async fn cancel(&self, repository: &Path, run_id: &str) -> Result<RunCheckpoint, RunError> {
        if let Some(active) = self.active.lock().await.get(run_id) {
            active.cancelled.store(true, Ordering::SeqCst);
            let handles: Vec<_> = active.handles.lock().await.values().cloned().collect();
            for handle in handles {
                self.host.cancel(&handle).await.map_err(RunError::Host)?;
            }
        }
        let (store, _, mut checkpoint) = RunStore::open(repository, run_id)?;
        if matches!(
            checkpoint.status,
            RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled
        ) {
            return Ok(checkpoint);
        }
        checkpoint.status = RunStatus::Cancelled;
        checkpoint.next_action = "run cancelled".into();
        for step in checkpoint.steps.values_mut() {
            if matches!(
                step.status,
                StepStatus::Running | StepStatus::Retrying | StepStatus::WaitingApproval
            ) {
                step.status = StepStatus::Cancelled;
            }
        }
        store.save(&checkpoint)?;
        store.summary(&summary(&checkpoint))?;
        let shared_worktree = shared_worktree_path(&checkpoint.repository, &checkpoint.run_id);
        if shared_worktree.exists() {
            let _ = self
                .host
                .remove_worktree(
                    &checkpoint.parent_thread_id,
                    &checkpoint.repository,
                    &shared_worktree,
                )
                .await;
        }
        Ok(checkpoint)
    }

    async fn execute(
        &self,
        store: RunStore,
        plan: ExecutionPlan,
        mut checkpoint: RunCheckpoint,
    ) -> Result<RunOutcome, RunError> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let handles = Arc::new(Mutex::new(HashMap::new()));
        self.active.lock().await.insert(
            checkpoint.run_id.clone(),
            ActiveRun {
                cancelled: Arc::clone(&cancelled),
                handles: Arc::clone(&handles),
            },
        );
        checkpoint.status = RunStatus::Running;
        store.save(&checkpoint)?;
        self.host
            .emit_activity(
                &checkpoint.parent_thread_id,
                &format!("Orchestra run `{}` started", checkpoint.run_id),
            )
            .await;

        loop {
            if cancelled.load(Ordering::SeqCst) {
                checkpoint.status = RunStatus::Cancelled;
                checkpoint.next_action = "run cancelled".into();
                break;
            }
            if checkpoint
                .steps
                .values()
                .any(|step| step.status == StepStatus::Failed)
            {
                checkpoint.status = RunStatus::Failed;
                checkpoint.next_action = "inspect failed step evidence".into();
                break;
            }
            if checkpoint
                .steps
                .values()
                .all(|step| step.status == StepStatus::Completed)
            {
                checkpoint.status = RunStatus::Completed;
                checkpoint.next_action = "run complete".into();
                break;
            }
            let ready: Vec<_> = plan
                .steps
                .iter()
                .filter(|step| {
                    checkpoint.steps[&step.id].status == StepStatus::Pending
                        && step.needs.iter().all(|dependency| {
                            checkpoint.steps[dependency].status == StepStatus::Completed
                        })
                })
                .take(plan.max_parallel)
                .cloned()
                .collect();
            if ready.is_empty() {
                checkpoint.status = RunStatus::Failed;
                checkpoint.next_action = "no dependency-ready steps remain".into();
                break;
            }

            if let Some(approval) = ready
                .iter()
                .find(|step| matches!(step.action, Action::Approval(_)))
                .cloned()
            {
                checkpoint.steps.get_mut(&approval.id).unwrap().status =
                    StepStatus::WaitingApproval;
                checkpoint.status = RunStatus::WaitingApproval;
                checkpoint.next_action = format!("approval required for `{}`", approval.id);
                store.save(&checkpoint)?;
                let Action::Approval(spec) = &approval.action else {
                    unreachable!()
                };
                match self
                    .host
                    .request_approval(&checkpoint.parent_thread_id, &spec.prompt, &spec.choices)
                    .await
                    .map_err(RunError::Host)?
                {
                    Some(decision) => {
                        let state = checkpoint.steps.get_mut(&approval.id).unwrap();
                        state.approval_decision = Some(decision.clone());
                        state.status = StepStatus::Completed;
                        store.approval(&approval.id, &decision)?;
                        checkpoint.status = RunStatus::Running;
                        checkpoint.next_action = "continue after approval".into();
                        store.save(&checkpoint)?;
                        continue;
                    }
                    None => {
                        break;
                    }
                }
            }

            let dependency_outputs = all_outputs(&checkpoint);
            let mut join_set = JoinSet::new();
            for step in ready
                .into_iter()
                .filter(|step| !matches!(step.action, Action::Approval(_)))
            {
                let state = checkpoint.steps.get_mut(&step.id).unwrap();
                state.status = StepStatus::Running;
                state.attempts += 1;
                let workspace = self
                    .host
                    .create_worktree(
                        &checkpoint.parent_thread_id,
                        &checkpoint.repository,
                        &checkpoint.run_id,
                        &step.id,
                        &step.worktree,
                        &checkpoint.source_revision,
                    )
                    .await
                    .map_err(RunError::Host)?;
                let context = match &step.action {
                    Action::Agent(agent) => {
                        match materialize_context(&workspace, &agent.context, &dependency_outputs) {
                            Ok(context) => context,
                            Err(error) => {
                                if step.worktree == crate::WorktreePolicy::Isolated {
                                    let _ = self
                                        .host
                                        .remove_worktree(
                                            &checkpoint.parent_thread_id,
                                            &checkpoint.repository,
                                            &workspace,
                                        )
                                        .await;
                                }
                                return Err(RunError::Host(error.to_string()));
                            }
                        }
                    }
                    _ => crate::ContextBundle {
                        sha256: format!("{:x}", Sha256::digest([])),
                        content: String::new(),
                        sources: Vec::new(),
                    },
                };
                state.context_sha256 = Some(context.sha256.clone());
                let task = StepTask {
                    host: Arc::clone(&self.host),
                    handles: Arc::clone(&handles),
                    repository: checkpoint.repository.clone(),
                    parent_thread_id: checkpoint.parent_thread_id.clone(),
                    run_id: checkpoint.run_id.clone(),
                    step: step.clone(),
                    attempt: state.attempts,
                    round: state.rounds + 1,
                    workspace,
                    context,
                };
                join_set.spawn(async move {
                    let id = step.id.clone();
                    (id, task.execute().await)
                });
            }
            store.save(&checkpoint)?;
            while let Some(result) = join_set.join_next().await {
                let (step_id, result) = result.map_err(|e| RunError::Join(e.to_string()))?;
                let step = plan.steps.iter().find(|step| step.id == step_id).unwrap();
                let state = checkpoint.steps.get_mut(&step_id).unwrap();
                match result {
                    Ok(result) => {
                        if let Some(evidence) = result.check_evidence {
                            store.evidence(&step_id, state.attempts, &evidence)?;
                        }
                        if let Some(error) = result.error {
                            state.error = Some(error);
                            state.status = if state.attempts < step.max_attempts {
                                StepStatus::Pending
                            } else {
                                StepStatus::Failed
                            };
                            store.save(&checkpoint)?;
                            continue;
                        }
                        let previous_outputs = state.outputs.clone();
                        state.final_response = result.final_response;
                        state.agent = result.agent;
                        state.outputs = result.outputs;
                        state.error = None;
                        state.rounds += 1;
                        let repeat = step.repeat.as_ref();
                        let condition_met = repeat.is_none_or(|policy| {
                            state.outputs.get(&policy.until_output) == Some(&policy.equals)
                        });
                        if condition_met {
                            state.status = StepStatus::Completed;
                            store.output(&step_id, &state.outputs)?;
                            self.host
                                .persist_outputs(&checkpoint.run_id, &step_id, &state.outputs)
                                .await;
                        } else if let Some(policy) = repeat {
                            if state.rounds < policy.max_rounds {
                                if policy.stop_on_no_progress && state.outputs == previous_outputs {
                                    state.status = StepStatus::Failed;
                                    state.error = Some(
                                        "repeat stopped because outputs made no progress".into(),
                                    );
                                } else {
                                    state.status = StepStatus::Pending;
                                    state.attempts = 0;
                                }
                            } else {
                                state.status = StepStatus::Failed;
                                state.error =
                                    Some("repeat condition was not met before max_rounds".into());
                            }
                        }
                    }
                    Err(error) => {
                        state.error = Some(error);
                        state.status = if state.attempts < step.max_attempts {
                            StepStatus::Pending
                        } else {
                            StepStatus::Failed
                        };
                    }
                }
                store.save(&checkpoint)?;
            }
        }

        store.save(&checkpoint)?;
        store.summary(&summary(&checkpoint))?;
        self.active.lock().await.remove(&checkpoint.run_id);
        if checkpoint.status != RunStatus::WaitingApproval {
            let shared_worktree = shared_worktree_path(&checkpoint.repository, &checkpoint.run_id);
            if shared_worktree.exists() {
                let _ = self
                    .host
                    .remove_worktree(
                        &checkpoint.parent_thread_id,
                        &checkpoint.repository,
                        &shared_worktree,
                    )
                    .await;
            }
        }
        self.host
            .emit_activity(
                &checkpoint.parent_thread_id,
                &format!(
                    "Orchestra run `{}` finished as {:?}",
                    checkpoint.run_id, checkpoint.status
                ),
            )
            .await;
        Ok(match checkpoint.status {
            RunStatus::Completed => RunOutcome::Completed(checkpoint),
            RunStatus::Cancelled => RunOutcome::Cancelled(checkpoint),
            RunStatus::WaitingApproval => RunOutcome::Paused(checkpoint),
            _ => RunOutcome::Failed(checkpoint),
        })
    }
}

struct StepTask<H: NativeHost> {
    host: Arc<H>,
    handles: Arc<Mutex<HashMap<String, AgentHandle>>>,
    repository: PathBuf,
    parent_thread_id: String,
    run_id: String,
    step: Step,
    attempt: u32,
    round: u32,
    workspace: PathBuf,
    context: crate::ContextBundle,
}

struct StepResult {
    outputs: StepOutputs,
    final_response: Option<String>,
    agent: Option<AgentHandle>,
    check_evidence: Option<CheckEvidence>,
    error: Option<String>,
}

#[derive(Serialize)]
struct CheckEvidence {
    argv: Vec<String>,
    cwd: Option<String>,
    timeout_ms: u64,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

impl<H: NativeHost> StepTask<H> {
    async fn execute(self) -> Result<StepResult, String> {
        let result = match &self.step.action {
            Action::Agent(agent) => {
                let delegation = if agent.allow_delegation {
                    "Recursive delegation is explicitly allowed for this step."
                } else {
                    "Do not spawn or delegate to child agents."
                };
                let output_contract = if agent.outputs.is_empty() {
                    "Return a JSON object.".into()
                } else {
                    format!(
                        "Return exactly one JSON object containing these keys: {}.",
                        agent.outputs.join(", ")
                    )
                };
                let prompt = format!(
                    "{}\n\n{}\n{}\nContext SHA-256: {}\n{}",
                    agent.prompt,
                    delegation,
                    output_contract,
                    self.context.sha256,
                    self.context.content
                );
                let request = SpawnRequest {
                    parent_thread_id: self.parent_thread_id.clone(),
                    task_name: format!(
                        "orchestra_{}_{}_r{}_a{}",
                        self.run_id
                            .replace(|character: char| !character.is_ascii_alphanumeric(), ""),
                        self.step.id.replace('-', "_"),
                        self.round,
                        self.attempt
                    ),
                    prompt,
                    cwd: self.workspace.clone(),
                    model: agent.model.clone(),
                    reasoning_effort: agent.reasoning_effort.clone(),
                    service_tier: agent.service_tier.clone(),
                    fork_turns: agent.fork_turns.clone(),
                    allow_delegation: agent.allow_delegation,
                };
                let handle = self.host.spawn(request).await?;
                self.handles
                    .lock()
                    .await
                    .insert(self.step.id.clone(), handle.clone());
                let _ = self.host.status(&handle).await?;
                let outcome = self.host.wait(&handle).await?;
                self.handles.lock().await.remove(&self.step.id);
                match outcome.status {
                    AgentStatus::Completed => {
                        let response = outcome.final_response.ok_or_else(|| {
                            "agent completed without a final response".to_string()
                        })?;
                        let value: Value = serde_json::from_str(&response)
                            .map_err(|e| format!("malformed agent output: {e}"))?;
                        let object = value
                            .as_object()
                            .ok_or_else(|| "agent output must be a JSON object".to_string())?;
                        let mut outputs = StepOutputs::new();
                        for name in &agent.outputs {
                            outputs.insert(
                                name.clone(),
                                object
                                    .get(name)
                                    .cloned()
                                    .ok_or_else(|| format!("agent output is missing `{name}`"))?,
                            );
                        }
                        if agent.outputs.is_empty() {
                            outputs.extend(object.clone());
                        }
                        Ok(StepResult {
                            outputs,
                            final_response: Some(response),
                            agent: Some(handle),
                            check_evidence: None,
                            error: None,
                        })
                    }
                    AgentStatus::Cancelled => Err("agent was cancelled".into()),
                    AgentStatus::Failed(error) => Err(format!("agent failed: {error}")),
                    status => Err(format!("agent ended in non-final status {status:?}")),
                }
            }
            Action::Check(check) => {
                let cwd = check.cwd.as_ref().map(|value| self.workspace.join(value));
                let outcome = self
                    .host
                    .run_command(
                        &self.parent_thread_id,
                        &self.workspace,
                        &check.command,
                        cwd.as_deref(),
                        check.timeout_ms,
                    )
                    .await?;
                let evidence = CheckEvidence {
                    argv: check.command.clone(),
                    cwd: check.cwd.clone(),
                    timeout_ms: check.timeout_ms,
                    exit_code: outcome.exit_code,
                    stdout: outcome.stdout,
                    stderr: outcome.stderr,
                };
                if evidence.exit_code == 0 {
                    Ok(StepResult {
                        outputs: BTreeMap::from([("passed".into(), Value::Bool(true))]),
                        final_response: None,
                        agent: None,
                        check_evidence: Some(evidence),
                        error: None,
                    })
                } else {
                    Ok(StepResult {
                        outputs: BTreeMap::from([("passed".into(), Value::Bool(false))]),
                        final_response: None,
                        agent: None,
                        error: Some(format!("check exited with {}", evidence.exit_code)),
                        check_evidence: Some(evidence),
                    })
                }
            }
            Action::Approval(_) => unreachable!(),
        };
        if self.step.worktree == crate::WorktreePolicy::Isolated {
            let _ = self
                .host
                .remove_worktree(&self.parent_thread_id, &self.repository, &self.workspace)
                .await;
        }
        result
    }
}

fn all_outputs(checkpoint: &RunCheckpoint) -> BTreeMap<String, StepOutputs> {
    checkpoint
        .steps
        .iter()
        .map(|(id, state)| (id.clone(), state.outputs.clone()))
        .collect()
}
fn new_run_id(hash: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{millis}-{}", &hash[..12])
}
fn shared_worktree_path(repository: &Path, run_id: &str) -> PathBuf {
    repository
        .join(".codex/orchestra/worktrees")
        .join(format!("{run_id}-shared"))
}
fn git_revision(repository: &Path) -> Result<String, std::io::Error> {
    let snapshot = std::process::Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["stash", "create", "codex-orchestra run snapshot"])
        .output()?;
    if snapshot.status.success() {
        let revision = String::from_utf8_lossy(&snapshot.stdout).trim().to_string();
        if !revision.is_empty() {
            return Ok(revision);
        }
    }
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    } else {
        Ok("unborn".into())
    }
}
fn summary(checkpoint: &RunCheckpoint) -> String {
    let mut text = format!(
        "# Orchestra run `{}`\n\nStatus: `{:?}`\n\n",
        checkpoint.run_id, checkpoint.status
    );
    for (id, step) in &checkpoint.steps {
        text.push_str(&format!(
            "- `{id}`: `{:?}` (attempts {}, rounds {})",
            step.status, step.attempts, step.rounds
        ));
        if let Some(error) = &step.error {
            text.push_str(&format!(" — {error}"));
        }
        text.push('\n');
    }
    text.push_str(&format!("\nNext action: {}\n", checkpoint.next_action));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentOutcome, CommandOutcome, ForkTurns, WorktreePolicy};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    struct FakeHost {
        responses: Mutex<Vec<String>>,
        approvals: Mutex<Vec<Option<String>>>,
        exit_code: i32,
        spawned: Mutex<Vec<SpawnRequest>>,
        cancelled: AtomicUsize,
        running: AtomicUsize,
        max_running: AtomicUsize,
    }
    impl FakeHost {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().rev().map(Into::into).collect()),
                approvals: Mutex::new(vec![]),
                exit_code: 0,
                spawned: Mutex::new(vec![]),
                cancelled: AtomicUsize::new(0),
                running: AtomicUsize::new(0),
                max_running: AtomicUsize::new(0),
            }
        }
    }
    #[async_trait]
    impl NativeHost for FakeHost {
        async fn spawn(&self, request: SpawnRequest) -> Result<AgentHandle, String> {
            self.spawned.lock().await.push(request.clone());
            let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_running.fetch_max(now, Ordering::SeqCst);
            Ok(AgentHandle {
                thread_id: format!("t{now}"),
                task_path: request.task_name,
                parent_thread_id: request.parent_thread_id,
            })
        }
        async fn status(&self, _: &AgentHandle) -> Result<AgentStatus, String> {
            Ok(AgentStatus::Running)
        }
        async fn wait(&self, _: &AgentHandle) -> Result<AgentOutcome, String> {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            self.running.fetch_sub(1, Ordering::SeqCst);
            Ok(AgentOutcome {
                status: AgentStatus::Completed,
                final_response: self.responses.lock().await.pop(),
            })
        }
        async fn cancel(&self, _: &AgentHandle) -> Result<(), String> {
            self.cancelled.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn run_command(
            &self,
            _: &str,
            _: &Path,
            _: &[String],
            _: Option<&Path>,
            _: u64,
        ) -> Result<CommandOutcome, String> {
            Ok(CommandOutcome {
                exit_code: self.exit_code,
                stdout: "ok".into(),
                stderr: String::new(),
            })
        }
        async fn create_worktree(
            &self,
            _: &str,
            repository: &Path,
            run_id: &str,
            step_id: &str,
            policy: &WorktreePolicy,
            _: &str,
        ) -> Result<PathBuf, String> {
            let path = if *policy == WorktreePolicy::Shared {
                repository
                    .join(".codex/orchestra/worktrees")
                    .join(format!("{run_id}-shared"))
            } else {
                repository
                    .join(".codex/orchestra/worktrees")
                    .join(format!("{run_id}-{step_id}"))
            };
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
            Ok(path)
        }
        async fn remove_worktree(&self, _: &str, _: &Path, path: &Path) -> Result<(), String> {
            std::fs::remove_dir_all(path).map_err(|e| e.to_string())
        }
        async fn request_approval(
            &self,
            _: &str,
            _: &str,
            _: &[String],
        ) -> Result<Option<String>, String> {
            Ok(self.approvals.lock().await.pop().unwrap_or(None))
        }
        async fn emit_activity(&self, _: &str, _: &str) {}
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(dir.path())
            .status()
            .unwrap();
        dir
    }

    struct GitHost {
        response: String,
        workspaces: Mutex<HashMap<String, PathBuf>>,
    }

    #[async_trait]
    impl NativeHost for GitHost {
        async fn spawn(&self, request: SpawnRequest) -> Result<AgentHandle, String> {
            let thread_id = request.task_name.clone();
            self.workspaces
                .lock()
                .await
                .insert(thread_id.clone(), request.cwd);
            Ok(AgentHandle {
                thread_id,
                task_path: request.task_name,
                parent_thread_id: request.parent_thread_id,
            })
        }

        async fn status(&self, _: &AgentHandle) -> Result<AgentStatus, String> {
            Ok(AgentStatus::Running)
        }

        async fn wait(&self, handle: &AgentHandle) -> Result<AgentOutcome, String> {
            let workspace = self.workspaces.lock().await[&handle.thread_id].clone();
            std::fs::create_dir_all(workspace.join("scope")).map_err(|e| e.to_string())?;
            std::fs::write(workspace.join("scope/change.txt"), "integrated\n")
                .map_err(|e| e.to_string())?;
            Ok(AgentOutcome {
                status: AgentStatus::Completed,
                final_response: Some(self.response.clone()),
            })
        }

        async fn cancel(&self, _: &AgentHandle) -> Result<(), String> {
            Ok(())
        }

        async fn run_command(
            &self,
            _: &str,
            repository: &Path,
            argv: &[String],
            cwd: Option<&Path>,
            _: u64,
        ) -> Result<CommandOutcome, String> {
            if argv == ["assert-integrated"] {
                let content = std::fs::read_to_string(repository.join("scope/change.txt"));
                return Ok(CommandOutcome {
                    exit_code: i32::from(
                        !content
                            .as_deref()
                            .is_ok_and(|value| value == "integrated\n"),
                    ),
                    stdout: content.unwrap_or_default(),
                    stderr: String::new(),
                });
            }
            let output = std::process::Command::new(&argv[0])
                .args(&argv[1..])
                .current_dir(cwd.unwrap_or(repository))
                .output()
                .map_err(|e| e.to_string())?;
            Ok(CommandOutcome {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }

        async fn create_worktree(
            &self,
            _: &str,
            repository: &Path,
            run_id: &str,
            step_id: &str,
            policy: &WorktreePolicy,
            source_revision: &str,
        ) -> Result<PathBuf, String> {
            let suffix = if *policy == WorktreePolicy::Shared {
                "shared".to_string()
            } else {
                step_id.to_string()
            };
            let path = repository
                .join(".codex/orchestra/worktrees")
                .join(format!("{run_id}-{suffix}"));
            if path.exists() {
                return Ok(path);
            }
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(repository)
                .args(["worktree", "add", "--detach"])
                .arg(&path)
                .arg(source_revision)
                .output()
                .map_err(|e| e.to_string())?;
            if output.status.success() {
                Ok(path)
            } else {
                Err(String::from_utf8_lossy(&output.stderr).into_owned())
            }
        }

        async fn remove_worktree(
            &self,
            _: &str,
            repository: &Path,
            path: &Path,
        ) -> Result<(), String> {
            let output = std::process::Command::new("git")
                .arg("-C")
                .arg(repository)
                .args(["worktree", "remove", "--force"])
                .arg(path)
                .output()
                .map_err(|e| e.to_string())?;
            if output.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).into_owned())
            }
        }

        async fn request_approval(
            &self,
            _: &str,
            _: &str,
            _: &[String],
        ) -> Result<Option<String>, String> {
            Ok(None)
        }

        async fn emit_activity(&self, _: &str, _: &str) {}
    }

    fn committed_repo() -> tempfile::TempDir {
        let dir = repo();
        std::fs::write(dir.path().join("README.md"), "source\n").unwrap();
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(["add", "."])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args([
                    "-c",
                    "user.name=Orchestra Test",
                    "-c",
                    "user.email=orchestra@example.invalid",
                    "commit",
                    "-qm",
                    "source",
                ])
                .status()
                .unwrap()
                .success()
        );
        dir
    }
    fn agent(id: &str, needs: Vec<&str>) -> Step {
        Step {
            id: id.into(),
            needs: needs.into_iter().map(Into::into).collect(),
            max_attempts: 1,
            repeat: None,
            worktree: WorktreePolicy::Shared,
            write_scope: vec![],
            action: Action::Agent(crate::AgentStep {
                prompt: "do it".into(),
                model: "gpt-5.4".into(),
                reasoning_effort: Some("high".into()),
                service_tier: None,
                fork_turns: ForkTurns::None,
                context: vec![],
                outputs: vec!["ok".into()],
                allow_delegation: false,
            }),
        }
    }

    #[tokio::test]
    async fn schedules_parallel_agents_with_explicit_native_settings() {
        let host = FakeHost::new(vec![r#"{"ok":true}"#, r#"{"ok":true}"#]);
        let runtime = OrchestraRuntime::new(host);
        let result = runtime
            .run(
                repo().path(),
                "parent",
                ExecutionPlan {
                    name: "parallel".into(),
                    description: String::new(),
                    max_parallel: 2,
                    steps: vec![
                        agent("inspect-runtime", vec![]),
                        agent("inspect-tests", vec![]),
                    ],
                },
            )
            .await
            .unwrap();
        assert!(matches!(result, RunOutcome::Completed(_)));
        assert_eq!(runtime.host.max_running.load(Ordering::SeqCst), 2);
        let requests = runtime.host.spawned.lock().await;
        assert!(requests.iter().all(|r| {
            r.cwd
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.ends_with("-shared"))
                && r.cwd.is_absolute()
                && r.model == "gpt-5.4"
                && r.reasoning_effort.as_deref() == Some("high")
                && r.fork_turns == ForkTurns::None
                && !r.allow_delegation
                && r.task_name.chars().all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
                })
        }));
        assert!(
            requests
                .iter()
                .any(|request| request.task_name.contains("inspect_runtime"))
        );
    }

    #[tokio::test]
    async fn malformed_output_retries_then_completes() {
        let host = FakeHost::new(vec!["not-json", r#"{"ok":true}"#]);
        let runtime = OrchestraRuntime::new(host);
        let mut step = agent("a", vec![]);
        step.max_attempts = 2;
        let RunOutcome::Completed(state) = runtime
            .run(
                repo().path(),
                "parent",
                ExecutionPlan {
                    name: "retry".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![step],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(state.steps["a"].attempts, 2);
    }

    #[tokio::test]
    async fn bounded_repeat_exhausts() {
        let host = FakeHost::new(vec![r#"{"ok":false}"#, r#"{"ok":false}"#]);
        let runtime = OrchestraRuntime::new(host);
        let mut step = agent("a", vec![]);
        step.repeat = Some(crate::RepeatPolicy {
            max_rounds: 2,
            until_output: "ok".into(),
            equals: Value::Bool(true),
            stop_on_no_progress: false,
        });
        let RunOutcome::Failed(state) = runtime
            .run(
                repo().path(),
                "parent",
                ExecutionPlan {
                    name: "repeat".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![step],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(state.steps["a"].rounds, 2);
        let requests = runtime.host.spawned.lock().await;
        assert_eq!(requests.len(), 2);
        assert_ne!(requests[0].task_name, requests[1].task_name);
        assert!(requests[0].task_name.contains("_r1_a1"));
        assert!(requests[1].task_name.contains("_r2_a1"));
    }

    #[tokio::test]
    async fn approval_can_pause_and_resume() {
        let runtime = OrchestraRuntime::new(FakeHost::new(vec![]));
        let step = Step {
            id: "approve".into(),
            needs: vec![],
            max_attempts: 1,
            repeat: None,
            worktree: WorktreePolicy::Shared,
            write_scope: vec![],
            action: Action::Approval(crate::ApprovalStep {
                prompt: "ship?".into(),
                choices: vec!["yes".into()],
            }),
        };
        let dir = repo();
        let RunOutcome::Paused(state) = runtime
            .run(
                dir.path(),
                "parent",
                ExecutionPlan {
                    name: "approval".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![step],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        runtime.host.approvals.lock().await.push(Some("yes".into()));
        assert!(matches!(
            runtime.resume(dir.path(), &state.run_id).await.unwrap(),
            RunOutcome::Completed(_)
        ));
        assert!(
            dir.path()
                .join(".codex/orchestra/runs")
                .join(&state.run_id)
                .join("summary.md")
                .is_file()
        );
    }

    #[tokio::test]
    async fn failed_check_persists_sandbox_evidence_and_exhausts() {
        let mut host = FakeHost::new(vec![]);
        host.exit_code = 7;
        let runtime = OrchestraRuntime::new(host);
        let step = Step {
            id: "check".into(),
            needs: vec![],
            max_attempts: 1,
            repeat: None,
            worktree: WorktreePolicy::Isolated,
            write_scope: vec![],
            action: Action::Check(crate::CheckStep {
                command: vec!["false".into()],
                cwd: None,
                timeout_ms: 1000,
            }),
        };
        let dir = repo();
        let RunOutcome::Failed(state) = runtime
            .run(
                dir.path(),
                "parent",
                ExecutionPlan {
                    name: "check".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![step],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        let evidence = dir
            .path()
            .join(".codex/orchestra/runs")
            .join(&state.run_id)
            .join("evidence/checks/check-1.json");
        assert!(evidence.is_file());
        assert_eq!(
            serde_json::from_slice::<Value>(&std::fs::read(evidence).unwrap()).unwrap()["exit_code"],
            7
        );
        assert!(
            !dir.path()
                .join(".codex/orchestra/worktrees")
                .join(format!("{}-check", state.run_id))
                .exists()
        );
    }

    #[tokio::test]
    async fn missing_final_response_exhausts_attempt_budget() {
        let runtime = OrchestraRuntime::new(FakeHost::new(vec![]));
        let RunOutcome::Failed(state) = runtime
            .run(
                repo().path(),
                "parent",
                ExecutionPlan {
                    name: "exhaust".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![agent("a", vec![])],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(state.steps["a"].attempts, 1);
        assert!(
            state.steps["a"]
                .error
                .as_deref()
                .unwrap()
                .contains("final response")
        );
    }

    #[tokio::test]
    async fn cancellation_interrupts_active_native_handle() {
        let runtime = OrchestraRuntime::new(FakeHost::new(vec![r#"{"ok":true}"#]));
        let dir = repo();
        let runner = runtime.clone();
        let repository = dir.path().to_path_buf();
        let task = tokio::spawn(async move {
            runner
                .run(
                    &repository,
                    "parent",
                    ExecutionPlan {
                        name: "cancel".into(),
                        description: String::new(),
                        max_parallel: 1,
                        steps: vec![agent("a", vec![])],
                    },
                )
                .await
                .unwrap()
        });
        let runs = dir.path().join(".codex/orchestra/runs");
        let run_id = loop {
            if let Ok(mut entries) = std::fs::read_dir(&runs)
                && let Some(Ok(entry)) = entries.next()
            {
                break entry.file_name().to_string_lossy().into_owned();
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        };
        while runtime.host.running.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
        runtime.cancel(dir.path(), &run_id).await.unwrap();
        assert!(matches!(task.await.unwrap(), RunOutcome::Cancelled(_)));
        assert_eq!(runtime.host.cancelled.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cancel_does_not_rewrite_completed_run() {
        let runtime = OrchestraRuntime::new(FakeHost::new(vec![r#"{"ok":true}"#]));
        let dir = repo();
        let RunOutcome::Completed(state) = runtime
            .run(
                dir.path(),
                "parent",
                ExecutionPlan {
                    name: "done".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![agent("a", vec![])],
                },
            )
            .await
            .unwrap()
        else {
            panic!()
        };
        let summary_path = dir
            .path()
            .join(".codex/orchestra/runs")
            .join(&state.run_id)
            .join("summary.md");
        let before = std::fs::read_to_string(&summary_path).unwrap();
        let cancelled = runtime.cancel(dir.path(), &state.run_id).await.unwrap();
        let after = std::fs::read_to_string(&summary_path).unwrap();
        assert_eq!(cancelled.status, RunStatus::Completed);
        assert_eq!(before, after);
    }

    #[tokio::test]
    async fn isolated_changes_are_verified_in_shared_worktree_then_cleaned_up() {
        let runtime = OrchestraRuntime::new(GitHost {
            response: r#"{"ok":true}"#.into(),
            workspaces: Mutex::new(HashMap::new()),
        });
        let dir = committed_repo();
        let mut implement = agent("implement", vec![]);
        implement.worktree = WorktreePolicy::Isolated;
        implement.write_scope = vec!["scope/".into()];
        let check = Step {
            id: "verify".into(),
            needs: vec!["implement".into()],
            max_attempts: 1,
            repeat: None,
            worktree: WorktreePolicy::Shared,
            write_scope: vec![],
            action: Action::Check(crate::CheckStep {
                command: vec!["assert-integrated".into()],
                cwd: None,
                timeout_ms: 1000,
            }),
        };
        let approval = Step {
            id: "accept".into(),
            needs: vec!["verify".into()],
            max_attempts: 1,
            repeat: None,
            worktree: WorktreePolicy::Shared,
            write_scope: vec![],
            action: Action::Approval(crate::ApprovalStep {
                prompt: "accept?".into(),
                choices: vec!["accept".into(), "reject".into()],
            }),
        };

        let RunOutcome::Paused(state) = runtime
            .run(
                dir.path(),
                "parent",
                ExecutionPlan {
                    name: "integrate".into(),
                    description: String::new(),
                    max_parallel: 1,
                    steps: vec![implement, check, approval],
                },
            )
            .await
            .unwrap()
        else {
            panic!("run should pause only after verifying the integrated change")
        };
        let worktrees = dir.path().join(".codex/orchestra/worktrees");
        assert_eq!(state.steps["verify"].outputs["passed"], Value::Bool(true));
        assert_eq!(
            std::fs::read_to_string(
                worktrees
                    .join(format!("{}-shared", state.run_id))
                    .join("scope/change.txt")
            )
            .unwrap(),
            "integrated\n"
        );
        assert!(!worktrees.join(format!("{}-implement", state.run_id)).exists());
        assert!(
            dir.path()
                .join(".codex/orchestra/runs")
                .join(&state.run_id)
                .join("evidence/changes/implement-1.patch")
                .is_file()
        );

        assert!(matches!(
            runtime
                .resume_with_approval(dir.path(), &state.run_id, Some("reject"))
                .await
                .unwrap(),
            RunOutcome::Completed(_)
        ));
        assert!(!worktrees.join(format!("{}-shared", state.run_id)).exists());
    }
}
