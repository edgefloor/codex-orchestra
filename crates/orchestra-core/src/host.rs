use crate::{ForkTurns, StepOutputs, WorktreePolicy};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AgentHandle {
    pub thread_id: String,
    pub task_path: String,
    pub parent_thread_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnRequest {
    pub parent_thread_id: String,
    pub task_name: String,
    pub prompt: String,
    pub cwd: PathBuf,
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub service_tier: Option<String>,
    pub fork_turns: ForkTurns,
    pub allow_delegation: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentOutcome {
    pub status: AgentStatus,
    pub final_response: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait NativeHost: Send + Sync + 'static {
    async fn spawn(&self, request: SpawnRequest) -> Result<AgentHandle, String>;
    async fn status(&self, handle: &AgentHandle) -> Result<AgentStatus, String>;
    async fn wait(&self, handle: &AgentHandle) -> Result<AgentOutcome, String>;
    async fn cancel(&self, handle: &AgentHandle) -> Result<(), String>;
    async fn run_command(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        argv: &[String],
        cwd: Option<&Path>,
        timeout_ms: u64,
    ) -> Result<CommandOutcome, String>;
    async fn create_worktree(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        run_id: &str,
        step_id: &str,
        policy: &WorktreePolicy,
        source_revision: &str,
    ) -> Result<PathBuf, String>;
    async fn remove_worktree(
        &self,
        parent_thread_id: &str,
        repository: &Path,
        path: &Path,
    ) -> Result<(), String>;
    async fn request_approval(
        &self,
        parent_thread_id: &str,
        prompt: &str,
        choices: &[String],
    ) -> Result<Option<String>, String>;
    async fn emit_activity(&self, parent_thread_id: &str, message: &str);
    async fn persist_outputs(&self, _run_id: &str, _step_id: &str, _outputs: &StepOutputs) {}
}
