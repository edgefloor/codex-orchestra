//! Narrow native capability exposed only to the pinned Orchestra extension.
//!
//! This wrapper deliberately preserves the active thread's `AgentControl`.
//! It does not expose the control plane itself to extensions.

use crate::agent::control::{AgentControl, SpawnAgentForkMode, SpawnAgentOptions};
use crate::agent::next_thread_spawn_depth;
use crate::config::Config;
use crate::exec::{ExecCapturePolicy, ExecExpiration, ExecParams, process_exec_tool_call};
use crate::windows_sandbox::windows_sandbox_level_from_config;
use codex_protocol::AgentPath;
use codex_protocol::ThreadId;
use codex_protocol::error::{CodexErr, Result as CodexResult};
use codex_protocol::models::SandboxPermissions;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::{AgentStatus, SessionSource, SubAgentSource};
use codex_protocol::user_input::UserInput;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum OrchestraForkTurns {
    None,
    All,
    Last(usize),
}

#[derive(Clone, Debug)]
pub struct OrchestraSpawnRequest {
    pub task_name: String,
    pub prompt: String,
    pub cwd: AbsolutePathBuf,
    pub model: String,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub service_tier: Option<String>,
    pub fork_turns: OrchestraForkTurns,
    pub allow_delegation: bool,
}

#[derive(Clone, Debug)]
pub struct OrchestraAgentHandle {
    pub thread_id: ThreadId,
    pub task_path: AgentPath,
}

#[derive(Clone, Debug)]
pub struct OrchestraCommandRequest {
    pub argv: Vec<String>,
    pub cwd: AbsolutePathBuf,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct OrchestraCommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone)]
pub struct OrchestraControl {
    control: AgentControl,
    parent_thread_id: ThreadId,
    parent_source: SessionSource,
    config: Config,
}

impl OrchestraControl {
    pub(crate) fn new(
        control: AgentControl,
        parent_thread_id: ThreadId,
        parent_source: SessionSource,
        config: Config,
    ) -> Self {
        Self {
            control,
            parent_thread_id,
            parent_source,
            config,
        }
    }

    pub async fn spawn(&self, request: OrchestraSpawnRequest) -> CodexResult<OrchestraAgentHandle> {
        let fork_mode = match request.fork_turns {
            OrchestraForkTurns::None => None,
            OrchestraForkTurns::All => Some(SpawnAgentForkMode::FullHistory),
            OrchestraForkTurns::Last(value) if value > 0 => {
                Some(SpawnAgentForkMode::LastNTurns(value))
            }
            OrchestraForkTurns::Last(_) => {
                return Err(CodexErr::InvalidRequest(
                    "fork_turns must be positive".into(),
                ));
            }
        };
        if matches!(fork_mode, Some(SpawnAgentForkMode::FullHistory))
            && (request.model != self.config.model.as_deref().unwrap_or_default()
                || request.reasoning_effort != self.config.model_reasoning_effort)
        {
            return Err(CodexErr::InvalidRequest(
                "full-history Orchestra agents must inherit model and reasoning effort".into(),
            ));
        }
        let mut config = self.config.clone();
        if !matches!(fork_mode, Some(SpawnAgentForkMode::FullHistory)) {
            config.model = Some(request.model);
            config.model_reasoning_effort = request.reasoning_effort;
        }
        if request.service_tier.is_some() {
            config.service_tier = request.service_tier;
        }
        let child_depth = next_thread_spawn_depth(&self.parent_source);
        config.cwd = request.cwd;
        if !request.allow_delegation {
            config.agent_max_depth = child_depth;
        }
        let parent_path = self
            .parent_source
            .get_agent_path()
            .unwrap_or_else(AgentPath::root);
        let task_path = parent_path
            .join(&request.task_name)
            .map_err(CodexErr::InvalidRequest)?;
        let source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id: self.parent_thread_id,
            depth: child_depth,
            agent_path: Some(task_path.clone()),
            agent_nickname: None,
            agent_role: None,
        });
        let live = self
            .control
            .spawn_agent_with_metadata(
                config,
                vec![UserInput::Text {
                    text: request.prompt,
                    text_elements: Vec::new(),
                }],
                Some(source),
                SpawnAgentOptions {
                    fork_parent_spawn_call_id: fork_mode
                        .as_ref()
                        .map(|_| format!("orchestra:{}", request.task_name)),
                    fork_mode,
                    parent_thread_id: Some(self.parent_thread_id),
                    environments: None,
                },
            )
            .await?;
        Ok(OrchestraAgentHandle {
            thread_id: live.thread_id,
            task_path,
        })
    }

    pub async fn status(&self, handle: &OrchestraAgentHandle) -> AgentStatus {
        self.control.get_status(handle.thread_id).await
    }

    pub async fn wait(&self, handle: &OrchestraAgentHandle) -> CodexResult<AgentStatus> {
        let mut receiver = self.control.subscribe_status(handle.thread_id).await?;
        loop {
            let status = receiver.borrow().clone();
            if !matches!(
                status,
                AgentStatus::PendingInit | AgentStatus::Running | AgentStatus::Interrupted
            ) {
                return Ok(status);
            }
            receiver
                .changed()
                .await
                .map_err(|_| CodexErr::InternalAgentDied)?;
        }
    }

    pub async fn cancel(&self, handle: &OrchestraAgentHandle) -> CodexResult<()> {
        self.control
            .interrupt_agent(handle.thread_id)
            .await
            .map(|_| ())
    }

    pub async fn run_command(
        &self,
        request: OrchestraCommandRequest,
    ) -> CodexResult<OrchestraCommandOutput> {
        let windows_level = windows_sandbox_level_from_config(&self.config);
        let mut env = HashMap::new();
        if let Ok(path) = std::env::var("PATH") {
            env.insert("PATH".to_string(), path);
        }
        let params = ExecParams {
            command: request.argv,
            cwd: request.cwd,
            expiration: ExecExpiration::from(request.timeout_ms),
            capture_policy: ExecCapturePolicy::ShellTool,
            env,
            network: None,
            network_environment_id: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: windows_level,
            windows_sandbox_private_desktop: self
                .config
                .permissions
                .windows_sandbox_private_desktop,
            justification: None,
            arg0: None,
        };
        let output = process_exec_tool_call(
            params,
            self.config.permissions.permission_profile(),
            &self.config.cwd,
            &self.config.effective_workspace_roots(),
            &self.config.codex_linux_sandbox_exe,
            self.config.features.use_legacy_landlock(),
            None,
        )
        .await?;
        Ok(OrchestraCommandOutput {
            exit_code: output.exit_code,
            stdout: output.stdout.text,
            stderr: output.stderr.text,
        })
    }
}
