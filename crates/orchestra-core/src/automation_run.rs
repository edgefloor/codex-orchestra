use crate::{AgentHandle, AutomationEffect, AutomationIssue, AutomationProfile, RunStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationRootStatus {
    Running,
    Suspended,
    Cancelled,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationReconciliationStatus {
    #[default]
    Complete,
    Required,
    InProgress,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationClaimStatus {
    Claimed,
    Running,
    Completed,
    Suspended,
    Cancelled,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationQueueStatus {
    Queued,
    Blocked,
    Terminal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationQueueCategory {
    Queued,
    Running,
    Blocked,
    WaitingGate,
    Handoff,
    Terminal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationQueueItem {
    pub issue_id: String,
    pub issue_identifier: String,
    pub issue_title: String,
    pub state: String,
    pub priority: Option<i64>,
    pub status: AutomationQueueStatus,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationQueueProjectionItem {
    pub issue_id: String,
    pub issue_identifier: String,
    pub issue_title: String,
    pub state: String,
    pub priority: Option<i64>,
    pub claim_id: Option<String>,
    pub category: AutomationQueueCategory,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationQueuePage {
    pub category: AutomationQueueCategory,
    pub total: u32,
    pub items: Vec<AutomationQueueProjectionItem>,
    pub next_offset: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationQueueCounts {
    pub queued: u32,
    pub running: u32,
    pub blocked: u32,
    pub waiting_gate: u32,
    pub handoff: u32,
    pub terminal: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCoordinationResult {
    pub dispatched_claim_ids: Vec<String>,
    pub counts: AutomationQueueCounts,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationIssueClaim {
    pub claim_id: String,
    pub issue_id: String,
    pub issue_identifier: String,
    pub issue_title: String,
    #[serde(default)]
    pub tracker_state: String,
    #[serde(default)]
    pub priority: Option<i64>,
    pub attempt: u32,
    pub status: AutomationClaimStatus,
    pub worktree: PathBuf,
    pub source_revision: String,
    pub issue_task: Option<AgentHandle>,
    pub workflow_run_id: Option<String>,
    pub workflow_status: Option<RunStatus>,
    #[serde(default)]
    pub effects: Vec<AutomationEffectReceipt>,
    pub next_action: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationGatePolicy {
    AutoAccept,
    AutoReject,
    AskHuman,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationEffectStatus {
    WaitingGate,
    Rejected,
    Executing,
    Committed,
    Failed,
    Ambiguous,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationEffectReceipt {
    pub effect_id: String,
    pub idempotency_key: String,
    pub kind: AutomationEffect,
    pub claim_id: String,
    pub tracker_project_slug: String,
    pub issue_id: String,
    pub request_sha256: String,
    pub body_preview: String,
    pub gate_policy: AutomationGatePolicy,
    pub status: AutomationEffectStatus,
    pub provider_receipt: Option<String>,
    pub failure: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationTrackerCommentRequest {
    pub effect_id: String,
    pub idempotency_key: String,
    pub claim_id: String,
    pub tracker_project_slug: String,
    pub issue_id: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutomationEffectExecution {
    Committed { provider_receipt: String },
    Failed { message: String },
    Ambiguous { message: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRootCheckpoint {
    pub schema_version: u32,
    pub run_id: String,
    pub owner_thread_id: String,
    pub repository: PathBuf,
    pub source_revision: String,
    pub profile_digest: String,
    pub tracker_project_slug: String,
    pub workspace_root: PathBuf,
    pub lease_key: String,
    #[serde(default)]
    pub lease_epoch: u64,
    #[serde(default)]
    pub revision: u64,
    pub status: AutomationRootStatus,
    #[serde(default)]
    pub reconciliation: AutomationReconciliationStatus,
    pub claims: BTreeMap<String, AutomationIssueClaim>,
    #[serde(default)]
    pub queue: BTreeMap<String, AutomationQueueItem>,
    pub next_action: String,
}

pub struct AutomationRunStart<'a> {
    pub repository: &'a Path,
    pub owner_thread_id: &'a str,
    pub source_revision: &'a str,
    pub profile: &'a AutomationProfile,
    pub profile_digest: &'a str,
}

#[derive(Debug, Error)]
pub enum AutomationRunError {
    #[error("Automation state storage failed: {0}")]
    Storage(#[from] std::io::Error),
    #[error("Automation profile digest does not match its canonical snapshot")]
    ProfileDigestMismatch,
    #[error("Automation lease `{lease_key}` is already owned by task `{owner_thread_id}`")]
    LeaseConflict {
        lease_key: String,
        owner_thread_id: String,
    },
    #[error("issue `{0}` already has a claim in this Automation Root Run")]
    DuplicateIssue(String),
    #[error("Automation workspace path is outside its configured root")]
    UnsafeWorkspace,
    #[error("Automation claim `{0}` was not found")]
    MissingClaim(String),
    #[error("Automation claim `{0}` is not active")]
    InactiveClaim(String),
    #[error(
        "Automation lease is stale (expected epoch {expected_epoch} revision {expected_revision}, found epoch {actual_epoch} revision {actual_revision})"
    )]
    StaleLease {
        expected_epoch: u64,
        expected_revision: u64,
        actual_epoch: u64,
        actual_revision: u64,
    },
    #[error("Automation Root Run must be suspended before reconciliation")]
    NotSuspended,
    #[error("Automation reconciliation has not started")]
    ReconciliationNotStarted,
    #[error("Automation reconciliation is incomplete: {0}")]
    ReconciliationBlocked(String),
    #[error("tracker.comment is not authorized by the effective Automation profile")]
    MissingEffectAuthority,
    #[error("tracker.comment body must contain 1..=4096 bytes")]
    InvalidComment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutomationLease {
    lease_key: String,
    run_id: String,
    owner_thread_id: String,
    repository: PathBuf,
    tracker_project_slug: String,
    #[serde(default)]
    lease_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationClaimReconciliation {
    pub claim_id: String,
    pub issue_task_active: bool,
    pub descendants_cancelled: bool,
    pub workflow_status: Option<RunStatus>,
}

pub struct AutomationRunStore {
    repository: PathBuf,
    root: PathBuf,
    lease_path: PathBuf,
}

impl AutomationRunStore {
    /// Start a resident Automation Root Run, or reopen the one already owned by
    /// the same task and exact profile. The lease is repository/project scoped.
    pub fn start(
        request: AutomationRunStart<'_>,
    ) -> Result<(Self, AutomationRootCheckpoint), AutomationRunError> {
        let canonical_profile = serde_json::to_value(request.profile)
            .map_err(std::io::Error::other)
            .and_then(|value| crate::canonical_sha256(&value).map_err(std::io::Error::other))?;
        if canonical_profile != request.profile_digest {
            return Err(AutomationRunError::ProfileDigestMismatch);
        }
        let repository = canonical_or_lexical(request.repository)?;
        let lease_key = sha256(
            format!(
                "{}\0{}",
                repository.display(),
                request.profile.tracker.project_slug
            )
            .as_bytes(),
        );
        let leases = repository.join(".codex/orchestra/leases");
        fs::create_dir_all(&leases)?;
        let lease_path = leases.join(format!("automation-{lease_key}.json"));

        if lease_path.exists() {
            let lease: AutomationLease = read_json(&lease_path)?;
            let store = Self::open(&repository, &lease.run_id)?;
            let checkpoint = store.load()?;
            if lease.owner_thread_id == request.owner_thread_id
                && checkpoint.profile_digest == request.profile_digest
                && checkpoint.status == AutomationRootStatus::Running
            {
                return Ok((store, checkpoint));
            }
            return Err(AutomationRunError::LeaseConflict {
                lease_key,
                owner_thread_id: lease.owner_thread_id,
            });
        }

        let run_id = automation_run_id(request.profile_digest);
        let root = repository.join(".codex/orchestra/runs").join(&run_id);
        fs::create_dir_all(&root)?;
        let lease = AutomationLease {
            lease_key: lease_key.clone(),
            run_id: run_id.clone(),
            owner_thread_id: request.owner_thread_id.into(),
            repository: repository.clone(),
            tracker_project_slug: request.profile.tracker.project_slug.clone(),
            lease_epoch: 0,
        };
        create_json(&lease_path, &lease)?;
        let store = Self {
            repository: repository.clone(),
            root,
            lease_path,
        };
        if let Err(error) =
            atomic_json(&store.root.join("automation-profile.json"), request.profile)
        {
            let _ = fs::remove_file(&store.lease_path);
            return Err(error.into());
        }
        let mut checkpoint = AutomationRootCheckpoint {
            schema_version: 1,
            run_id,
            owner_thread_id: request.owner_thread_id.into(),
            repository,
            source_revision: request.source_revision.into(),
            profile_digest: request.profile_digest.into(),
            tracker_project_slug: request.profile.tracker.project_slug.clone(),
            workspace_root: PathBuf::from(&request.profile.workspace.root),
            lease_key,
            lease_epoch: 0,
            revision: 0,
            status: AutomationRootStatus::Running,
            reconciliation: AutomationReconciliationStatus::Complete,
            claims: BTreeMap::new(),
            queue: BTreeMap::new(),
            next_action: "dispatch one eligible issue".into(),
        };
        if let Err(error) = store.save(&mut checkpoint) {
            let _ = fs::remove_file(&store.lease_path);
            return Err(error);
        }
        Ok((store, checkpoint))
    }

    pub fn open(repository: &Path, run_id: &str) -> Result<Self, AutomationRunError> {
        let repository = canonical_or_lexical(repository)?;
        let root = repository.join(".codex/orchestra/runs").join(run_id);
        let checkpoint: AutomationRootCheckpoint = read_json(&root.join("automation-state.json"))?;
        let lease_path = repository
            .join(".codex/orchestra/leases")
            .join(format!("automation-{}.json", checkpoint.lease_key));
        Ok(Self {
            repository,
            root,
            lease_path,
        })
    }

    pub fn load(&self) -> Result<AutomationRootCheckpoint, AutomationRunError> {
        Ok(read_json(&self.root.join("automation-state.json"))?)
    }

    pub fn load_profile(&self) -> Result<AutomationProfile, AutomationRunError> {
        Ok(read_json(&self.root.join("automation-profile.json"))?)
    }

    pub fn claim_fixture(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        issue: &AutomationIssue,
        attempt: u32,
    ) -> Result<String, AutomationRunError> {
        if checkpoint
            .claims
            .values()
            .any(|claim| claim.issue_id == issue.id)
        {
            return Err(AutomationRunError::DuplicateIssue(issue.identifier.clone()));
        }
        let claim_id = format!(
            "claim-{}",
            &sha256(format!("{}\0{attempt}", issue.id).as_bytes())[..16]
        );
        let workspace_root = canonical_or_lexical(&checkpoint.workspace_root)?;
        let worktree =
            workspace_root.join(format!("{}-a{attempt}", safe_segment(&issue.identifier)));
        if !worktree.starts_with(&workspace_root) || worktree == workspace_root {
            return Err(AutomationRunError::UnsafeWorkspace);
        }
        checkpoint.claims.insert(
            claim_id.clone(),
            AutomationIssueClaim {
                claim_id: claim_id.clone(),
                issue_id: issue.id.clone(),
                issue_identifier: issue.identifier.clone(),
                issue_title: issue.title.clone(),
                tracker_state: issue.state.clone(),
                priority: issue.priority,
                attempt,
                status: AutomationClaimStatus::Claimed,
                worktree,
                source_revision: checkpoint.source_revision.clone(),
                issue_task: None,
                workflow_run_id: None,
                workflow_status: None,
                effects: Vec::new(),
                next_action: "create persistent issue worktree and native Issue task".into(),
            },
        );
        checkpoint.next_action = format!("start claim `{claim_id}`");
        self.save(checkpoint)?;
        Ok(claim_id)
    }

    /// Reconcile normalized tracker pages into a deterministic queue and claim
    /// as much eligible work as the effective profile permits. A saturated
    /// state is skipped rather than ending the scan, so capacity in another
    /// state remains usable.
    pub fn coordinate_fixture(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        profile: &AutomationProfile,
        issues: &[AutomationIssue],
        attempt: u32,
    ) -> Result<AutomationCoordinationResult, AutomationRunError> {
        let mut observations = BTreeMap::<String, AutomationIssue>::new();
        for issue in issues {
            observations
                .entry(issue.id.clone())
                .and_modify(|current| {
                    if issue_observation_key(issue) > issue_observation_key(current) {
                        *current = issue.clone();
                    }
                })
                .or_insert_with(|| issue.clone());
        }

        checkpoint.queue.clear();
        for claim in checkpoint.claims.values_mut() {
            if let Some(issue) = observations.get(&claim.issue_id) {
                claim.tracker_state = issue.state.clone();
                if is_terminal_state(profile, &issue.state) && claim_is_active(claim.status) {
                    claim.next_action =
                        "reconcile externally terminal tracker state before dispatch".into();
                }
            }
        }

        let claimed_issue_ids = checkpoint
            .claims
            .values()
            .filter(|claim| claim_is_active(claim.status))
            .map(|claim| claim.issue_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let required_labels = profile
            .tracker
            .required_labels
            .iter()
            .map(|label| label.trim().to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        let mut eligible = Vec::new();
        for issue in observations.values() {
            if claimed_issue_ids.contains(&issue.id) {
                continue;
            }
            if is_terminal_state(profile, &issue.state) {
                checkpoint.queue.insert(
                    issue.id.clone(),
                    queue_item(
                        issue,
                        AutomationQueueStatus::Terminal,
                        "tracker state is terminal",
                    ),
                );
                continue;
            }
            if !is_active_state(profile, &issue.state) {
                continue;
            }
            let labels = issue
                .labels
                .iter()
                .map(|label| label.trim().to_ascii_lowercase())
                .collect::<std::collections::BTreeSet<_>>();
            if !required_labels.is_subset(&labels) {
                continue;
            }
            if has_nonterminal_blocker(profile, issue) {
                checkpoint.queue.insert(
                    issue.id.clone(),
                    queue_item(
                        issue,
                        AutomationQueueStatus::Blocked,
                        "waiting for a nonterminal blocker",
                    ),
                );
                continue;
            }
            eligible.push(issue.clone());
        }
        eligible.sort_by(|left, right| dispatch_key(left).cmp(&dispatch_key(right)));

        let mut active_total = checkpoint
            .claims
            .values()
            .filter(|claim| claim_is_active(claim.status))
            .count() as u32;
        let mut active_by_state = BTreeMap::<String, u32>::new();
        for claim in checkpoint
            .claims
            .values()
            .filter(|claim| claim_is_active(claim.status))
        {
            *active_by_state
                .entry(claim.tracker_state.to_ascii_lowercase())
                .or_default() += 1;
        }

        let mut dispatched_claim_ids = Vec::new();
        for issue in eligible {
            if active_total >= profile.agent.max_concurrent_agents {
                checkpoint.queue.insert(
                    issue.id.clone(),
                    queue_item(
                        &issue,
                        AutomationQueueStatus::Queued,
                        "waiting for global capacity",
                    ),
                );
                continue;
            }
            let state_key = issue.state.to_ascii_lowercase();
            let state_limit = state_limit(profile, &issue.state);
            let state_active = active_by_state.get(&state_key).copied().unwrap_or_default();
            if state_limit.is_some_and(|limit| state_active >= limit) {
                checkpoint.queue.insert(
                    issue.id.clone(),
                    queue_item(
                        &issue,
                        AutomationQueueStatus::Queued,
                        "waiting for state capacity",
                    ),
                );
                continue;
            }
            let claim_id = self.claim_fixture(checkpoint, &issue, attempt)?;
            dispatched_claim_ids.push(claim_id);
            active_total += 1;
            *active_by_state.entry(state_key).or_default() += 1;
        }
        checkpoint.next_action = if dispatched_claim_ids.is_empty() {
            "queue reconciled; wait for eligible capacity or tracker changes".into()
        } else {
            format!(
                "start {} deterministically selected claim(s)",
                dispatched_claim_ids.len()
            )
        };
        self.save(checkpoint)?;
        Ok(AutomationCoordinationResult {
            dispatched_claim_ids,
            counts: automation_queue_counts(checkpoint),
        })
    }

    pub fn queue_page(
        &self,
        checkpoint: &AutomationRootCheckpoint,
        category: AutomationQueueCategory,
        offset: u32,
        limit: u32,
    ) -> AutomationQueuePage {
        automation_queue_page(checkpoint, category, offset, limit)
    }

    pub fn update_claim<F>(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        claim_id: &str,
        update: F,
    ) -> Result<(), AutomationRunError>
    where
        F: FnOnce(&mut AutomationIssueClaim),
    {
        let claim = checkpoint
            .claims
            .get_mut(claim_id)
            .ok_or_else(|| AutomationRunError::MissingClaim(claim_id.into()))?;
        update(claim);
        self.save(checkpoint)
    }

    /// Fence all work before descendants are interrupted. Advancing the lease
    /// epoch makes every previously loaded checkpoint and in-flight provider
    /// callback unable to commit.
    pub fn pause(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        reason: &str,
    ) -> Result<(), AutomationRunError> {
        let expected_epoch = checkpoint.lease_epoch;
        let expected_revision = checkpoint.revision;
        self.ensure_fresh(expected_epoch, expected_revision)?;
        checkpoint.lease_epoch = checkpoint.lease_epoch.saturating_add(1);
        checkpoint.status = AutomationRootStatus::Suspended;
        checkpoint.reconciliation = AutomationReconciliationStatus::Required;
        checkpoint.next_action =
            format!("Automation fenced for {reason}; reconcile retained work before dispatch");
        for claim in checkpoint.claims.values_mut() {
            if matches!(
                claim.status,
                AutomationClaimStatus::Claimed | AutomationClaimStatus::Running
            ) {
                claim.status = AutomationClaimStatus::Suspended;
                claim.next_action =
                    "inspect retained Issue task and Workflow during reconciliation".into();
            }
            for effect in &mut claim.effects {
                if effect.status == AutomationEffectStatus::Executing {
                    effect.status = AutomationEffectStatus::Ambiguous;
                    effect.failure = Some(
                        "Automation was fenced before the provider result became durable".into(),
                    );
                }
            }
        }
        self.persist(checkpoint, expected_epoch, expected_revision)?;
        self.write_lease(checkpoint)?;
        Ok(())
    }

    pub fn begin_reconciliation(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
    ) -> Result<(), AutomationRunError> {
        if checkpoint.status != AutomationRootStatus::Suspended {
            return Err(AutomationRunError::NotSuspended);
        }
        checkpoint.reconciliation = AutomationReconciliationStatus::InProgress;
        checkpoint.next_action =
            "reconcile profile, lease, tracker, worktrees, tasks, Child Runs, and effects".into();
        self.save(checkpoint)
    }

    /// Complete the fenced reconciliation pass. Existing native identities and
    /// receipts are only observed and retained here; this method never creates
    /// a replacement worktree, task, Child Run, or provider mutation.
    pub fn reconcile(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        profile: &AutomationProfile,
        tracker_issues: &[AutomationIssue],
        observations: &[AutomationClaimReconciliation],
    ) -> Result<(), AutomationRunError> {
        if checkpoint.status != AutomationRootStatus::Suspended {
            return Err(AutomationRunError::NotSuspended);
        }
        if checkpoint.reconciliation != AutomationReconciliationStatus::InProgress {
            return Err(AutomationRunError::ReconciliationNotStarted);
        }
        let canonical_profile = serde_json::to_value(profile)
            .map_err(std::io::Error::other)
            .and_then(|value| crate::canonical_sha256(&value).map_err(std::io::Error::other))?;
        if canonical_profile != checkpoint.profile_digest {
            return Err(AutomationRunError::ProfileDigestMismatch);
        }
        self.verify_lease(checkpoint)?;

        let issues = tracker_issues
            .iter()
            .map(|issue| (issue.id.as_str(), issue))
            .collect::<BTreeMap<_, _>>();
        let observed = observations
            .iter()
            .map(|observation| (observation.claim_id.as_str(), observation))
            .collect::<BTreeMap<_, _>>();
        let mut blockers = Vec::new();
        for claim in checkpoint.claims.values_mut() {
            if !claim_is_active(claim.status) {
                continue;
            }
            let Some(issue) = issues.get(claim.issue_id.as_str()) else {
                blockers.push(format!("{} tracker state", claim.issue_identifier));
                claim.next_action = "refresh tracker state before dispatch".into();
                continue;
            };
            claim.tracker_state = issue.state.clone();
            claim.priority = issue.priority;
            let Some(observation) = observed.get(claim.claim_id.as_str()) else {
                blockers.push(format!("{} native descendants", claim.issue_identifier));
                claim.next_action = "inspect retained Issue task and Child Run".into();
                continue;
            };
            if is_terminal_state(profile, &issue.state) {
                if !observation.descendants_cancelled {
                    blockers.push(format!(
                        "{} descendants still active",
                        claim.issue_identifier
                    ));
                    claim.next_action = "cancel descendants before terminal reconciliation".into();
                    continue;
                }
                if claim.effects.iter().any(|effect| {
                    matches!(
                        effect.status,
                        AutomationEffectStatus::Executing | AutomationEffectStatus::Ambiguous
                    )
                }) {
                    blockers.push(format!("{} ambiguous effects", claim.issue_identifier));
                    claim.next_action =
                        "resolve ambiguous Tracker effects before cleanup eligibility".into();
                    continue;
                }
                claim.status = AutomationClaimStatus::Cancelled;
                claim.workflow_status = observation.workflow_status.clone();
                claim.next_action =
                    "externally terminal; retained resources are cleanup eligible".into();
                continue;
            }
            if claim.issue_task.is_some() && !observation.issue_task_active {
                blockers.push(format!("{} missing Issue task", claim.issue_identifier));
                claim.next_action = "inspect missing Issue task; do not create a duplicate".into();
                continue;
            }
            if claim.worktree.exists() == false && claim.issue_task.is_some() {
                blockers.push(format!("{} missing worktree", claim.issue_identifier));
                claim.next_action = "inspect missing worktree; do not create a duplicate".into();
                continue;
            }
            claim.workflow_status = observation.workflow_status.clone();
            claim.status = AutomationClaimStatus::Suspended;
            claim.next_action = if claim.workflow_run_id.is_some() {
                "resume the existing Child Run from its native checkpoint".into()
            } else if claim.issue_task.is_some() {
                "continue the existing Issue task without respawning it".into()
            } else {
                "continue the retained claim without duplicating resources".into()
            };
        }
        if blockers.is_empty() {
            checkpoint.status = AutomationRootStatus::Running;
            checkpoint.reconciliation = AutomationReconciliationStatus::Complete;
            checkpoint.next_action =
                "reconciliation complete; eligible dispatch may continue".into();
            self.save(checkpoint)?;
            return Ok(());
        }
        checkpoint.reconciliation = AutomationReconciliationStatus::Blocked;
        checkpoint.next_action = format!("reconciliation blocked: {}", blockers.join(", "));
        self.save(checkpoint)?;
        Err(AutomationRunError::ReconciliationBlocked(
            blockers.join(", "),
        ))
    }

    pub fn cancel(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
    ) -> Result<(), AutomationRunError> {
        checkpoint.status = AutomationRootStatus::Cancelled;
        checkpoint.next_action =
            "Automation cancelled; retain checkpoints and worktrees for inspection".into();
        for claim in checkpoint.claims.values_mut() {
            if matches!(
                claim.status,
                AutomationClaimStatus::Claimed | AutomationClaimStatus::Running
            ) {
                claim.status = AutomationClaimStatus::Cancelled;
                claim.next_action = "inspect or explicitly remove retained worktree".into();
            }
        }
        self.save(checkpoint)?;
        if self.lease_path.exists() {
            fs::remove_file(&self.lease_path)?;
        }
        Ok(())
    }

    pub fn resolve_tracker_comment<F>(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        claim_id: &str,
        profile: &AutomationProfile,
        body: &str,
        gate_policy: AutomationGatePolicy,
        execute: F,
    ) -> Result<AutomationEffectReceipt, AutomationRunError>
    where
        F: FnOnce(&AutomationTrackerCommentRequest) -> AutomationEffectExecution,
    {
        if !profile
            .orchestra
            .effects
            .contains(&AutomationEffect::TrackerComment)
        {
            return Err(AutomationRunError::MissingEffectAuthority);
        }
        let body = body.trim();
        if body.is_empty() || body.len() > 4096 {
            return Err(AutomationRunError::InvalidComment);
        }
        let claim = checkpoint
            .claims
            .get_mut(claim_id)
            .ok_or_else(|| AutomationRunError::MissingClaim(claim_id.into()))?;
        let request_sha256 = sha256(body.as_bytes());
        let idempotency_key = sha256(
            format!(
                "{}\0{}\0tracker.comment\0{}",
                checkpoint.profile_digest, claim_id, request_sha256
            )
            .as_bytes(),
        );
        if let Some(index) = claim
            .effects
            .iter()
            .position(|receipt| receipt.idempotency_key == idempotency_key)
        {
            if claim.effects[index].status == AutomationEffectStatus::Executing {
                claim.effects[index].status = AutomationEffectStatus::Ambiguous;
                claim.effects[index].failure =
                    Some("execution was interrupted before a durable provider receipt".into());
                let receipt = claim.effects[index].clone();
                self.save(checkpoint)?;
                return Ok(receipt);
            }
            return Ok(claim.effects[index].clone());
        }
        if !matches!(
            claim.status,
            AutomationClaimStatus::Claimed | AutomationClaimStatus::Running
        ) {
            return Err(AutomationRunError::InactiveClaim(claim_id.into()));
        }
        let effect_id = format!("effect-{}", &idempotency_key[..16]);
        let mut receipt = AutomationEffectReceipt {
            effect_id: effect_id.clone(),
            idempotency_key: idempotency_key.clone(),
            kind: AutomationEffect::TrackerComment,
            claim_id: claim_id.into(),
            tracker_project_slug: checkpoint.tracker_project_slug.clone(),
            issue_id: claim.issue_id.clone(),
            request_sha256,
            body_preview: bounded_preview(body, 240),
            gate_policy,
            status: match gate_policy {
                AutomationGatePolicy::AutoAccept => AutomationEffectStatus::Executing,
                AutomationGatePolicy::AutoReject => AutomationEffectStatus::Rejected,
                AutomationGatePolicy::AskHuman => AutomationEffectStatus::WaitingGate,
            },
            provider_receipt: None,
            failure: None,
        };
        claim.effects.push(receipt.clone());
        self.save(checkpoint)?;
        if gate_policy != AutomationGatePolicy::AutoAccept {
            return Ok(receipt);
        }

        let request = AutomationTrackerCommentRequest {
            effect_id,
            idempotency_key,
            claim_id: claim_id.into(),
            tracker_project_slug: checkpoint.tracker_project_slug.clone(),
            issue_id: checkpoint.claims[claim_id].issue_id.clone(),
            body: body.into(),
        };
        match execute(&request) {
            AutomationEffectExecution::Committed { provider_receipt } => {
                receipt.status = AutomationEffectStatus::Committed;
                receipt.provider_receipt = Some(provider_receipt);
            }
            AutomationEffectExecution::Failed { message } => {
                receipt.status = AutomationEffectStatus::Failed;
                receipt.failure = Some(message);
            }
            AutomationEffectExecution::Ambiguous { message } => {
                receipt.status = AutomationEffectStatus::Ambiguous;
                receipt.failure = Some(message);
            }
        }
        let claim = checkpoint.claims.get_mut(claim_id).expect("claim exists");
        let stored = claim
            .effects
            .iter_mut()
            .find(|stored| stored.idempotency_key == receipt.idempotency_key)
            .expect("prepared receipt exists");
        *stored = receipt.clone();
        self.save(checkpoint)?;
        Ok(receipt)
    }

    pub fn save(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
    ) -> Result<(), AutomationRunError> {
        let expected_epoch = checkpoint.lease_epoch;
        let expected_revision = checkpoint.revision;
        self.persist(checkpoint, expected_epoch, expected_revision)
    }

    pub fn repository(&self) -> &Path {
        &self.repository
    }

    fn ensure_fresh(
        &self,
        expected_epoch: u64,
        expected_revision: u64,
    ) -> Result<(), AutomationRunError> {
        let path = self.root.join("automation-state.json");
        if !path.exists() {
            return Ok(());
        }
        let current: AutomationRootCheckpoint = read_json(&path)?;
        if current.lease_epoch != expected_epoch || current.revision != expected_revision {
            return Err(AutomationRunError::StaleLease {
                expected_epoch,
                expected_revision,
                actual_epoch: current.lease_epoch,
                actual_revision: current.revision,
            });
        }
        Ok(())
    }

    fn persist(
        &self,
        checkpoint: &mut AutomationRootCheckpoint,
        expected_epoch: u64,
        expected_revision: u64,
    ) -> Result<(), AutomationRunError> {
        self.ensure_fresh(expected_epoch, expected_revision)?;
        let previous_revision = checkpoint.revision;
        checkpoint.revision = expected_revision.saturating_add(1);
        if let Err(error) = atomic_json(&self.root.join("automation-state.json"), checkpoint) {
            checkpoint.revision = previous_revision;
            return Err(error.into());
        }
        Ok(())
    }

    fn write_lease(&self, checkpoint: &AutomationRootCheckpoint) -> Result<(), AutomationRunError> {
        atomic_json(
            &self.lease_path,
            &AutomationLease {
                lease_key: checkpoint.lease_key.clone(),
                run_id: checkpoint.run_id.clone(),
                owner_thread_id: checkpoint.owner_thread_id.clone(),
                repository: checkpoint.repository.clone(),
                tracker_project_slug: checkpoint.tracker_project_slug.clone(),
                lease_epoch: checkpoint.lease_epoch,
            },
        )?;
        Ok(())
    }

    fn verify_lease(
        &self,
        checkpoint: &AutomationRootCheckpoint,
    ) -> Result<(), AutomationRunError> {
        let lease: AutomationLease = read_json(&self.lease_path)?;
        if lease.run_id != checkpoint.run_id
            || lease.owner_thread_id != checkpoint.owner_thread_id
            || lease.lease_epoch != checkpoint.lease_epoch
        {
            return Err(AutomationRunError::LeaseConflict {
                lease_key: checkpoint.lease_key.clone(),
                owner_thread_id: lease.owner_thread_id,
            });
        }
        Ok(())
    }
}

fn automation_run_id(profile_digest: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("automation-{millis}-{}", &profile_digest[..12])
}

fn safe_segment(value: &str) -> String {
    let segment = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    segment.trim_matches('-').to_owned()
}

fn claim_is_active(status: AutomationClaimStatus) -> bool {
    matches!(
        status,
        AutomationClaimStatus::Claimed
            | AutomationClaimStatus::Running
            | AutomationClaimStatus::Suspended
    )
}

fn is_active_state(profile: &AutomationProfile, state: &str) -> bool {
    profile
        .tracker
        .active_states
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(state))
}

fn is_terminal_state(profile: &AutomationProfile, state: &str) -> bool {
    profile
        .tracker
        .terminal_states
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(state))
}

fn has_nonterminal_blocker(profile: &AutomationProfile, issue: &AutomationIssue) -> bool {
    issue.blocked_by.iter().any(|blocker| {
        blocker
            .state
            .as_deref()
            .is_none_or(|state| !is_terminal_state(profile, state))
    })
}

fn state_limit(profile: &AutomationProfile, state: &str) -> Option<u32> {
    profile
        .agent
        .max_concurrent_agents_by_state
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(state))
        .map(|(_, limit)| *limit)
}

fn dispatch_key(issue: &AutomationIssue) -> (i64, &str, &str, &str) {
    (
        issue
            .priority
            .filter(|priority| *priority > 0)
            .unwrap_or(i64::MAX),
        issue.created_at.as_deref().unwrap_or("~"),
        issue.identifier.as_str(),
        issue.id.as_str(),
    )
}

fn issue_observation_key(issue: &AutomationIssue) -> (&str, &str, &str) {
    (
        issue.updated_at.as_deref().unwrap_or(""),
        issue.identifier.as_str(),
        issue.title.as_str(),
    )
}

fn queue_item(
    issue: &AutomationIssue,
    status: AutomationQueueStatus,
    reason: &str,
) -> AutomationQueueItem {
    AutomationQueueItem {
        issue_id: issue.id.clone(),
        issue_identifier: issue.identifier.clone(),
        issue_title: issue.title.clone(),
        state: issue.state.clone(),
        priority: issue.priority,
        status,
        reason: reason.into(),
    }
}

pub fn automation_queue_counts(checkpoint: &AutomationRootCheckpoint) -> AutomationQueueCounts {
    let mut counts = AutomationQueueCounts::default();
    for item in checkpoint.queue.values() {
        match item.status {
            AutomationQueueStatus::Queued => counts.queued += 1,
            AutomationQueueStatus::Blocked => counts.blocked += 1,
            AutomationQueueStatus::Terminal => counts.terminal += 1,
        }
    }
    for claim in checkpoint.claims.values() {
        let waiting_gate = claim
            .effects
            .iter()
            .any(|effect| effect.status == AutomationEffectStatus::WaitingGate);
        match claim.status {
            AutomationClaimStatus::Claimed | AutomationClaimStatus::Running if waiting_gate => {
                counts.waiting_gate += 1;
            }
            AutomationClaimStatus::Claimed | AutomationClaimStatus::Running => {
                counts.running += 1;
            }
            AutomationClaimStatus::Suspended if waiting_gate => counts.waiting_gate += 1,
            AutomationClaimStatus::Suspended => counts.handoff += 1,
            AutomationClaimStatus::Completed
            | AutomationClaimStatus::Cancelled
            | AutomationClaimStatus::Failed => counts.terminal += 1,
        }
    }
    counts
}

pub fn automation_queue_page(
    checkpoint: &AutomationRootCheckpoint,
    category: AutomationQueueCategory,
    offset: u32,
    limit: u32,
) -> AutomationQueuePage {
    let mut items = checkpoint
        .queue
        .values()
        .filter_map(|item| {
            let item_category = match item.status {
                AutomationQueueStatus::Queued => AutomationQueueCategory::Queued,
                AutomationQueueStatus::Blocked => AutomationQueueCategory::Blocked,
                AutomationQueueStatus::Terminal => AutomationQueueCategory::Terminal,
            };
            (item_category == category).then(|| AutomationQueueProjectionItem {
                issue_id: item.issue_id.clone(),
                issue_identifier: item.issue_identifier.clone(),
                issue_title: item.issue_title.clone(),
                state: item.state.clone(),
                priority: item.priority,
                claim_id: None,
                category: item_category,
                next_action: item.reason.clone(),
            })
        })
        .chain(checkpoint.claims.values().filter_map(|claim| {
            let waiting_gate = claim
                .effects
                .iter()
                .any(|effect| effect.status == AutomationEffectStatus::WaitingGate);
            let claim_category = match claim.status {
                AutomationClaimStatus::Claimed | AutomationClaimStatus::Running if waiting_gate => {
                    AutomationQueueCategory::WaitingGate
                }
                AutomationClaimStatus::Claimed | AutomationClaimStatus::Running => {
                    AutomationQueueCategory::Running
                }
                AutomationClaimStatus::Suspended if waiting_gate => {
                    AutomationQueueCategory::WaitingGate
                }
                AutomationClaimStatus::Suspended => AutomationQueueCategory::Handoff,
                AutomationClaimStatus::Completed
                | AutomationClaimStatus::Cancelled
                | AutomationClaimStatus::Failed => AutomationQueueCategory::Terminal,
            };
            (claim_category == category).then(|| AutomationQueueProjectionItem {
                issue_id: claim.issue_id.clone(),
                issue_identifier: claim.issue_identifier.clone(),
                issue_title: claim.issue_title.clone(),
                state: claim.tracker_state.clone(),
                priority: claim.priority,
                claim_id: Some(claim.claim_id.clone()),
                category: claim_category,
                next_action: claim.next_action.clone(),
            })
        }))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        (
            left.priority.unwrap_or(i64::MAX),
            left.issue_identifier.as_str(),
            left.issue_id.as_str(),
        )
            .cmp(&(
                right.priority.unwrap_or(i64::MAX),
                right.issue_identifier.as_str(),
                right.issue_id.as_str(),
            ))
    });
    let total = items.len() as u32;
    let start = usize::min(offset as usize, items.len());
    let bounded_limit = limit.clamp(1, 50) as usize;
    let end = usize::min(start + bounded_limit, items.len());
    let page = items[start..end].to_vec();
    AutomationQueuePage {
        category,
        total,
        items: page,
        next_offset: (end < items.len()).then_some(end as u32),
    }
}

fn bounded_preview(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.into();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &value[..end])
}

fn canonical_or_lexical(path: &Path) -> Result<PathBuf, std::io::Error> {
    if path.exists() {
        return path.canonicalize();
    }
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    let mut out = base;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(value) => out.push(value),
        }
    }
    Ok(out)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, std::io::Error> {
    serde_json::from_slice(&fs::read(path)?).map_err(std::io::Error::other)
}

fn create_json<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut data = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
    data.push(b'\n');
    file.write_all(&data)?;
    file.sync_all()
}

fn atomic_json<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
    let mut data = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
    data.push(b'\n');
    let nonce = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = path.with_extension(format!("tmp-{}-{nanos}-{nonce}", std::process::id()));
    fs::write(&temp, data)?;
    fs::rename(temp, path)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutomationAgentProfile, AutomationCodexPolicy, AutomationEffect, AutomationHooksProfile,
        AutomationOrchestraProfile, AutomationPollingProfile, AutomationSecretKind,
        AutomationSecretReference, AutomationTrackerProfile, AutomationWorkspaceProfile,
    };
    use serde_json::{Value, json};
    use tempfile::tempdir;

    fn profile(workspace: &Path) -> AutomationProfile {
        AutomationProfile {
            tracker: AutomationTrackerProfile {
                kind: "linear".into(),
                endpoint: "https://api.linear.app/graphql".into(),
                project_slug: "orchestra".into(),
                required_labels: vec!["automation".into()],
                active_states: vec!["Todo".into()],
                terminal_states: vec!["Done".into()],
                credential: AutomationSecretReference {
                    kind: AutomationSecretKind::Environment,
                    reference: "LINEAR_API_KEY".into(),
                    digest: "digest".into(),
                },
            },
            polling: AutomationPollingProfile {
                interval_ms: 30_000,
            },
            workspace: AutomationWorkspaceProfile {
                root: workspace.to_string_lossy().into_owned(),
            },
            hooks: AutomationHooksProfile {
                after_create: None,
                before_run: None,
                after_run: None,
                before_remove: None,
                timeout_ms: 60_000,
            },
            agent: AutomationAgentProfile {
                max_concurrent_agents: 1,
                max_turns: 20,
                max_retry_backoff_ms: 300_000,
                max_concurrent_agents_by_state: BTreeMap::new(),
            },
            codex: AutomationCodexPolicy {
                approval_policy: json!("on-request"),
                thread_sandbox: "workspace-write".into(),
                turn_sandbox_policy: Value::Null,
                turn_timeout_ms: 3_600_000,
                read_timeout_ms: 5_000,
                stall_timeout_ms: 300_000,
            },
            orchestra: AutomationOrchestraProfile {
                workflow_path: "issue.workflow.ts".into(),
                workflow_sha256: "workflow".into(),
                workflow_name: "issue".into(),
                effects: vec![AutomationEffect::TrackerComment],
            },
            prompt_template: "Implement {{ issue.identifier }}".into(),
        }
    }

    fn issue() -> AutomationIssue {
        AutomationIssue {
            id: "issue-33".into(),
            identifier: "ORC-33".into(),
            title: "Run fixture".into(),
            description: None,
            priority: None,
            state: "Todo".into(),
            branch_name: None,
            url: None,
            labels: vec!["automation".into()],
            blocked_by: Vec::new(),
            created_at: None,
            updated_at: None,
        }
    }

    fn queued_issue(
        id: &str,
        identifier: &str,
        state: &str,
        priority: Option<i64>,
    ) -> AutomationIssue {
        AutomationIssue {
            id: id.into(),
            identifier: identifier.into(),
            title: format!("Coordinate {identifier}"),
            description: None,
            priority,
            state: state.into(),
            branch_name: None,
            url: None,
            labels: vec!["automation".into()],
            blocked_by: Vec::new(),
            created_at: Some(format!("2026-07-{:02}T00:00:00Z", priority.unwrap_or(9))),
            updated_at: Some("2026-07-16T00:00:00Z".into()),
        }
    }

    #[test]
    fn root_lease_and_claim_are_stable_and_task_scoped() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let request = AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-1",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        };
        let (store, mut checkpoint) = AutomationRunStore::start(request).unwrap();
        let claim_id = store.claim_fixture(&mut checkpoint, &issue(), 1).unwrap();
        assert_eq!(checkpoint.owner_thread_id, "task-1");
        assert_eq!(checkpoint.claims[&claim_id].source_revision, "abc123");
        assert!(
            checkpoint.claims[&claim_id]
                .worktree
                .starts_with(workspace.path().canonicalize().unwrap())
        );

        let (reopened, reopened_checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-1",
            source_revision: "different-is-ignored-for-resident-root",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        assert_eq!(reopened_checkpoint.run_id, checkpoint.run_id);
        assert!(matches!(
            reopened.claim_fixture(&mut reopened_checkpoint.clone(), &issue(), 1),
            Err(AutomationRunError::DuplicateIssue(_))
        ));
    }

    #[test]
    fn another_task_cannot_take_the_repository_project_lease() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-1",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let result = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-2",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        });
        assert!(matches!(
            result,
            Err(AutomationRunError::LeaseConflict { .. })
        ));
    }

    #[test]
    fn deterministic_queue_enforces_global_and_per_state_capacity_without_head_of_line_blocking() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let mut profile = profile(workspace.path());
        profile.tracker.active_states = vec!["Todo".into(), "In Progress".into()];
        profile.agent.max_concurrent_agents = 2;
        profile
            .agent
            .max_concurrent_agents_by_state
            .insert("Todo".into(), 1);
        profile
            .agent
            .max_concurrent_agents_by_state
            .insert("In Progress".into(), 1);
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-queue",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();

        let urgent = queued_issue("issue-1", "ORC-1", "Todo", Some(1));
        let later_same_state = queued_issue("issue-2", "ORC-2", "Todo", Some(2));
        let other_state = queued_issue("issue-3", "ORC-3", "In Progress", Some(3));
        let mut blocked = queued_issue("issue-4", "ORC-4", "Todo", Some(0));
        blocked.blocked_by.push(crate::AutomationIssueBlocker {
            id: Some("blocker-1".into()),
            identifier: Some("ORC-99".into()),
            state: Some("Todo".into()),
        });
        let terminal = queued_issue("issue-5", "ORC-5", "Done", Some(1));
        let mut wrong_label = queued_issue("issue-6", "ORC-6", "Todo", Some(1));
        wrong_label.labels = vec!["not-automation".into()];

        let result = store
            .coordinate_fixture(
                &mut checkpoint,
                &profile,
                &[
                    later_same_state.clone(),
                    terminal,
                    other_state,
                    urgent,
                    blocked,
                    wrong_label,
                ],
                1,
            )
            .unwrap();
        let claimed = result
            .dispatched_claim_ids
            .iter()
            .map(|claim_id| checkpoint.claims[claim_id].issue_identifier.as_str())
            .collect::<Vec<_>>();
        assert_eq!(claimed, ["ORC-1", "ORC-3"]);
        assert_eq!(
            result.counts,
            AutomationQueueCounts {
                queued: 1,
                running: 2,
                blocked: 1,
                waiting_gate: 0,
                handoff: 0,
                terminal: 1,
            }
        );
        assert!(!checkpoint.queue.contains_key("issue-6"));
        assert_eq!(
            store
                .queue_page(&checkpoint, AutomationQueueCategory::Queued, 0, 1)
                .items[0]
                .issue_identifier,
            "ORC-2"
        );

        let mut externally_done = queued_issue("issue-1", "ORC-1", "Done", Some(1));
        externally_done.updated_at = Some("2026-07-17T00:00:00Z".into());
        let reconciled = store
            .coordinate_fixture(
                &mut checkpoint,
                &profile,
                &[externally_done, later_same_state],
                1,
            )
            .unwrap();
        assert!(reconciled.dispatched_claim_ids.is_empty());
        let urgent_claim = checkpoint
            .claims
            .values()
            .find(|claim| claim.issue_id == "issue-1")
            .unwrap();
        assert_eq!(urgent_claim.tracker_state, "Done");
        assert_eq!(
            urgent_claim.next_action,
            "reconcile externally terminal tracker state before dispatch"
        );
        assert_eq!(
            checkpoint
                .claims
                .values()
                .filter(|claim| claim.issue_id == "issue-1")
                .count(),
            1
        );
    }

    #[test]
    fn queue_pages_are_bounded_and_derive_waiting_gate_handoff_and_terminal_claims() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-pages",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        for index in 0..55 {
            let issue = queued_issue(
                &format!("issue-{index}"),
                &format!("ORC-{index:02}"),
                "Todo",
                None,
            );
            checkpoint.queue.insert(
                issue.id.clone(),
                queue_item(
                    &issue,
                    AutomationQueueStatus::Queued,
                    "waiting for capacity",
                ),
            );
        }
        let first = store.queue_page(&checkpoint, AutomationQueueCategory::Queued, 0, 100);
        assert_eq!(first.total, 55);
        assert_eq!(first.items.len(), 50);
        assert_eq!(first.next_offset, Some(50));
        let second = store.queue_page(
            &checkpoint,
            AutomationQueueCategory::Queued,
            first.next_offset.unwrap(),
            50,
        );
        assert_eq!(second.items.len(), 5);
        assert_eq!(second.next_offset, None);

        let waiting_id = store.claim_fixture(&mut checkpoint, &issue(), 1).unwrap();
        let waiting = checkpoint.claims.get_mut(&waiting_id).unwrap();
        waiting.status = AutomationClaimStatus::Suspended;
        waiting.effects.push(AutomationEffectReceipt {
            effect_id: "effect-wait".into(),
            idempotency_key: "idem-wait".into(),
            kind: AutomationEffect::TrackerComment,
            claim_id: waiting_id.clone(),
            tracker_project_slug: "orchestra".into(),
            issue_id: waiting.issue_id.clone(),
            request_sha256: "request".into(),
            body_preview: "preview".into(),
            gate_policy: AutomationGatePolicy::AskHuman,
            status: AutomationEffectStatus::WaitingGate,
            provider_receipt: None,
            failure: None,
        });
        assert_eq!(automation_queue_counts(&checkpoint).waiting_gate, 1);
        assert_eq!(
            store
                .queue_page(&checkpoint, AutomationQueueCategory::WaitingGate, 0, 8)
                .items[0]
                .claim_id
                .as_deref(),
            Some(waiting_id.as_str())
        );
    }

    #[test]
    fn normalized_linear_pages_feed_the_same_deterministic_coordinator() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let mut profile = profile(workspace.path());
        profile.agent.max_concurrent_agents = 1;
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-pages-to-queue",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let raw_issue = |id: &str, identifier: &str, priority: i64| {
            json!({
                "id": id,
                "identifier": identifier,
                "title": format!("Issue {identifier}"),
                "priority": priority,
                "state": {"name": "Todo"},
                "labels": {"nodes": [{"name": "automation"}]},
                "createdAt": "2026-07-01T00:00:00Z",
                "updatedAt": "2026-07-16T00:00:00Z"
            })
        };
        let first = crate::normalize_linear_issue_page(&json!({"data": {"project": {"issues": {
            "nodes": [raw_issue("issue-low", "ORC-LOW", 4)],
            "pageInfo": {"hasNextPage": true, "endCursor": "cursor-1"}
        }}}}))
        .unwrap();
        let second = crate::normalize_linear_issue_page(&json!({"data": {"project": {"issues": {
            "nodes": [raw_issue("issue-urgent", "ORC-URGENT", 1)],
            "pageInfo": {"hasNextPage": false, "endCursor": null}
        }}}}))
        .unwrap();
        assert!(first.has_next_page);
        assert_eq!(first.end_cursor.as_deref(), Some("cursor-1"));
        let issues = first
            .issues
            .into_iter()
            .chain(second.issues)
            .collect::<Vec<_>>();
        let result = store
            .coordinate_fixture(&mut checkpoint, &profile, &issues, 1)
            .unwrap();
        assert_eq!(result.dispatched_claim_ids.len(), 1);
        assert_eq!(
            checkpoint.claims[&result.dispatched_claim_ids[0]].issue_identifier,
            "ORC-URGENT"
        );
        assert_eq!(result.counts.queued, 1);
    }

    #[test]
    fn tracker_comment_gate_and_receipt_are_claim_scoped_and_idempotent() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-1",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let claim_id = store.claim_fixture(&mut checkpoint, &issue(), 1).unwrap();
        let executions = std::cell::Cell::new(0);
        let committed = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Implemented and verified.",
                AutomationGatePolicy::AutoAccept,
                |request| {
                    executions.set(executions.get() + 1);
                    assert_eq!(request.claim_id, claim_id);
                    assert_eq!(request.issue_id, "issue-33");
                    AutomationEffectExecution::Committed {
                        provider_receipt: "fixture-comment-1".into(),
                    }
                },
            )
            .unwrap();
        assert_eq!(committed.status, AutomationEffectStatus::Committed);
        assert_eq!(executions.get(), 1);

        let duplicate = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Implemented and verified.",
                AutomationGatePolicy::AutoAccept,
                |_| panic!("a committed idempotency key must not execute twice"),
            )
            .unwrap();
        assert_eq!(duplicate, committed);

        checkpoint.claims.get_mut(&claim_id).unwrap().status = AutomationClaimStatus::Completed;
        let completed_replay = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Implemented and verified.",
                AutomationGatePolicy::AutoAccept,
                |_| panic!("a completed claim must replay its durable receipt"),
            )
            .unwrap();
        assert_eq!(completed_replay, committed);
        assert!(matches!(
            store.resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "A new mutation after completion",
                AutomationGatePolicy::AutoAccept,
                |_| unreachable!(),
            ),
            Err(AutomationRunError::InactiveClaim(_))
        ));
        checkpoint.claims.get_mut(&claim_id).unwrap().status = AutomationClaimStatus::Claimed;
        assert!(matches!(
            store.resolve_tracker_comment(
                &mut checkpoint,
                "claim-from-another-issue",
                &profile,
                "Cross-claim mutation",
                AutomationGatePolicy::AutoAccept,
                |_| unreachable!(),
            ),
            Err(AutomationRunError::MissingClaim(_))
        ));

        let rejected = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Do not publish this variant.",
                AutomationGatePolicy::AutoReject,
                |_| panic!("a rejected gate must precede mutation"),
            )
            .unwrap();
        assert_eq!(rejected.status, AutomationEffectStatus::Rejected);
        let paused = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Ask before publishing this variant.",
                AutomationGatePolicy::AskHuman,
                |_| panic!("a waiting gate must precede mutation"),
            )
            .unwrap();
        assert_eq!(paused.status, AutomationEffectStatus::WaitingGate);
    }

    #[test]
    fn missing_authority_and_interrupted_or_ambiguous_effects_fail_closed() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let mut profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut checkpoint) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-1",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let claim_id = store.claim_fixture(&mut checkpoint, &issue(), 1).unwrap();
        profile.orchestra.effects.clear();
        assert!(matches!(
            store.resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Unauthorized",
                AutomationGatePolicy::AutoAccept,
                |_| unreachable!(),
            ),
            Err(AutomationRunError::MissingEffectAuthority)
        ));
        profile
            .orchestra
            .effects
            .push(AutomationEffect::TrackerComment);
        let ambiguous = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                "Maybe published",
                AutomationGatePolicy::AutoAccept,
                |_| AutomationEffectExecution::Ambiguous {
                    message: "provider timed out after accepting bytes".into(),
                },
            )
            .unwrap();
        assert_eq!(ambiguous.status, AutomationEffectStatus::Ambiguous);

        let crash_body = "crashed after mutation";
        let mut interrupted = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                crash_body,
                AutomationGatePolicy::AskHuman,
                |_| unreachable!(),
            )
            .unwrap();
        interrupted.status = AutomationEffectStatus::Executing;
        let effect = checkpoint
            .claims
            .get_mut(&claim_id)
            .unwrap()
            .effects
            .iter_mut()
            .find(|receipt| receipt.idempotency_key == interrupted.idempotency_key)
            .unwrap();
        *effect = interrupted;
        store.save(&mut checkpoint).unwrap();
        let recovered = store
            .resolve_tracker_comment(
                &mut checkpoint,
                &claim_id,
                &profile,
                crash_body,
                AutomationGatePolicy::AutoAccept,
                |_| panic!("interrupted execution must reconcile before retry"),
            )
            .unwrap();
        assert_eq!(recovered.status, AutomationEffectStatus::Ambiguous);
    }

    #[test]
    fn pause_advances_the_epoch_before_descendants_and_fences_stale_provider_results() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut root) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-pause",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let claim_id = store.claim_fixture(&mut root, &issue(), 1).unwrap();
        let mut stale = root.clone();
        let initial_epoch = root.lease_epoch;
        store.pause(&mut root, "host shutdown").unwrap();
        assert_eq!(root.lease_epoch, initial_epoch + 1);
        assert_eq!(root.status, AutomationRootStatus::Suspended);
        assert_eq!(
            root.reconciliation,
            AutomationReconciliationStatus::Required
        );
        assert_eq!(
            root.claims[&claim_id].status,
            AutomationClaimStatus::Suspended
        );
        assert!(matches!(
            store.update_claim(&mut stale, &claim_id, |_| {}),
            Err(AutomationRunError::StaleLease { .. })
        ));

        store.begin_reconciliation(&mut root).unwrap();
        store
            .reconcile(
                &mut root,
                &profile,
                &[issue()],
                &[AutomationClaimReconciliation {
                    claim_id: claim_id.clone(),
                    issue_task_active: false,
                    descendants_cancelled: false,
                    workflow_status: None,
                }],
            )
            .unwrap();
        store
            .update_claim(&mut root, &claim_id, |claim| {
                claim.status = AutomationClaimStatus::Running;
            })
            .unwrap();
        let provider_result = store.resolve_tracker_comment(
            &mut root,
            &claim_id,
            &profile,
            "Provider accepted this before shutdown.",
            AutomationGatePolicy::AutoAccept,
            |_| {
                let mut authoritative = store.load().unwrap();
                store.pause(&mut authoritative, "host shutdown").unwrap();
                AutomationEffectExecution::Committed {
                    provider_receipt: "late-provider-receipt".into(),
                }
            },
        );
        assert!(matches!(
            provider_result,
            Err(AutomationRunError::StaleLease { .. })
        ));
        let durable = store.load().unwrap();
        assert_eq!(durable.status, AutomationRootStatus::Suspended);
        let effect = &durable.claims[&claim_id].effects[0];
        assert_eq!(effect.status, AutomationEffectStatus::Ambiguous);
        assert_eq!(effect.provider_receipt, None);
    }

    #[test]
    fn resume_reconciles_retained_identities_and_terminal_tracker_state_before_dispatch() {
        let repository = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let profile = profile(workspace.path());
        let digest = crate::canonical_sha256(&serde_json::to_value(&profile).unwrap()).unwrap();
        let (store, mut root) = AutomationRunStore::start(AutomationRunStart {
            repository: repository.path(),
            owner_thread_id: "task-resume",
            source_revision: "abc123",
            profile: &profile,
            profile_digest: &digest,
        })
        .unwrap();
        let claim_id = store.claim_fixture(&mut root, &issue(), 1).unwrap();
        let retained_worktree = root.claims[&claim_id].worktree.clone();
        fs::create_dir_all(&retained_worktree).unwrap();
        let retained_run_id = "child-run-1".to_owned();
        let retained_task = AgentHandle {
            thread_id: "issue-task-1".into(),
            task_path: "/root/issue-task-1".into(),
            parent_thread_id: "task-resume".into(),
        };
        let claim = root.claims.get_mut(&claim_id).unwrap();
        claim.issue_task = Some(retained_task.clone());
        claim.workflow_run_id = Some(retained_run_id.clone());
        store.save(&mut root).unwrap();

        store.pause(&mut root, "desktop pause").unwrap();
        store.begin_reconciliation(&mut root).unwrap();
        store
            .reconcile(
                &mut root,
                &profile,
                &[issue()],
                &[AutomationClaimReconciliation {
                    claim_id: claim_id.clone(),
                    issue_task_active: true,
                    descendants_cancelled: false,
                    workflow_status: Some(RunStatus::Running),
                }],
            )
            .unwrap();
        assert_eq!(root.status, AutomationRootStatus::Running);
        assert_eq!(
            root.reconciliation,
            AutomationReconciliationStatus::Complete
        );
        assert_eq!(root.claims[&claim_id].worktree, retained_worktree);
        assert_eq!(root.claims[&claim_id].issue_task, Some(retained_task));
        assert_eq!(
            root.claims[&claim_id].workflow_run_id.as_deref(),
            Some(retained_run_id.as_str())
        );

        store.pause(&mut root, "tracker refresh").unwrap();
        store.begin_reconciliation(&mut root).unwrap();
        let mut terminal = issue();
        terminal.state = "Done".into();
        let blocked = store.reconcile(
            &mut root,
            &profile,
            &[terminal.clone()],
            &[AutomationClaimReconciliation {
                claim_id: claim_id.clone(),
                issue_task_active: false,
                descendants_cancelled: false,
                workflow_status: Some(RunStatus::Cancelled),
            }],
        );
        assert!(matches!(
            blocked,
            Err(AutomationRunError::ReconciliationBlocked(_))
        ));
        assert_eq!(root.status, AutomationRootStatus::Suspended);
        assert_eq!(root.reconciliation, AutomationReconciliationStatus::Blocked);

        store.begin_reconciliation(&mut root).unwrap();
        store
            .reconcile(
                &mut root,
                &profile,
                &[terminal],
                &[AutomationClaimReconciliation {
                    claim_id: claim_id.clone(),
                    issue_task_active: false,
                    descendants_cancelled: true,
                    workflow_status: Some(RunStatus::Cancelled),
                }],
            )
            .unwrap();
        assert_eq!(
            root.claims[&claim_id].status,
            AutomationClaimStatus::Cancelled
        );
        assert!(
            root.claims[&claim_id]
                .next_action
                .contains("cleanup eligible")
        );
    }
}
