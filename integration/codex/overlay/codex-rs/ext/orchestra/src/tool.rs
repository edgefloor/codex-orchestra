use async_trait::async_trait;
use codex_core::ThreadManager;
use codex_core::orchestra::{
    OrchestraAgentHandle, OrchestraCommandRequest, OrchestraControl, OrchestraForkTurns,
    OrchestraSkillRequirement, OrchestraSpawnRequest,
};
use codex_extension_api::{
    ExtensionData, FunctionCallError, JsonToolOutput, ToolCall, ToolContributor, ToolExecutor,
    ToolName, ToolSpec,
};
use codex_http_client::{HttpClient, build_reqwest_client_with_custom_ca};
use codex_orchestra_core::{
    AgentHandle, AgentOutcome, AgentStatus, AutomationClaimReconciliation, AutomationClaimStatus,
    AutomationEffectExecution, AutomationEffectStatus, AutomationGatePolicy, AutomationIssue,
    AutomationProfile, AutomationQueueCategory, AutomationQueuePage, AutomationRootCheckpoint,
    AutomationRootStatus, AutomationRunStart, AutomationRunStore, AutomationSecretKind,
    AutomationTrackerCommentRequest,
    AutomationValidationRequest, AutomationValidationResult, CommandOutcome,
    ExecutionHistoryRecord, ExecutionHistorySource, ExecutionPlan, ExecutionQueryBudget,
    ExecutionQueryLimits, ExecutionQueryResult, ExecutionQueryService, ExecutionSelector,
    ForkTurns, HistoryCursor, HistoryReadRequest, InheritedCodexPolicy, NativeHost,
    OrchestraRuntime, ResolvedSkill, RunCheckpoint, RunDigest, RunOutcome, RunStatus,
    SkillIdentity, SkillRequirement, SkillSourceKind, SkillToolDependency, SpawnRequest,
    WorktreePolicy, compile_workflow, normalize_linear_issue, normalize_linear_issue_page,
    repository_revision, validate_automation_profile,
};
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AgentStatus as CodexAgentStatus;
use codex_protocol::protocol::OrchestraLifecycleKind;
use codex_protocol::protocol::OrchestraPromotionStatus as CodexOrchestraPromotionStatus;
use codex_protocol::protocol::OrchestraRolloutItem;
use codex_protocol::protocol::OrchestraRunProjection as CodexOrchestraRunProjection;
use codex_protocol::protocol::OrchestraRunStatus as CodexOrchestraRunStatus;
use codex_protocol::protocol::OrchestraStepProjection as CodexOrchestraStepProjection;
use codex_protocol::protocol::OrchestraStepStatus as CodexOrchestraStepStatus;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::{AgentPath, ThreadId};
use codex_tools::{JsonSchema, ResponsesApiTool};
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationLinearReadKind {
    Candidates,
    Terminal,
    Refresh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationLinearReadStatus {
    Ready,
    Skipped,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutomationLinearRead {
    pub status: AutomationLinearReadStatus,
    pub issues: Vec<AutomationIssue>,
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
    pub next_action: String,
}

const LINEAR_ISSUE_FIELDS: &str = r#"
id identifier title description priority branchName url createdAt updatedAt
state { name }
labels(first: 50) { nodes { name } }
relations(first: 50) { nodes { type relatedIssue { id identifier state { name } } issue { id identifier state { name } } } }
inverseRelations(first: 50) { nodes { type relatedIssue { id identifier state { name } } issue { id identifier state { name } } } }
"#;

#[derive(Clone)]
struct CodexHost {
    manager: Weak<ThreadManager>,
}

#[derive(Clone)]
struct CodexExecutionHistory {
    manager: Weak<ThreadManager>,
}

#[async_trait]
impl ExecutionHistorySource for CodexExecutionHistory {
    async fn read(
        &self,
        request: &HistoryReadRequest,
    ) -> Result<Vec<ExecutionHistoryRecord>, String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(&request.parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;

        let mut events = if let Some(state_db) = thread.state_db() {
            state_db
                .orchestra_task_snapshot(thread_id)
                .await
                .map_err(|error| error.to_string())?
                .map(|snapshot| snapshot.replay)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        if events.is_empty() {
            events = thread
                .load_history(/*include_archived*/ true)
                .await
                .map_err(|error| error.to_string())?
                .items
                .into_iter()
                .filter_map(|item| match item {
                    RolloutItem::Orchestra(event) => Some(event),
                    _ => None,
                })
                .collect();
        }
        events.sort_by_key(|event| (event.sequence, event.event_id.clone(), event.revision));
        let after = request.after.as_ref();
        Ok(events
            .into_iter()
            .filter(|event| event.run_id == request.run_id)
            .filter(|event| {
                after.is_none_or(|cursor| {
                    (event.sequence, &event.event_id, event.revision)
                        > (cursor.sequence, &cursor.item_id, cursor.revision)
                })
            })
            .take(request.limit)
            .map(|event| ExecutionHistoryRecord {
                sequence: event.sequence,
                item_id: event.event_id,
                revision: event.revision,
                kind: lifecycle_kind_name(event.kind).into(),
                step_id: None,
                summary: format!(
                    "run {} {} ({})",
                    event.run_id,
                    lifecycle_kind_name(event.kind),
                    rollout_status_name(event.projection.status)
                ),
            })
            .collect())
    }
}

fn lifecycle_kind_name(kind: OrchestraLifecycleKind) -> &'static str {
    match kind {
        OrchestraLifecycleKind::Invoked => "invoked",
        OrchestraLifecycleKind::Resumed => "resumed",
        OrchestraLifecycleKind::Cancelled => "cancelled",
        OrchestraLifecycleKind::Recovered => "recovered",
    }
}

fn rollout_status_name(status: CodexOrchestraRunStatus) -> &'static str {
    match status {
        CodexOrchestraRunStatus::Pending => "pending",
        CodexOrchestraRunStatus::Running => "running",
        CodexOrchestraRunStatus::WaitingApproval => "waiting approval",
        CodexOrchestraRunStatus::Completed => "completed",
        CodexOrchestraRunStatus::Failed => "failed",
        CodexOrchestraRunStatus::Cancelled => "cancelled",
    }
}

fn is_nonterminal_rollout_status(status: CodexOrchestraRunStatus) -> bool {
    matches!(
        status,
        CodexOrchestraRunStatus::Pending
            | CodexOrchestraRunStatus::Running
            | CodexOrchestraRunStatus::WaitingApproval
    )
}

impl CodexHost {
    async fn control(&self, parent: &str) -> Result<OrchestraControl, String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id = ThreadId::from_string(parent).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        Ok(thread.orchestra_control().await)
    }
}

#[async_trait]
impl NativeHost for CodexHost {
    async fn resolve_skills(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        source_revision: &str,
        requirements: &[SkillRequirement],
    ) -> Result<Vec<ResolvedSkill>, String> {
        let resolved = self
            .control(parent_thread_id)
            .await?
            .resolve_skills(
                AbsolutePathBuf::try_from(repository.to_path_buf())
                    .map_err(|error| error.to_string())?,
                source_revision,
                &requirements
                    .iter()
                    .map(|requirement| OrchestraSkillRequirement {
                        name: requirement.name.clone(),
                        resources: requirement.resources.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|error| error.to_string())?;
        Ok(resolved
            .into_iter()
            .map(|skill| ResolvedSkill {
                requirement: skill.requirement,
                identity: SkillIdentity {
                    canonical_name: skill.canonical_name,
                    source_kind: match skill.source_kind {
                        codex_core::orchestra::OrchestraSkillSourceKind::Admin => {
                            SkillSourceKind::Admin
                        }
                        codex_core::orchestra::OrchestraSkillSourceKind::User => {
                            SkillSourceKind::User
                        }
                        codex_core::orchestra::OrchestraSkillSourceKind::Repo => {
                            SkillSourceKind::Repo
                        }
                        codex_core::orchestra::OrchestraSkillSourceKind::System => {
                            SkillSourceKind::System
                        }
                    },
                    source_locator: skill.source_locator,
                    plugin_id: skill.plugin_id,
                },
                instructions: skill.instructions,
                resources: skill.resources,
                tool_dependencies: skill
                    .tool_dependencies
                    .into_iter()
                    .map(|tool| SkillToolDependency {
                        kind: tool.kind,
                        value: tool.value,
                        description: tool.description,
                        transport: tool.transport,
                        command: tool.command,
                        url: tool.url,
                    })
                    .collect(),
            })
            .collect())
    }
    async fn spawn(&self, request: SpawnRequest) -> Result<AgentHandle, String> {
        let control = self.control(&request.parent_thread_id).await?;
        let reasoning_effort = request
            .reasoning_effort
            .map(|value| {
                serde_json::from_value::<ReasoningEffort>(Value::String(value))
                    .map_err(|error| error.to_string())
            })
            .transpose()?;
        let fork_turns = match request.fork_turns {
            ForkTurns::None => OrchestraForkTurns::None,
            ForkTurns::All => OrchestraForkTurns::All,
            ForkTurns::Last(value) => OrchestraForkTurns::Last(value),
        };
        let handle = control
            .spawn(OrchestraSpawnRequest {
                task_name: request.task_name,
                prompt: request.prompt,
                skill_context: request.skill_context,
                cwd: AbsolutePathBuf::try_from(request.cwd).map_err(|error| error.to_string())?,
                model: request.model,
                reasoning_effort,
                service_tier: request.service_tier,
                fork_turns,
                allow_delegation: request.allow_delegation,
                minimum_descendant_depth: request.minimum_descendant_depth,
            })
            .await
            .map_err(|error| error.to_string())?;
        Ok(AgentHandle {
            thread_id: handle.thread_id.to_string(),
            task_path: handle.task_path.to_string(),
            parent_thread_id: request.parent_thread_id,
        })
    }

    async fn status(&self, handle: &AgentHandle) -> Result<AgentStatus, String> {
        let control = self.control(&handle.parent_thread_id).await?;
        Ok(map_status(control.status(&native_handle(handle)?).await))
    }

    async fn wait(&self, handle: &AgentHandle) -> Result<AgentOutcome, String> {
        let control = self.control(&handle.parent_thread_id).await?;
        let status = control
            .wait(&native_handle(handle)?)
            .await
            .map_err(|error| error.to_string())?;
        let final_response = match &status {
            CodexAgentStatus::Completed(message) => message.clone(),
            _ => None,
        };
        Ok(AgentOutcome {
            status: map_status(status),
            final_response,
        })
    }

    async fn cancel(&self, handle: &AgentHandle) -> Result<(), String> {
        let control = self.control(&handle.parent_thread_id).await?;
        control
            .cancel(&native_handle(handle)?)
            .await
            .map_err(|error| error.to_string())
    }

    async fn run_command(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        argv: &[String],
        cwd: Option<&Path>,
        timeout_ms: u64,
    ) -> Result<CommandOutcome, String> {
        let control = self.control(parent_thread_id).await?;
        let cwd = AbsolutePathBuf::try_from(cwd.unwrap_or(repository).to_path_buf())
            .map_err(|error| error.to_string())?;
        let output = control
            .run_command(OrchestraCommandRequest {
                argv: argv.to_vec(),
                cwd,
                timeout_ms,
            })
            .await
            .map_err(|error| error.to_string())?;
        Ok(CommandOutcome {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    async fn create_worktree(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        run_id: &str,
        step_id: &str,
        policy: &WorktreePolicy,
        source_revision: &str,
    ) -> Result<PathBuf, String> {
        if source_revision == "unborn" && *policy == WorktreePolicy::Shared {
            return Ok(repository.to_path_buf());
        }
        if source_revision == "unborn" {
            return Err("isolated worktrees require a committed source revision".into());
        }
        let root = repository.join(".codex/orchestra/worktrees");
        std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = if *policy == WorktreePolicy::Shared {
            root.join(format!("{run_id}-shared"))
        } else {
            root.join(format!("{run_id}-{step_id}"))
        };
        if path.exists() {
            if *policy == WorktreePolicy::Shared {
                return Ok(path);
            }
            self.remove_worktree(parent_thread_id, repository, &path)
                .await?;
        }
        let outcome = self
            .run_command(
                parent_thread_id,
                repository,
                &[
                    "git".into(),
                    "worktree".into(),
                    "add".into(),
                    "--detach".into(),
                    path.to_string_lossy().into_owned(),
                    source_revision.into(),
                ],
                Some(repository),
                120_000,
            )
            .await?;
        if outcome.exit_code != 0 {
            return Err(format!("git worktree add failed: {}", outcome.stderr));
        }
        Ok(path)
    }

    async fn remove_worktree(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        path: &Path,
    ) -> Result<(), String> {
        let outcome = self
            .run_command(
                parent_thread_id,
                repository,
                &[
                    "git".into(),
                    "worktree".into(),
                    "remove".into(),
                    "--force".into(),
                    path.to_string_lossy().into_owned(),
                ],
                Some(repository),
                120_000,
            )
            .await?;
        if outcome.exit_code == 0 {
            Ok(())
        } else {
            Err(outcome.stderr)
        }
    }

    async fn create_persistent_worktree(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        path: &Path,
        source_revision: &str,
    ) -> Result<PathBuf, String> {
        if source_revision == "unborn" {
            return Err("Automation worktrees require a committed source revision".into());
        }
        if path.exists() {
            return Err(format!(
                "Automation worktree already exists at {}",
                path.display()
            ));
        }
        let parent = path
            .parent()
            .ok_or("Automation worktree is missing its configured root")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let outcome = self
            .run_command(
                parent_thread_id,
                repository,
                &[
                    "git".into(),
                    "worktree".into(),
                    "add".into(),
                    "--detach".into(),
                    path.to_string_lossy().into_owned(),
                    source_revision.into(),
                ],
                Some(repository),
                120_000,
            )
            .await?;
        if outcome.exit_code != 0 {
            return Err(format!("git worktree add failed: {}", outcome.stderr));
        }
        path.canonicalize().map_err(|error| error.to_string())
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

fn native_handle(handle: &AgentHandle) -> Result<OrchestraAgentHandle, String> {
    Ok(OrchestraAgentHandle {
        thread_id: ThreadId::from_string(&handle.thread_id).map_err(|error| error.to_string())?,
        task_path: AgentPath::try_from(handle.task_path.as_str())
            .map_err(|error| error.to_string())?,
    })
}
fn map_status(status: CodexAgentStatus) -> AgentStatus {
    match status {
        CodexAgentStatus::PendingInit => AgentStatus::Pending,
        CodexAgentStatus::Running | CodexAgentStatus::Interrupted => AgentStatus::Running,
        CodexAgentStatus::Completed(_) => AgentStatus::Completed,
        CodexAgentStatus::Errored(error) => AgentStatus::Failed(error),
        CodexAgentStatus::Shutdown | CodexAgentStatus::NotFound => AgentStatus::Cancelled,
    }
}

#[derive(Clone)]
pub struct OrchestraService {
    host: CodexHost,
    runtime: OrchestraRuntime<CodexHost>,
    queries: ExecutionQueryService,
    manager: Weak<ThreadManager>,
    automation_shutdown: Arc<AutomationShutdownFence>,
}

#[derive(Default)]
struct AutomationShutdownFence {
    roots: Mutex<BTreeMap<(PathBuf, String), ()>>,
}

impl AutomationShutdownFence {
    fn track(&self, repository: &Path, run_id: &str) {
        if let Ok(mut roots) = self.roots.lock() {
            roots.insert((repository.to_path_buf(), run_id.to_owned()), ());
        }
    }

    fn remove(&self, repository: &Path, run_id: &str) {
        if let Ok(mut roots) = self.roots.lock() {
            roots.remove(&(repository.to_path_buf(), run_id.to_owned()));
        }
    }
}

impl Drop for AutomationShutdownFence {
    fn drop(&mut self) {
        let Ok(roots) = self.roots.get_mut() else {
            return;
        };
        for (repository, run_id) in roots.keys() {
            let result = (|| {
                let store = AutomationRunStore::open(repository, run_id)?;
                let mut root = store.load()?;
                if root.status == AutomationRootStatus::Running {
                    store.pause(&mut root, "graceful Codex host shutdown")?;
                }
                Ok::<(), codex_orchestra_core::AutomationRunError>(())
            })();
            if let Err(error) = result {
                eprintln!("failed to fence Automation `{run_id}` during host shutdown: {error}");
            }
        }
    }
}

impl OrchestraService {
    pub fn new(manager: Weak<ThreadManager>) -> Self {
        let host = CodexHost {
            manager: manager.clone(),
        };
        Self {
            runtime: OrchestraRuntime::new(host.clone()),
            host,
            queries: ExecutionQueryService::with_history_source(
                ExecutionQueryLimits::default(),
                Arc::new(CodexExecutionHistory {
                    manager: manager.clone(),
                }),
            ),
            manager,
            automation_shutdown: Arc::new(AutomationShutdownFence::default()),
        }
    }

    pub async fn validate(
        &self,
        parent_thread_id: &str,
        workflow_path: &str,
    ) -> Result<ExecutionPlan, String> {
        let repository = self.repository(parent_thread_id).await?;
        let path = safe_workflow(&repository, workflow_path)?;
        let source = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        compile_workflow(&source).map_err(|error| error.to_string())
    }

    pub async fn validate_automation(
        &self,
        parent_thread_id: &str,
        profile_path: &str,
        fixture_issue: AutomationIssue,
        attempt: Option<u32>,
    ) -> Result<AutomationValidationResult, String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        let config = thread.config_snapshot().await;
        let repository = config.cwd().as_path().to_path_buf();
        let profile_path = safe_automation_profile(&repository, profile_path)?;
        let sandbox_policy = config.sandbox_policy();
        let thread_sandbox = match sandbox_policy {
            SandboxPolicy::ReadOnly { .. } => "read-only",
            SandboxPolicy::WorkspaceWrite { .. } => "workspace-write",
            SandboxPolicy::DangerFullAccess => "danger-full-access",
            SandboxPolicy::ExternalSandbox { .. } => "read-only",
        }
        .to_owned();
        let inherited_policy = InheritedCodexPolicy {
            approval_policy: serde_json::to_value(config.approval_policy)
                .map_err(|error| error.to_string())?,
            thread_sandbox,
            turn_sandbox_policy: serde_json::to_value(sandbox_policy)
                .map_err(|error| error.to_string())?,
        };
        Ok(validate_automation_profile(AutomationValidationRequest {
            workflow_md_path: profile_path,
            repository_root: repository,
            fixture_issue,
            attempt,
            environment: std::env::vars().collect(),
            home_dir: std::env::var_os("HOME").map(PathBuf::from),
            inherited_policy,
        }))
    }

    pub async fn read_linear_automation(
        &self,
        parent_thread_id: &str,
        profile_path: &str,
        kind: AutomationLinearReadKind,
        after: Option<&str>,
        first: Option<u32>,
        issue_identifier: Option<&str>,
    ) -> Result<AutomationLinearRead, String> {
        let validation = self
            .validate_automation(
                parent_thread_id,
                profile_path,
                AutomationIssue {
                    id: "live-preview".into(),
                    identifier: issue_identifier.unwrap_or("LIVE-PREVIEW").into(),
                    title: "Live Linear intake preview".into(),
                    description: None,
                    priority: None,
                    state: "live".into(),
                    branch_name: None,
                    url: None,
                    labels: Vec::new(),
                    blocked_by: Vec::new(),
                    created_at: None,
                    updated_at: None,
                },
                None,
            )
            .await?;
        if !validation.valid {
            return Err(format!(
                "Automation profile is invalid: {}",
                validation
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.severity
                            == codex_orchestra_core::AutomationValidationSeverity::Error
                    })
                    .map(|diagnostic| format!("{}: {}", diagnostic.path, diagnostic.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        let profile = validation
            .profile
            .ok_or("valid Automation profile is missing its canonical snapshot")?;
        let credential = match profile.tracker.credential.kind {
            AutomationSecretKind::Environment => {
                std::env::var(&profile.tracker.credential.reference)
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            }
            AutomationSecretKind::InlineDigest => None,
        };
        let Some(credential) = credential else {
            return Ok(AutomationLinearRead {
                status: AutomationLinearReadStatus::Skipped,
                issues: Vec::new(),
                has_next_page: false,
                end_cursor: None,
                next_action: "configure the referenced Linear credential to opt into live reads"
                    .into(),
            });
        };
        validate_linear_endpoint(&profile.tracker.endpoint)?;
        let first = first.unwrap_or(25);
        if !(1..=50).contains(&first) {
            return Err("Linear page size must be between 1 and 50".into());
        }
        if after.is_some_and(|cursor| cursor.len() > 512) {
            return Err("Linear cursor exceeds the 512-byte limit".into());
        }

        let (query, variables) = match kind {
            AutomationLinearReadKind::Candidates | AutomationLinearReadKind::Terminal => (
                linear_project_issues_query(),
                json!({
                    "projectId": profile.tracker.project_slug,
                    "first": first,
                    "after": after,
                }),
            ),
            AutomationLinearReadKind::Refresh => {
                let identifier = issue_identifier
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or("Linear refresh requires an issue identifier")?;
                (linear_issue_query(), json!({"issueId": identifier}))
            }
        };
        let response = execute_linear_read(
            &profile.tracker.endpoint,
            &credential,
            profile.codex.read_timeout_ms,
            query,
            variables,
        )
        .await?;
        let (mut issues, has_next_page, end_cursor) = match kind {
            AutomationLinearReadKind::Refresh => (
                vec![normalize_linear_issue(&response).map_err(|error| error.to_string())?],
                false,
                None,
            ),
            AutomationLinearReadKind::Candidates | AutomationLinearReadKind::Terminal => {
                let page =
                    normalize_linear_issue_page(&response).map_err(|error| error.to_string())?;
                (page.issues, page.has_next_page, page.end_cursor)
            }
        };
        let selected_states = match kind {
            AutomationLinearReadKind::Candidates => Some(&profile.tracker.active_states),
            AutomationLinearReadKind::Terminal => Some(&profile.tracker.terminal_states),
            AutomationLinearReadKind::Refresh => None,
        };
        if let Some(states) = selected_states {
            issues.retain(|issue| {
                states
                    .iter()
                    .any(|state| state.eq_ignore_ascii_case(&issue.state))
            });
        }
        Ok(AutomationLinearRead {
            status: AutomationLinearReadStatus::Ready,
            issues,
            has_next_page,
            end_cursor,
            next_action: if has_next_page {
                "request the next bounded Linear page".into()
            } else {
                "live Linear read complete".into()
            },
        })
    }

    pub async fn run_automation_fixture(
        &self,
        parent_thread_id: &str,
        profile_path: &str,
        fixture_issue: AutomationIssue,
        attempt: Option<u32>,
    ) -> Result<AutomationRootCheckpoint, String> {
        let attempt = attempt.unwrap_or(1);
        if attempt == 0 {
            return Err("Automation attempt must be greater than zero".into());
        }
        let validation = self
            .validate_automation(
                parent_thread_id,
                profile_path,
                fixture_issue.clone(),
                Some(attempt),
            )
            .await?;
        if !validation.valid {
            return Err(format!(
                "Automation profile is invalid: {}",
                validation
                    .diagnostics
                    .iter()
                    .map(|diagnostic| format!("{}: {}", diagnostic.path, diagnostic.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        let profile = validation
            .profile
            .ok_or("valid Automation profile is missing its canonical snapshot")?;
        let profile_digest = validation
            .profile_digest
            .ok_or("valid Automation profile is missing its digest")?;
        let task_prompt = validation
            .preview
            .and_then(|preview| preview.rendered_prompt)
            .ok_or("valid Automation profile is missing its rendered task prompt")?;
        validate_fixture_eligibility(&profile, &fixture_issue)?;

        let repository = self.repository(parent_thread_id).await?;
        let source_revision =
            repository_revision(&repository).map_err(|error| error.to_string())?;
        let (store, mut root) = AutomationRunStore::start(AutomationRunStart {
            repository: &repository,
            owner_thread_id: parent_thread_id,
            source_revision: &source_revision,
            profile: &profile,
            profile_digest: &profile_digest,
        })
        .map_err(|error| error.to_string())?;
        self.automation_shutdown.track(&repository, &root.run_id);
        let coordination = store
            .coordinate_fixture(
                &mut root,
                &profile,
                std::slice::from_ref(&fixture_issue),
                attempt,
            )
            .map_err(|error| error.to_string())?;
        let claim_id = coordination
            .dispatched_claim_ids
            .into_iter()
            .next()
            .ok_or("fixture issue is not dispatchable under current Automation capacity")?;
        let requested_worktree = root.claims[&claim_id].worktree.clone();
        let worktree = match self
            .host
            .create_persistent_worktree(
                parent_thread_id,
                &repository,
                &requested_worktree,
                &source_revision,
            )
            .await
        {
            Ok(path) => path,
            Err(error) => {
                fail_automation_claim(&store, &mut root, &claim_id, &error)?;
                return Err(error);
            }
        };
        store
            .update_claim(&mut root, &claim_id, |claim| {
                claim.worktree = worktree.clone();
                claim.next_action = "initialize native Issue task".into();
            })
            .map_err(|error| error.to_string())?;

        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        let config = thread.config_snapshot().await;
        let issue_json =
            serde_json::to_string_pretty(&fixture_issue).map_err(|error| error.to_string())?;
        let bootstrap_prompt = format!(
            "You are the persistent Issue task for `{}`. The native Orchestra runtime will execute the selected typed Workflow under this task after this initialization turn. Retain the issue context below and return exactly {{\"ready\":true}}.\n\n{}\n\nIssue snapshot:\n{}",
            fixture_issue.identifier, task_prompt, issue_json
        );
        let issue_handle = match self
            .host
            .spawn(SpawnRequest {
                parent_thread_id: parent_thread_id.into(),
                task_name: format!("automation_{}", safe_task_name(&fixture_issue.identifier)),
                prompt: bootstrap_prompt,
                skill_context: String::new(),
                cwd: worktree.clone(),
                model: config.model.clone(),
                reasoning_effort: config
                    .reasoning_effort
                    .clone()
                    .map(|value| serde_json::to_value(value).unwrap_or(Value::Null))
                    .and_then(|value| value.as_str().map(str::to_owned)),
                service_tier: config.service_tier.clone(),
                fork_turns: ForkTurns::None,
                allow_delegation: true,
                minimum_descendant_depth: 1,
            })
            .await
        {
            Ok(handle) => handle,
            Err(error) => {
                fail_automation_claim(&store, &mut root, &claim_id, &error)?;
                return Err(error);
            }
        };
        store
            .update_claim(&mut root, &claim_id, |claim| {
                claim.issue_task = Some(issue_handle.clone());
                claim.next_action = "wait for Issue task initialization".into();
            })
            .map_err(|error| error.to_string())?;
        let initialized = self.host.wait(&issue_handle).await?;
        if !matches!(initialized.status, AgentStatus::Completed) {
            let error = format!(
                "Issue task initialization did not complete: {:?}",
                initialized.status
            );
            fail_automation_claim(&store, &mut root, &claim_id, &error)?;
            let _ = self.host.cancel(&issue_handle).await;
            return Err(error);
        }
        if initialized
            .final_response
            .as_deref()
            .and_then(|response| serde_json::from_str::<Value>(response).ok())
            != Some(json!({"ready": true}))
        {
            let error = "Issue task initialization returned an invalid readiness result";
            fail_automation_claim(&store, &mut root, &claim_id, error)?;
            return Err(error.into());
        }

        let workflow_source = std::fs::read_to_string(&profile.orchestra.workflow_path)
            .map_err(|error| error.to_string())?;
        let plan = compile_workflow(&workflow_source).map_err(|error| error.to_string())?;
        let normalized_inputs = json!({
            "issue": fixture_issue,
            "task_prompt": task_prompt,
            "automation": {
                "profileDigest": profile_digest,
                "claimId": claim_id,
                "attempt": attempt,
                "reason": "initial",
            },
        });
        store
            .update_claim(&mut root, &claim_id, |claim| {
                claim.status = AutomationClaimStatus::Running;
                claim.next_action = "execute selected typed Workflow in Issue task".into();
            })
            .map_err(|error| error.to_string())?;
        let observed_store = AutomationRunStore::open(&repository, &root.run_id)
            .map_err(|error| error.to_string())?;
        let observed_claim = claim_id.clone();
        let outcome = self
            .runtime
            .run_with_inputs_observed(
                &worktree,
                &issue_handle.thread_id,
                plan,
                Some(&normalized_inputs),
                move |checkpoint| {
                    let mut automation =
                        observed_store.load().map_err(|error| error.to_string())?;
                    observed_store
                        .update_claim(&mut automation, &observed_claim, |claim| {
                            claim.workflow_run_id = Some(checkpoint.run_id.clone());
                            claim.workflow_status = Some(checkpoint.status.clone());
                            claim.next_action = "observe Workflow checkpoint".into();
                        })
                        .map_err(|error| error.to_string())
                },
            )
            .await;
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                fail_automation_claim(&store, &mut root, &claim_id, &error.to_string())?;
                let _ = self.host.cancel(&issue_handle).await;
                return Err(error.to_string());
            }
        };
        let workflow = outcome_checkpoint(&outcome);
        self.persist_lifecycle(
            &issue_handle.thread_id,
            workflow,
            OrchestraLifecycleKind::Invoked,
        )
        .await?;
        let effect_status = if matches!(&outcome, RunOutcome::Completed(_))
            && profile
                .orchestra
                .effects
                .contains(&codex_orchestra_core::AutomationEffect::TrackerComment)
        {
            let body = extract_tracker_comment(workflow)?;
            Some(
                store
                    .resolve_tracker_comment(
                        &mut root,
                        &claim_id,
                        &profile,
                        &body,
                        AutomationGatePolicy::AutoAccept,
                        |request| execute_fixture_tracker_comment(&repository, request),
                    )
                    .map_err(|error| error.to_string())?
                    .status,
            )
        } else {
            None
        };
        let claim_status = match (&outcome, effect_status) {
            (_, Some(AutomationEffectStatus::WaitingGate)) => AutomationClaimStatus::Suspended,
            (_, Some(AutomationEffectStatus::Rejected | AutomationEffectStatus::Failed)) => {
                AutomationClaimStatus::Failed
            }
            (_, Some(AutomationEffectStatus::Ambiguous | AutomationEffectStatus::Executing)) => {
                AutomationClaimStatus::Suspended
            }
            (RunOutcome::Completed(_), _) => AutomationClaimStatus::Completed,
            (RunOutcome::Paused(_), _) => AutomationClaimStatus::Suspended,
            (RunOutcome::Failed(_), _) => AutomationClaimStatus::Failed,
            (RunOutcome::Cancelled(_), _) => AutomationClaimStatus::Cancelled,
        };
        store
            .update_claim(&mut root, &claim_id, |claim| {
                claim.status = claim_status;
                claim.workflow_run_id = Some(workflow.run_id.clone());
                claim.workflow_status = Some(workflow.status.clone());
                claim.next_action = match claim_status {
                    AutomationClaimStatus::Completed => "claim complete".into(),
                    AutomationClaimStatus::Suspended => "resume Workflow from checkpoint".into(),
                    AutomationClaimStatus::Cancelled => "claim cancelled".into(),
                    AutomationClaimStatus::Failed => {
                        "inspect Issue task and Workflow evidence".into()
                    }
                    _ => unreachable!(),
                };
            })
            .map_err(|error| error.to_string())?;
        root.next_action = format!(
            "claim `{claim_id}` is {}; Automation remains resident",
            automation_claim_status_name(claim_status)
        );
        store.save(&mut root).map_err(|error| error.to_string())?;
        Ok(root)
    }

    pub async fn cancel_automation(
        &self,
        parent_thread_id: &str,
        run_id: &str,
    ) -> Result<AutomationRootCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        let store =
            AutomationRunStore::open(&repository, run_id).map_err(|error| error.to_string())?;
        let mut root = store.load().map_err(|error| error.to_string())?;
        if root.owner_thread_id != parent_thread_id {
            return Err("Automation Root Run does not belong to the requested task".into());
        }
        for claim in root.claims.values() {
            if let Some(workflow_run_id) = claim.workflow_run_id.as_deref() {
                let checkpoint = self
                    .runtime
                    .cancel(&claim.worktree, workflow_run_id)
                    .await
                    .map_err(|error| error.to_string())?;
                if let Some(issue_task) = claim.issue_task.as_ref() {
                    self.persist_lifecycle(
                        &issue_task.thread_id,
                        &checkpoint,
                        OrchestraLifecycleKind::Cancelled,
                    )
                    .await?;
                }
            }
            if let Some(issue_task) = claim.issue_task.as_ref() {
                let _ = self.host.cancel(issue_task).await;
            }
        }
        store.cancel(&mut root).map_err(|error| error.to_string())?;
        self.automation_shutdown.remove(&repository, run_id);
        Ok(root)
    }

    pub async fn automation_status(
        &self,
        parent_thread_id: &str,
        run_id: &str,
    ) -> Result<AutomationRootCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        let store =
            AutomationRunStore::open(&repository, run_id).map_err(|error| error.to_string())?;
        let root = store.load().map_err(|error| error.to_string())?;
        authorize_automation_root(&root, parent_thread_id)?;
        self.automation_shutdown.track(&repository, run_id);
        Ok(root)
    }

    pub async fn pause_automation(
        &self,
        parent_thread_id: &str,
        run_id: &str,
    ) -> Result<AutomationRootCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        let store =
            AutomationRunStore::open(&repository, run_id).map_err(|error| error.to_string())?;
        let mut root = store.load().map_err(|error| error.to_string())?;
        authorize_automation_root(&root, parent_thread_id)?;
        self.automation_shutdown.track(&repository, run_id);
        if root.status == AutomationRootStatus::Running {
            store
                .pause(&mut root, "explicit native pause")
                .map_err(|error| error.to_string())?;
        }
        let claims = root.claims.values().cloned().collect::<Vec<_>>();
        for claim in claims {
            let Some(workflow_run_id) = claim.workflow_run_id.as_deref() else {
                continue;
            };
            let checkpoint = self
                .runtime
                .pause(&claim.worktree, workflow_run_id)
                .await
                .map_err(|error| error.to_string())?;
            store
                .update_claim(&mut root, &claim.claim_id, |stored| {
                    stored.workflow_status = Some(checkpoint.status.clone());
                    stored.next_action = "reconcile the paused native Child Run".into();
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(root)
    }

    pub async fn reconcile_automation(
        &self,
        parent_thread_id: &str,
        run_id: &str,
        profile_path: &str,
        resume: bool,
    ) -> Result<AutomationRootCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        let store =
            AutomationRunStore::open(&repository, run_id).map_err(|error| error.to_string())?;
        let mut root = store.load().map_err(|error| error.to_string())?;
        authorize_automation_root(&root, parent_thread_id)?;
        self.automation_shutdown.track(&repository, run_id);
        if root.status == AutomationRootStatus::Running {
            store
                .pause(&mut root, "native reconciliation refresh")
                .map_err(|error| error.to_string())?;
        }
        store
            .begin_reconciliation(&mut root)
            .map_err(|error| error.to_string())?;
        let pinned_profile = store.load_profile().map_err(|error| error.to_string())?;
        let fixture = root
            .claims
            .values()
            .next()
            .map(|claim| AutomationIssue {
                id: claim.issue_id.clone(),
                identifier: claim.issue_identifier.clone(),
                title: claim.issue_title.clone(),
                description: None,
                priority: claim.priority,
                state: claim.tracker_state.clone(),
                branch_name: None,
                url: None,
                labels: pinned_profile.tracker.required_labels.clone(),
                blocked_by: Vec::new(),
                created_at: None,
                updated_at: None,
            })
            .unwrap_or_else(|| AutomationIssue {
                id: "reconcile-profile".into(),
                identifier: "RECONCILE-PROFILE".into(),
                title: "Reconcile Automation profile".into(),
                description: None,
                priority: None,
                state: pinned_profile
                    .tracker
                    .active_states
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "active".into()),
                branch_name: None,
                url: None,
                labels: pinned_profile.tracker.required_labels.clone(),
                blocked_by: Vec::new(),
                created_at: None,
                updated_at: None,
            });
        let validation = self
            .validate_automation(parent_thread_id, profile_path, fixture, None)
            .await?;
        if !validation.valid || validation.profile_digest.as_deref() != Some(&root.profile_digest) {
            return Err(
                "Automation profile revision differs from the pinned Root Run snapshot".into(),
            );
        }

        let mut tracker_issues = Vec::new();
        for claim in root.claims.values() {
            let read = self
                .read_linear_automation(
                    parent_thread_id,
                    profile_path,
                    AutomationLinearReadKind::Refresh,
                    None,
                    Some(1),
                    Some(&claim.issue_identifier),
                )
                .await?;
            if read.status == AutomationLinearReadStatus::Ready {
                tracker_issues.extend(read.issues);
            }
        }

        let mut observations = Vec::new();
        for claim in root.claims.values() {
            let issue_task_active = match claim.issue_task.as_ref() {
                Some(handle) => self.host.status(handle).await.is_ok_and(|status| {
                    !matches!(status, AgentStatus::Cancelled | AgentStatus::Failed(_))
                }),
                None => false,
            };
            let mut workflow_status = match claim.workflow_run_id.as_deref() {
                Some(workflow_run_id) => self
                    .runtime
                    .status(&claim.worktree, workflow_run_id)
                    .await
                    .ok()
                    .map(|checkpoint| checkpoint.status),
                None => None,
            };
            let terminal = tracker_issues.iter().any(|issue| {
                issue.id == claim.issue_id
                    && pinned_profile
                        .tracker
                        .terminal_states
                        .iter()
                        .any(|state| state.eq_ignore_ascii_case(&issue.state))
            });
            let mut descendants_cancelled = false;
            if terminal {
                if let Some(workflow_run_id) = claim.workflow_run_id.as_deref() {
                    let checkpoint = self
                        .runtime
                        .cancel(&claim.worktree, workflow_run_id)
                        .await
                        .map_err(|error| error.to_string())?;
                    workflow_status = Some(checkpoint.status);
                }
                if let Some(handle) = claim.issue_task.as_ref() {
                    let _ = self.host.cancel(handle).await;
                }
                descendants_cancelled = true;
            }
            observations.push(AutomationClaimReconciliation {
                claim_id: claim.claim_id.clone(),
                issue_task_active,
                descendants_cancelled,
                workflow_status,
            });
        }
        if let Err(error) =
            store.reconcile(&mut root, &pinned_profile, &tracker_issues, &observations)
        {
            if !matches!(
                error,
                codex_orchestra_core::AutomationRunError::ReconciliationBlocked(_)
            ) {
                return Err(error.to_string());
            }
            return store.load().map_err(|error| error.to_string());
        }
        if !resume {
            return Ok(root);
        }

        let resumable = root.claims.values().cloned().collect::<Vec<_>>();
        for claim in resumable {
            if claim.workflow_status != Some(RunStatus::WaitingApproval) {
                continue;
            }
            let Some(workflow_run_id) = claim.workflow_run_id.as_deref() else {
                continue;
            };
            let outcome = self
                .runtime
                .resume(&claim.worktree, workflow_run_id)
                .await
                .map_err(|error| error.to_string())?;
            let workflow = outcome_checkpoint(&outcome);
            store
                .update_claim(&mut root, &claim.claim_id, |stored| {
                    stored.workflow_status = Some(workflow.status.clone());
                    stored.status = match &outcome {
                        RunOutcome::Completed(_) => AutomationClaimStatus::Completed,
                        RunOutcome::Paused(_) => AutomationClaimStatus::Suspended,
                        RunOutcome::Failed(_) => AutomationClaimStatus::Failed,
                        RunOutcome::Cancelled(_) => AutomationClaimStatus::Cancelled,
                    };
                    stored.next_action = "native Child Run resumed from retained checkpoint".into();
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(root)
    }

    pub async fn read_automation_queue(
        &self,
        parent_thread_id: &str,
        run_id: &str,
        category: AutomationQueueCategory,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<AutomationQueuePage, String> {
        let repository = self.repository(parent_thread_id).await?;
        let store =
            AutomationRunStore::open(&repository, run_id).map_err(|error| error.to_string())?;
        let root = store.load().map_err(|error| error.to_string())?;
        if root.owner_thread_id != parent_thread_id {
            return Err("Automation Root Run does not belong to the requested task".into());
        }
        Ok(store.queue_page(
            &root,
            category,
            offset.unwrap_or_default(),
            limit.unwrap_or(25),
        ))
    }

    pub async fn run(
        &self,
        parent_thread_id: &str,
        workflow_path: &str,
        inputs: Option<&Value>,
    ) -> Result<RunOutcome, String> {
        let repository = self.repository(parent_thread_id).await?;
        reject_existing_root_run(&repository, parent_thread_id)?;
        let plan = self.validate(parent_thread_id, workflow_path).await?;
        let outcome = self
            .runtime
            .run_with_inputs(&repository, parent_thread_id, plan, inputs)
            .await
            .map_err(|error| error.to_string())?;
        self.persist_lifecycle(
            parent_thread_id,
            outcome_checkpoint(&outcome),
            OrchestraLifecycleKind::Invoked,
        )
        .await?;
        Ok(outcome)
    }

    pub async fn resume(
        &self,
        parent_thread_id: &str,
        run_id: &str,
        approval_decision: Option<&str>,
        inputs: Option<&Value>,
    ) -> Result<RunOutcome, String> {
        let repository = self.repository(parent_thread_id).await?;
        self.status(parent_thread_id, run_id).await?;
        let outcome = self
            .runtime
            .resume_with_approval_and_inputs(&repository, run_id, approval_decision, inputs)
            .await
            .map_err(|error| error.to_string())?;
        self.persist_lifecycle(
            parent_thread_id,
            outcome_checkpoint(&outcome),
            OrchestraLifecycleKind::Resumed,
        )
        .await?;
        Ok(outcome)
    }

    pub async fn status(
        &self,
        parent_thread_id: &str,
        run_id: &str,
    ) -> Result<RunCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        let checkpoint = self
            .runtime
            .status(&repository, run_id)
            .await
            .map_err(|error| error.to_string())?;
        if checkpoint.parent_thread_id != parent_thread_id {
            return Err("run does not belong to the requested task".into());
        }
        Ok(checkpoint)
    }

    pub async fn cancel(
        &self,
        parent_thread_id: &str,
        run_id: &str,
    ) -> Result<RunCheckpoint, String> {
        let repository = self.repository(parent_thread_id).await?;
        self.status(parent_thread_id, run_id).await?;
        let checkpoint = self
            .runtime
            .cancel(&repository, run_id)
            .await
            .map_err(|error| error.to_string())?;
        self.persist_lifecycle(
            parent_thread_id,
            &checkpoint,
            OrchestraLifecycleKind::Cancelled,
        )
        .await?;
        Ok(checkpoint)
    }

    pub async fn query(
        &self,
        parent_thread_id: &str,
        run_id: &str,
        selector: ExecutionSelector,
        budget: ExecutionQueryBudget,
    ) -> Result<ExecutionQueryResult, String> {
        let repository = self.repository(parent_thread_id).await?;
        self.queries
            .query(&repository, parent_thread_id, run_id, selector, budget)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn digest(
        &self,
        parent_thread_id: &str,
        run_id: &str,
        max_bytes: usize,
    ) -> Result<RunDigest, String> {
        let repository = self.repository(parent_thread_id).await?;
        self.queries
            .digest(&repository, parent_thread_id, run_id, max_bytes)
            .map_err(|error| error.to_string())
    }

    /// Return the bounded digest for this task's active root run, if one exists.
    ///
    /// The task-local Codex projection selects the run; the shared query service
    /// still owns checkpoint authorization, canonical hashing, prioritization,
    /// and byte limits.
    pub async fn active_run_digest(
        &self,
        parent_thread_id: &str,
        max_bytes: usize,
    ) -> Result<Option<RunDigest>, String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        let Some(state_db) = thread.state_db() else {
            return Ok(None);
        };
        let Some(snapshot) = state_db
            .orchestra_task_snapshot(thread_id)
            .await
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let projection = snapshot.projection.projection;
        if projection.parent_thread_id != parent_thread_id
            || !is_nonterminal_rollout_status(projection.status)
        {
            return Ok(None);
        }
        self.digest(parent_thread_id, &projection.run_id, max_bytes)
            .await
            .map(Some)
    }

    async fn repository(&self, parent_thread_id: &str) -> Result<PathBuf, String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        Ok(thread.config_snapshot().await.cwd().as_path().to_path_buf())
    }

    async fn persist_lifecycle(
        &self,
        parent_thread_id: &str,
        checkpoint: &RunCheckpoint,
        kind: OrchestraLifecycleKind,
    ) -> Result<(), String> {
        let manager = self.manager.upgrade().ok_or("thread manager dropped")?;
        let thread_id =
            ThreadId::from_string(parent_thread_id).map_err(|error| error.to_string())?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| error.to_string())?;
        let state_db = thread.state_db();
        let mut previous = if let Some(state_db) = state_db.as_ref() {
            state_db
                .orchestra_task_snapshot(thread_id)
                .await
                .map_err(|error| error.to_string())?
                .map(|snapshot| snapshot.projection)
        } else {
            None
        };
        if previous.is_none() {
            let history = thread
                .load_history(/*include_archived*/ true)
                .await
                .map_err(|error| error.to_string())?;
            let mut recovered = history
                .items
                .iter()
                .filter_map(|item| match item {
                    RolloutItem::Orchestra(event) => Some(event),
                    _ => None,
                })
                .max_by_key(|event| event.sequence)
                .cloned();
            if let (Some(state_db), Some(event)) = (state_db.as_ref(), recovered.as_ref()) {
                state_db
                    .apply_orchestra_event(thread_id, event)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            previous = recovered.take();
        }
        let sequence = previous
            .as_ref()
            .map_or(1, |event| event.sequence.saturating_add(1));
        let revision = previous
            .as_ref()
            .filter(|event| event.run_id == checkpoint.run_id)
            .map_or(1, |event| event.revision.saturating_add(1));
        let event = OrchestraRolloutItem {
            schema_version: 1,
            event_id: format!("{}:{revision}", checkpoint.run_id),
            run_id: checkpoint.run_id.clone(),
            sequence,
            revision,
            kind,
            projection: project_checkpoint(checkpoint),
        };
        // Canonical JSONL wins the barrier before the rebuildable SQLite view.
        thread
            .append_rollout_items(&[RolloutItem::Orchestra(event.clone())])
            .await
            .map_err(|error| error.to_string())?;
        if let Some(state_db) = state_db {
            state_db
                .apply_orchestra_event(thread_id, &event)
                .await
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

fn outcome_checkpoint(outcome: &RunOutcome) -> &RunCheckpoint {
    match outcome {
        RunOutcome::Completed(checkpoint)
        | RunOutcome::Paused(checkpoint)
        | RunOutcome::Failed(checkpoint)
        | RunOutcome::Cancelled(checkpoint) => checkpoint,
    }
}

fn validate_linear_endpoint(endpoint: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(endpoint).map_err(|error| error.to_string())?;
    if url.scheme() != "https"
        || url.host_str() != Some("api.linear.app")
        || url.path() != "/graphql"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("live Linear endpoint must be exactly `https://api.linear.app/graphql`".into());
    }
    Ok(())
}

fn linear_project_issues_query() -> String {
    format!(
        "query OrchestraProjectIssues($projectId: String!, $first: Int!, $after: String) {{ project(id: $projectId) {{ issues(first: $first, after: $after, orderBy: updatedAt) {{ nodes {{ {LINEAR_ISSUE_FIELDS} }} pageInfo {{ hasNextPage endCursor }} }} }} }}"
    )
}

fn linear_issue_query() -> String {
    format!(
        "query OrchestraIssueRefresh($issueId: String!) {{ issue(id: $issueId) {{ {LINEAR_ISSUE_FIELDS} }} }}"
    )
}

async fn execute_linear_read(
    endpoint: &str,
    credential: &str,
    timeout_ms: u64,
    query: String,
    variables: Value,
) -> Result<Value, String> {
    const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
    let client = build_reqwest_client_with_custom_ca(reqwest::Client::builder())
        .map_err(|error| error.to_string())?;
    let response = HttpClient::new(client)
        .post(endpoint)
        .header("Authorization", credential)
        .timeout(Duration::from_millis(timeout_ms.clamp(100, 30_000)))
        .json(&json!({"query": query, "variables": variables}))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES)
    {
        return Err("Linear response exceeds the 1 MiB limit".into());
    }
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err("Linear response exceeds the 1 MiB limit".into());
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if !status.is_success() {
        let message = value
            .pointer("/errors/0/message")
            .and_then(Value::as_str)
            .unwrap_or("Linear read failed");
        return Err(format!(
            "Linear returned HTTP {}: {message}",
            status.as_u16()
        ));
    }
    Ok(value)
}

fn validate_fixture_eligibility(
    profile: &AutomationProfile,
    issue: &AutomationIssue,
) -> Result<(), String> {
    if !profile
        .tracker
        .active_states
        .iter()
        .any(|state| state.eq_ignore_ascii_case(&issue.state))
    {
        return Err(format!(
            "fixture issue `{}` is not in an active Automation state",
            issue.identifier
        ));
    }
    let labels = issue
        .labels
        .iter()
        .map(|label| label.trim().to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    let missing = profile
        .tracker
        .required_labels
        .iter()
        .filter(|label| !labels.contains(&label.to_ascii_lowercase()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "fixture issue `{}` is missing required labels: {}",
            issue.identifier,
            missing.join(", ")
        ));
    }
    let blocked = issue.blocked_by.iter().any(|blocker| {
        blocker.state.as_ref().is_none_or(|state| {
            !profile
                .tracker
                .terminal_states
                .iter()
                .any(|terminal| terminal.eq_ignore_ascii_case(state))
        })
    });
    if blocked {
        return Err(format!(
            "fixture issue `{}` has a nonterminal blocker",
            issue.identifier
        ));
    }
    Ok(())
}

fn authorize_automation_root(
    root: &AutomationRootCheckpoint,
    parent_thread_id: &str,
) -> Result<(), String> {
    if root.owner_thread_id == parent_thread_id {
        Ok(())
    } else {
        Err("Automation Root Run does not belong to the requested task".into())
    }
}

fn extract_tracker_comment(checkpoint: &RunCheckpoint) -> Result<String, String> {
    let values = checkpoint
        .steps
        .values()
        .filter_map(|step| step.outputs.get("tracker_comment"))
        .collect::<Vec<_>>();
    let [value] = values.as_slice() else {
        return Err(
            "completed fixture Workflow must return exactly one `tracker_comment` output".into(),
        );
    };
    let object = value
        .as_object()
        .ok_or("`tracker_comment` must be an object containing only `body`")?;
    if object.len() != 1 {
        return Err("`tracker_comment` must contain only the claim-scoped `body` field".into());
    }
    object
        .get("body")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "`tracker_comment.body` must be a string".into())
}

fn execute_fixture_tracker_comment(
    repository: &Path,
    request: &AutomationTrackerCommentRequest,
) -> AutomationEffectExecution {
    let root = repository.join(".codex/orchestra/fixture-tracker");
    if let Err(error) = std::fs::create_dir_all(&root) {
        return AutomationEffectExecution::Failed {
            message: error.to_string(),
        };
    }
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("comments.jsonl"))
    {
        Ok(file) => file,
        Err(error) => {
            return AutomationEffectExecution::Failed {
                message: error.to_string(),
            };
        }
    };
    let line = match serde_json::to_vec(&json!({
        "idempotencyKey": request.idempotency_key,
        "effectId": request.effect_id,
        "claimId": request.claim_id,
        "projectSlug": request.tracker_project_slug,
        "issueId": request.issue_id,
        "body": request.body,
    })) {
        Ok(line) => line,
        Err(error) => {
            return AutomationEffectExecution::Failed {
                message: error.to_string(),
            };
        }
    };
    if let Err(error) = file.write_all(&line).and_then(|_| file.write_all(b"\n")) {
        return AutomationEffectExecution::Ambiguous {
            message: format!("fixture tracker write was interrupted: {error}"),
        };
    }
    if let Err(error) = file.sync_all() {
        return AutomationEffectExecution::Ambiguous {
            message: format!("fixture tracker receipt durability is ambiguous: {error}"),
        };
    }
    AutomationEffectExecution::Committed {
        provider_receipt: format!("fixture-comment:{}", request.idempotency_key),
    }
}

fn fail_automation_claim(
    store: &AutomationRunStore,
    root: &mut AutomationRootCheckpoint,
    claim_id: &str,
    error: &str,
) -> Result<(), String> {
    store
        .update_claim(root, claim_id, |claim| {
            claim.status = AutomationClaimStatus::Failed;
            claim.next_action = bounded_lifecycle_text(error.into());
        })
        .map_err(|storage| storage.to_string())?;
    root.next_action = format!("inspect failed claim `{claim_id}`");
    store.save(root).map_err(|storage| storage.to_string())
}

fn safe_task_name(identifier: &str) -> String {
    identifier
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn automation_claim_status_name(status: AutomationClaimStatus) -> &'static str {
    match status {
        AutomationClaimStatus::Claimed => "claimed",
        AutomationClaimStatus::Running => "running",
        AutomationClaimStatus::Completed => "completed",
        AutomationClaimStatus::Suspended => "suspended",
        AutomationClaimStatus::Cancelled => "cancelled",
        AutomationClaimStatus::Failed => "failed",
    }
}

fn project_checkpoint(checkpoint: &RunCheckpoint) -> CodexOrchestraRunProjection {
    CodexOrchestraRunProjection {
        run_id: checkpoint.run_id.clone(),
        workflow_sha256: checkpoint.workflow_sha256.clone(),
        parent_thread_id: checkpoint.parent_thread_id.clone(),
        source_revision: checkpoint.source_revision.clone(),
        status: match checkpoint.status {
            RunStatus::Pending => CodexOrchestraRunStatus::Pending,
            RunStatus::Running => CodexOrchestraRunStatus::Running,
            RunStatus::WaitingApproval => CodexOrchestraRunStatus::WaitingApproval,
            RunStatus::Completed => CodexOrchestraRunStatus::Completed,
            RunStatus::Failed => CodexOrchestraRunStatus::Failed,
            RunStatus::Cancelled => CodexOrchestraRunStatus::Cancelled,
        },
        promotion: match checkpoint.promotion {
            codex_orchestra_core::PromotionStatus::Pending => {
                CodexOrchestraPromotionStatus::Pending
            }
            codex_orchestra_core::PromotionStatus::Applied => {
                CodexOrchestraPromotionStatus::Applied
            }
            codex_orchestra_core::PromotionStatus::NotRequired => {
                CodexOrchestraPromotionStatus::NotRequired
            }
        },
        steps: checkpoint
            .steps
            .iter()
            .map(|(id, step)| CodexOrchestraStepProjection {
                id: id.clone(),
                status: match step.status {
                    codex_orchestra_core::StepStatus::Pending => CodexOrchestraStepStatus::Pending,
                    codex_orchestra_core::StepStatus::Running => CodexOrchestraStepStatus::Running,
                    codex_orchestra_core::StepStatus::Retrying => {
                        CodexOrchestraStepStatus::Retrying
                    }
                    codex_orchestra_core::StepStatus::WaitingApproval => {
                        CodexOrchestraStepStatus::WaitingApproval
                    }
                    codex_orchestra_core::StepStatus::Completed => {
                        CodexOrchestraStepStatus::Completed
                    }
                    codex_orchestra_core::StepStatus::Failed => CodexOrchestraStepStatus::Failed,
                    codex_orchestra_core::StepStatus::Cancelled => {
                        CodexOrchestraStepStatus::Cancelled
                    }
                },
                attempts: step.attempts,
                rounds: step.rounds,
                output_keys: step.outputs.keys().cloned().collect(),
                final_response: step.final_response.clone().map(bounded_lifecycle_text),
                error: step.error.clone().map(bounded_lifecycle_text),
            })
            .collect(),
        next_action: bounded_lifecycle_text(checkpoint.next_action.clone()),
    }
}

fn bounded_lifecycle_text(mut text: String) -> String {
    const MAX_BYTES: usize = 4096;
    if text.len() <= MAX_BYTES {
        return text;
    }
    let mut end = MAX_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    text.push('…');
    text
}

pub struct OrchestraTools {
    service: OrchestraService,
}

impl OrchestraTools {
    pub fn new(service: OrchestraService) -> Self {
        Self { service }
    }
}

impl ToolContributor for OrchestraTools {
    fn tools(
        &self,
        _: &ExtensionData,
        thread_store: &ExtensionData,
    ) -> Vec<Arc<dyn ToolExecutor<ToolCall>>> {
        let parent_thread_id = thread_store.level_id().to_string();
        [
            Kind::Validate,
            Kind::Run,
            Kind::Resume,
            Kind::Status,
            Kind::Cancel,
            Kind::Query,
        ]
        .into_iter()
        .map(|kind| {
            Arc::new(OrchestraTool {
                kind,
                parent_thread_id: parent_thread_id.clone(),
                service: self.service.clone(),
            }) as Arc<dyn ToolExecutor<ToolCall>>
        })
        .collect()
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Validate,
    Run,
    Resume,
    Status,
    Cancel,
    Query,
}

struct OrchestraTool {
    kind: Kind,
    parent_thread_id: String,
    service: OrchestraService,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowArgs {
    workflow_path: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteArgs {
    workflow_path: String,
    inputs: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResumeArgs {
    run_id: String,
    approval_decision: Option<String>,
    inputs: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunArgs {
    run_id: String,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum QuerySelector {
    Run,
    Steps,
    Outputs,
    Evidence,
    History,
    Digest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryArgs {
    run_id: String,
    selector: QuerySelector,
    step_id: Option<String>,
    after: Option<String>,
    history_after: Option<HistoryCursor>,
    max_items: Option<usize>,
    max_bytes: Option<usize>,
}

impl ToolExecutor<ToolCall> for OrchestraTool {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(self.kind.name())
    }
    fn spec(&self) -> ToolSpec {
        self.kind.spec()
    }
    fn handle(&self, call: ToolCall) -> codex_extension_api::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(call))
    }
}

impl OrchestraTool {
    async fn handle_call(
        &self,
        call: ToolCall,
    ) -> Result<Box<dyn codex_extension_api::ToolOutput>, FunctionCallError> {
        let value = match self.kind {
            Kind::Validate => {
                let args: WorkflowArgs = parse(&call)?;
                let plan = self
                    .service
                    .validate(&self.parent_thread_id, &args.workflow_path)
                    .await
                    .map_err(to_model)?;
                json!({"valid": true, "plan": plan})
            }
            Kind::Run => {
                let args: ExecuteArgs = parse(&call)?;
                json!(
                    self.service
                        .run(
                            &self.parent_thread_id,
                            &args.workflow_path,
                            args.inputs.as_ref(),
                        )
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Resume => {
                let args: ResumeArgs = parse(&call)?;
                json!(
                    self.service
                        .resume(
                            &self.parent_thread_id,
                            &args.run_id,
                            args.approval_decision.as_deref(),
                            args.inputs.as_ref(),
                        )
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Status => {
                let args: RunArgs = parse(&call)?;
                json!(
                    self.service
                        .status(&self.parent_thread_id, &args.run_id)
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Cancel => {
                let args: RunArgs = parse(&call)?;
                json!(
                    self.service
                        .cancel(&self.parent_thread_id, &args.run_id)
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Query => {
                let args: QueryArgs = parse(&call)?;
                if matches!(args.selector, QuerySelector::Digest) {
                    let digest = self
                        .service
                        .digest(
                            &self.parent_thread_id,
                            &args.run_id,
                            args.max_bytes.unwrap_or(4096),
                        )
                        .await
                        .map_err(to_model)?;
                    return Ok(Box::new(JsonToolOutput::new(json!({
                        "selector": "digest",
                        "result": digest,
                    }))));
                }
                let selector = match args.selector {
                    QuerySelector::Run => ExecutionSelector::Run,
                    QuerySelector::Steps => ExecutionSelector::Steps { after: args.after },
                    QuerySelector::Outputs => ExecutionSelector::Outputs {
                        step_id: args.step_id,
                        after: args.after,
                    },
                    QuerySelector::Evidence => ExecutionSelector::Evidence {
                        step_id: args.step_id,
                        after: args.after,
                    },
                    QuerySelector::History => ExecutionSelector::History {
                        after: args.history_after,
                    },
                    QuerySelector::Digest => unreachable!(),
                };
                let defaults = ExecutionQueryBudget::default();
                json!(
                    self.service
                        .query(
                            &self.parent_thread_id,
                            &args.run_id,
                            selector,
                            ExecutionQueryBudget {
                                max_items: args.max_items.unwrap_or(defaults.max_items),
                                max_bytes: args.max_bytes.unwrap_or(defaults.max_bytes),
                            },
                        )
                        .await
                        .map_err(to_model)?
                )
            }
        };
        Ok(Box::new(JsonToolOutput::new(value)))
    }
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Validate => "orchestra_validate",
            Self::Run => "orchestra_run",
            Self::Resume => "orchestra_resume",
            Self::Status => "orchestra_status",
            Self::Cancel => "orchestra_cancel",
            Self::Query => "orchestra_query",
        }
    }
    fn spec(self) -> ToolSpec {
        if matches!(self, Self::Query) {
            let history_after = JsonSchema::object(
                BTreeMap::from([
                    (
                        "sequence".into(),
                        JsonSchema::integer(Some("Last lifecycle sequence.".into())),
                    ),
                    (
                        "item_id".into(),
                        JsonSchema::string(Some("Last lifecycle event id.".into())),
                    ),
                    (
                        "revision".into(),
                        JsonSchema::integer(Some("Last lifecycle revision.".into())),
                    ),
                ]),
                Some(vec!["sequence".into(), "item_id".into(), "revision".into()]),
                Some(false.into()),
            );
            let properties = BTreeMap::from([
                (
                    "run_id".into(),
                    JsonSchema::string(Some("Task-owned Orchestra run id.".into())),
                ),
                (
                    "selector".into(),
                    JsonSchema::string_enum(
                        ["run", "steps", "outputs", "evidence", "history", "digest"]
                            .map(|value| Value::String(value.into()))
                            .to_vec(),
                        Some("Fixed bounded projection to read.".into()),
                    ),
                ),
                (
                    "step_id".into(),
                    JsonSchema::string(Some("Optional outputs/evidence step filter.".into())),
                ),
                (
                    "after".into(),
                    JsonSchema::string(Some("Opaque selector-specific page cursor.".into())),
                ),
                ("history_after".into(), history_after),
                (
                    "max_items".into(),
                    JsonSchema::integer(Some(
                        "Requested item cap; server limits still apply.".into(),
                    )),
                ),
                (
                    "max_bytes".into(),
                    JsonSchema::integer(Some(
                        "Requested response byte cap; server limits still apply.".into(),
                    )),
                ),
            ]);
            return ToolSpec::Function(ResponsesApiTool {
                name: self.name().into(),
                description: "Read one fixed, bounded, task-authorized Orchestra projection."
                    .into(),
                strict: false,
                defer_loading: None,
                parameters: JsonSchema::object(
                    properties,
                    Some(vec!["run_id".into(), "selector".into()]),
                    Some(false.into()),
                ),
                output_schema: None,
            });
        }
        let (property, description) = match self {
            Self::Validate | Self::Run => (
                "workflow_path",
                "Repository-relative path to a restricted .workflow.ts file.",
            ),
            Self::Resume | Self::Status | Self::Cancel => {
                ("run_id", "Orchestra run id under .codex/orchestra/runs/.")
            }
            Self::Query => unreachable!(),
        };
        let mut properties = BTreeMap::from([(
            property.into(),
            JsonSchema::string(Some(description.into())),
        )]);
        if matches!(self, Self::Resume) {
            properties.insert(
                "approval_decision".into(),
                JsonSchema::string(Some(
                    "Optional decision for the pending approval step.".into(),
                )),
            );
        }
        if matches!(self, Self::Run | Self::Resume) {
            properties.insert(
                "inputs".into(),
                JsonSchema::object(BTreeMap::new(), None, Some(true.into())),
            );
        }
        ToolSpec::Function(ResponsesApiTool {
            name: self.name().into(),
            description: format!(
                "Native Orchestra {} operation using the active thread's V2 control plane.",
                self.name()
            ),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                properties,
                Some(vec![property.into()]),
                Some(false.into()),
            ),
            output_schema: None,
        })
    }
}

fn parse<T: for<'de> Deserialize<'de>>(call: &ToolCall) -> Result<T, FunctionCallError> {
    serde_json::from_str(call.function_arguments()?).map_err(to_model)
}
fn to_model(error: impl std::fmt::Display) -> FunctionCallError {
    FunctionCallError::RespondToModel(error.to_string())
}
fn safe_workflow(repository: &Path, relative: &str) -> Result<PathBuf, String> {
    if !relative.ends_with(".workflow.ts") {
        return Err("workflow path must end in .workflow.ts".into());
    }
    let root = repository
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !path.starts_with(root) {
        return Err("workflow path escapes repository".into());
    }
    Ok(path)
}

fn safe_automation_profile(repository: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err("Automation profile path must stay inside the repository".into());
    }
    if relative.file_name().and_then(|value| value.to_str()) != Some("WORKFLOW.md") {
        return Err("Automation profile path must name WORKFLOW.md".into());
    }
    let root = repository
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = root.join(relative);
    if path.exists() {
        let canonical = path.canonicalize().map_err(|error| error.to_string())?;
        if !canonical.starts_with(&root) {
            return Err("Automation profile path escapes repository".into());
        }
        return Ok(canonical);
    }
    Ok(path)
}

fn reject_existing_root_run(repository: &Path, parent_thread_id: &str) -> Result<(), String> {
    let runs = repository.join(".codex/orchestra/runs");
    let Ok(entries) = std::fs::read_dir(runs) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let Ok(bytes) = std::fs::read(entry.path().join("state.json")) else {
            continue;
        };
        let Ok(checkpoint) = serde_json::from_slice::<RunCheckpoint>(&bytes) else {
            continue;
        };
        if checkpoint.parent_thread_id == parent_thread_id
            && matches!(
                checkpoint.status,
                RunStatus::Pending | RunStatus::Running | RunStatus::WaitingApproval
            )
        {
            return Err(format!(
                "task already owns nonterminal root run `{}`",
                checkpoint.run_id
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automation_profile_path_stays_task_scoped_and_preserves_missing_diagnostics() {
        let repository = tempfile::tempdir().unwrap();
        let expected = repository
            .path()
            .canonicalize()
            .unwrap()
            .join("config/WORKFLOW.md");
        assert_eq!(
            safe_automation_profile(repository.path(), "config/WORKFLOW.md").unwrap(),
            expected
        );
        assert!(safe_automation_profile(repository.path(), "../WORKFLOW.md").is_err());
        assert!(safe_automation_profile(repository.path(), "/tmp/WORKFLOW.md").is_err());
        assert!(safe_automation_profile(repository.path(), "workflow.md").is_err());
    }

    #[test]
    fn live_linear_reads_are_pinned_to_the_official_https_endpoint() {
        assert!(validate_linear_endpoint("https://api.linear.app/graphql").is_ok());
        assert!(validate_linear_endpoint("http://api.linear.app/graphql").is_err());
        assert!(validate_linear_endpoint("https://linear.example/graphql").is_err());
        assert!(validate_linear_endpoint("https://api.linear.app/graphql?token=nope").is_err());
        assert!(linear_project_issues_query().starts_with("query "));
        assert!(!linear_project_issues_query().contains("mutation"));
        assert!(!linear_issue_query().contains("mutation"));
    }

    #[test]
    fn tracker_comment_output_is_exactly_one_claim_scoped_body() {
        fn checkpoint(output: Value) -> RunCheckpoint {
            serde_json::from_value(json!({
                "schema_version": 1,
                "run_id": "workflow-34",
                "workflow_sha256": "sha",
                "parent_thread_id": "issue-task-34",
                "repository": "/tmp/worktree",
                "source_revision": "abc123",
                "status": "completed",
                "steps": {
                    "comment": {
                        "status": "completed",
                        "attempts": 1,
                        "rounds": 1,
                        "outputs": { "tracker_comment": output }
                    }
                },
                "next_action": "complete"
            }))
            .unwrap()
        }

        assert_eq!(
            extract_tracker_comment(&checkpoint(json!({"body": "Implemented and verified."})))
                .unwrap(),
            "Implemented and verified."
        );
        assert!(extract_tracker_comment(&checkpoint(json!("not an object"))).is_err());
        assert!(
            extract_tracker_comment(&checkpoint(json!({
                "body": "attempted cross-target comment",
                "issueId": "another-issue"
            })))
            .is_err()
        );

        let mut duplicate = checkpoint(json!({"body": "first"}));
        duplicate.steps.insert(
            "second".into(),
            serde_json::from_value(json!({
                "status": "completed",
                "attempts": 1,
                "rounds": 1,
                "outputs": { "tracker_comment": {"body": "second"} }
            }))
            .unwrap(),
        );
        assert!(extract_tracker_comment(&duplicate).is_err());
    }

    #[test]
    fn fixture_tracker_comment_returns_a_durable_provider_receipt() {
        let repository = tempfile::tempdir().unwrap();
        let request = AutomationTrackerCommentRequest {
            effect_id: "effect-34".into(),
            idempotency_key: "idem-34".into(),
            claim_id: "claim-34".into(),
            tracker_project_slug: "orchestra".into(),
            issue_id: "issue-34".into(),
            body: "Implemented and verified.".into(),
        };

        assert_eq!(
            execute_fixture_tracker_comment(repository.path(), &request),
            AutomationEffectExecution::Committed {
                provider_receipt: "fixture-comment:idem-34".into()
            }
        );
        let persisted = std::fs::read_to_string(
            repository
                .path()
                .join(".codex/orchestra/fixture-tracker/comments.jsonl"),
        )
        .unwrap();
        let record: Value = serde_json::from_str(persisted.trim()).unwrap();
        assert_eq!(record["claimId"], "claim-34");
        assert_eq!(record["issueId"], "issue-34");
        assert_eq!(record["idempotencyKey"], "idem-34");
        assert_eq!(record["body"], "Implemented and verified.");
    }

    #[test]
    fn exposes_exact_native_tool_surface() {
        let names = [
            Kind::Validate,
            Kind::Run,
            Kind::Resume,
            Kind::Status,
            Kind::Cancel,
            Kind::Query,
        ]
        .map(Kind::name);
        assert_eq!(
            names,
            [
                "orchestra_validate",
                "orchestra_run",
                "orchestra_resume",
                "orchestra_status",
                "orchestra_cancel",
                "orchestra_query",
            ]
        );
    }

    #[test]
    fn run_and_resume_accept_input_objects_without_changing_other_tool_contracts() {
        for kind in [Kind::Run, Kind::Resume] {
            let ToolSpec::Function(tool) = kind.spec() else {
                panic!()
            };
            let inputs = &tool.parameters.properties.as_ref().unwrap()["inputs"];
            assert!(inputs.additional_properties.is_some());
        }
        let ToolSpec::Function(validate) = Kind::Validate.spec() else {
            panic!()
        };
        assert!(
            !validate
                .parameters
                .properties
                .as_ref()
                .unwrap()
                .contains_key("inputs")
        );
    }

    #[test]
    fn query_tool_exposes_only_fixed_bounded_selectors() {
        let ToolSpec::Function(query) = Kind::Query.spec() else {
            panic!()
        };
        let properties = query.parameters.properties.as_ref().unwrap();
        assert_eq!(
            properties["selector"].enum_values.as_ref().unwrap(),
            &[
                json!("run"),
                json!("steps"),
                json!("outputs"),
                json!("evidence"),
                json!("history"),
                json!("digest"),
            ]
        );
        assert!(properties.contains_key("max_items"));
        assert!(properties.contains_key("max_bytes"));
    }

    #[test]
    fn maps_v2_completion_error_and_cancellation_statuses() {
        assert_eq!(
            map_status(CodexAgentStatus::Completed(Some("done".into()))),
            AgentStatus::Completed
        );
        assert_eq!(
            map_status(CodexAgentStatus::Errored("boom".into())),
            AgentStatus::Failed("boom".into())
        );
        assert_eq!(
            map_status(CodexAgentStatus::Shutdown),
            AgentStatus::Cancelled
        );
    }
}
