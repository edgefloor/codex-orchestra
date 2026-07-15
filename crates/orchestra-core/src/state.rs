use crate::{AgentHandle, ExecutionPlan, ResolvedInputs, RunInputs, StepOutputs};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionStatus {
    #[default]
    Pending,
    Applied,
    NotRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Retrying,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StepCheckpoint {
    pub status: StepStatus,
    pub attempts: u32,
    pub rounds: u32,
    #[serde(default)]
    pub outputs: StepOutputs,
    #[serde(default)]
    pub final_response: Option<String>,
    #[serde(default)]
    pub agent: Option<AgentHandle>,
    #[serde(default)]
    pub context_sha256: Option<String>,
    #[serde(default)]
    pub approval_decision: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl Default for StepCheckpoint {
    fn default() -> Self {
        Self {
            status: StepStatus::Pending,
            attempts: 0,
            rounds: 0,
            outputs: BTreeMap::new(),
            final_response: None,
            agent: None,
            context_sha256: None,
            approval_decision: None,
            error: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RunCheckpoint {
    pub schema_version: u32,
    pub run_id: String,
    pub workflow_sha256: String,
    #[serde(default)]
    pub inputs: RunInputs,
    #[serde(default)]
    pub inputs_sha256: String,
    pub parent_thread_id: String,
    pub repository: PathBuf,
    pub source_revision: String,
    pub status: RunStatus,
    #[serde(default)]
    pub promotion: PromotionStatus,
    pub steps: BTreeMap<String, StepCheckpoint>,
    pub next_action: String,
}

pub(crate) struct RunStore {
    root: PathBuf,
}

impl RunStore {
    pub fn create(
        repository: &Path,
        run_id: &str,
        plan: &ExecutionPlan,
        workflow_sha256: &str,
        parent_thread_id: &str,
        source_revision: String,
        inputs: &ResolvedInputs,
    ) -> Result<(Self, RunCheckpoint), std::io::Error> {
        let root = repository.join(".codex/orchestra/runs").join(run_id);
        fs::create_dir_all(root.join("outputs"))?;
        fs::create_dir_all(root.join("evidence/checks"))?;
        fs::create_dir_all(root.join("evidence/changes"))?;
        fs::create_dir_all(root.join("approvals"))?;
        let store = Self { root };
        atomic_json(&store.root.join("workflow.json"), plan)?;
        atomic_json(&store.root.join("inputs.json"), &inputs.values)?;
        let checkpoint = RunCheckpoint {
            schema_version: 3,
            run_id: run_id.into(),
            workflow_sha256: workflow_sha256.into(),
            inputs: inputs.values.clone(),
            inputs_sha256: inputs.sha256.clone(),
            parent_thread_id: parent_thread_id.into(),
            repository: repository.to_path_buf(),
            source_revision,
            status: RunStatus::Pending,
            promotion: PromotionStatus::Pending,
            steps: plan
                .steps
                .iter()
                .map(|step| (step.id.clone(), StepCheckpoint::default()))
                .collect(),
            next_action: "start dependency-ready steps".into(),
        };
        store.save(&checkpoint)?;
        Ok((store, checkpoint))
    }

    pub fn open(
        repository: &Path,
        run_id: &str,
    ) -> Result<(Self, ExecutionPlan, RunCheckpoint), std::io::Error> {
        let root = repository.join(".codex/orchestra/runs").join(run_id);
        fs::create_dir_all(root.join("evidence/changes"))?;
        let plan = serde_json::from_slice(&fs::read(root.join("workflow.json"))?)
            .map_err(std::io::Error::other)?;
        let checkpoint = serde_json::from_slice(&fs::read(root.join("state.json"))?)
            .map_err(std::io::Error::other)?;
        Ok((Self { root }, plan, checkpoint))
    }

    pub fn save(&self, checkpoint: &RunCheckpoint) -> Result<(), std::io::Error> {
        atomic_json(&self.root.join("state.json"), checkpoint)
    }
    pub fn inputs(&self) -> Result<RunInputs, std::io::Error> {
        serde_json::from_slice(&fs::read(self.root.join("inputs.json"))?)
            .map_err(std::io::Error::other)
    }
    pub fn output(&self, step_id: &str, outputs: &StepOutputs) -> Result<(), std::io::Error> {
        atomic_json(
            &self.root.join("outputs").join(format!("{step_id}.json")),
            outputs,
        )
    }
    pub fn evidence<T: Serialize>(
        &self,
        step_id: &str,
        attempt: u32,
        evidence: &T,
    ) -> Result<(), std::io::Error> {
        atomic_json(
            &self
                .root
                .join("evidence/checks")
                .join(format!("{step_id}-{attempt}.json")),
            evidence,
        )
    }
    pub fn change_patch(
        &self,
        step_id: &str,
        attempt: u32,
        patch: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let path = self
            .root
            .join("evidence/changes")
            .join(format!("{step_id}-{attempt}.patch"));
        atomic_write(&path, patch)?;
        Ok(path)
    }
    pub fn promotion_patch(&self, patch: &[u8]) -> Result<PathBuf, std::io::Error> {
        let path = self.root.join("evidence/changes/promoted.patch");
        atomic_write(&path, patch)?;
        Ok(path)
    }
    pub fn approval(&self, step_id: &str, decision: &str) -> Result<(), std::io::Error> {
        atomic_json(
            &self.root.join("approvals").join(format!("{step_id}.json")),
            &serde_json::json!({"decision": decision}),
        )
    }
    pub fn summary(&self, text: &str) -> Result<(), std::io::Error> {
        atomic_write(&self.root.join("summary.md"), text.as_bytes())
    }
}

fn atomic_json<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
    let mut data = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
    data.push(b'\n');
    atomic_write(path, &data)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp, data)?;
    fs::rename(temp, path)
}
