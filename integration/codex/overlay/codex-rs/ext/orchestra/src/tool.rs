use async_trait::async_trait;
use codex_core::ThreadManager;
use codex_core::orchestra::{
    OrchestraAgentHandle, OrchestraCommandRequest, OrchestraControl, OrchestraForkTurns,
    OrchestraSpawnRequest,
};
use codex_extension_api::{
    ExtensionData, FunctionCallError, JsonToolOutput, ToolCall, ToolContributor, ToolExecutor,
    ToolName, ToolSpec,
};
use codex_orchestra_core::{
    AgentHandle, AgentOutcome, AgentStatus, CommandOutcome, ForkTurns, NativeHost,
    OrchestraRuntime, SpawnRequest, WorktreePolicy, compile_workflow,
};
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AgentStatus as CodexAgentStatus;
use codex_protocol::{AgentPath, ThreadId};
use codex_tools::{JsonSchema, ResponsesApiTool};
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

#[derive(Clone)]
struct CodexHost {
    manager: Weak<ThreadManager>,
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
                model: request.model,
                reasoning_effort,
                service_tier: request.service_tier,
                fork_turns,
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
    ) -> Result<PathBuf, String> {
        if *policy == WorktreePolicy::Shared {
            return Ok(repository.to_path_buf());
        }
        let path = repository
            .join(".codex/orchestra/worktrees")
            .join(format!("{run_id}-{step_id}"));
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
                    "HEAD".into(),
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

pub struct OrchestraTools {
    runtime: OrchestraRuntime<CodexHost>,
    manager: Weak<ThreadManager>,
}

impl OrchestraTools {
    pub fn new(manager: Weak<ThreadManager>) -> Self {
        Self {
            runtime: OrchestraRuntime::new(CodexHost {
                manager: manager.clone(),
            }),
            manager,
        }
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
        ]
        .into_iter()
        .map(|kind| {
            Arc::new(OrchestraTool {
                kind,
                parent_thread_id: parent_thread_id.clone(),
                runtime: self.runtime.clone(),
                manager: self.manager.clone(),
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
}

struct OrchestraTool {
    kind: Kind,
    parent_thread_id: String,
    runtime: OrchestraRuntime<CodexHost>,
    manager: Weak<ThreadManager>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowArgs {
    workflow_path: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunArgs {
    run_id: String,
    approval_decision: Option<String>,
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
        let manager = self
            .manager
            .upgrade()
            .ok_or_else(|| FunctionCallError::RespondToModel("thread manager dropped".into()))?;
        let thread_id = ThreadId::from_string(&self.parent_thread_id)
            .map_err(|error| FunctionCallError::RespondToModel(error.to_string()))?;
        let thread = manager
            .get_thread(thread_id)
            .await
            .map_err(|error| FunctionCallError::RespondToModel(error.to_string()))?;
        let repository = thread.config_snapshot().await.cwd().as_path().to_path_buf();
        let value = match self.kind {
            Kind::Validate | Kind::Run => {
                let args: WorkflowArgs = parse(&call)?;
                let path = safe_workflow(&repository, &args.workflow_path)?;
                let source = std::fs::read_to_string(path).map_err(to_model)?;
                let plan = compile_workflow(&source).map_err(to_model)?;
                if matches!(self.kind, Kind::Validate) {
                    json!({"valid": true, "plan": plan})
                } else {
                    json!(
                        self.runtime
                            .run(&repository, &self.parent_thread_id, plan)
                            .await
                            .map_err(to_model)?
                    )
                }
            }
            Kind::Resume => {
                let args: RunArgs = parse(&call)?;
                json!(
                    self.runtime
                        .resume_with_approval(
                            &repository,
                            &args.run_id,
                            args.approval_decision.as_deref()
                        )
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Status => {
                let args: RunArgs = parse(&call)?;
                json!(
                    self.runtime
                        .status(&repository, &args.run_id)
                        .await
                        .map_err(to_model)?
                )
            }
            Kind::Cancel => {
                let args: RunArgs = parse(&call)?;
                json!(
                    self.runtime
                        .cancel(&repository, &args.run_id)
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
        }
    }
    fn spec(self) -> ToolSpec {
        let (property, description) = match self {
            Self::Validate | Self::Run => (
                "workflow_path",
                "Repository-relative path to a restricted .workflow.ts file.",
            ),
            _ => ("run_id", "Orchestra run id under .codex/orchestra/runs/."),
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
fn safe_workflow(repository: &Path, relative: &str) -> Result<PathBuf, FunctionCallError> {
    if !relative.ends_with(".workflow.ts") {
        return Err(to_model("workflow path must end in .workflow.ts"));
    }
    let root = repository.canonicalize().map_err(to_model)?;
    let path = root.join(relative).canonicalize().map_err(to_model)?;
    if !path.starts_with(root) {
        return Err(to_model("workflow path escapes repository"));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_exact_native_tool_surface() {
        let names = [
            Kind::Validate,
            Kind::Run,
            Kind::Resume,
            Kind::Status,
            Kind::Cancel,
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
            ]
        );
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
