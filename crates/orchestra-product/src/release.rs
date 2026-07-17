use crate::{ProductError, ReleaseManifest, verify_manifest_identity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const RELEASE_POLICY_SCHEMA_VERSION: u32 = 1;
pub const RELEASE_EVIDENCE_SCHEMA_VERSION: u32 = 1;
pub const RELEASE_RECORD_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReleasePolicy {
    pub schema_version: u32,
    pub updater: UpdaterPolicy,
    pub distribution: DistributionPolicy,
    pub state: StatePolicy,
    pub plugin: PluginPolicy,
    pub materials: MaterialPolicy,
    pub gates: GatePolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct UpdaterPolicy {
    pub implementation: String,
    pub version: String,
    pub full_app_only: bool,
    pub metadata_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DistributionPolicy {
    pub minimum_macos: String,
    pub targets: Vec<String>,
    pub retained_predecessors: usize,
    pub automatic_rollback_attempts: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StatePolicy {
    pub canonical_paths: Vec<String>,
    pub projection_root: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all(serialize = "camelCase"))]
pub struct PluginPolicy {
    #[serde(alias = "baselineActive")]
    pub baseline_active: bool,
    #[serde(alias = "installsNativeExecution")]
    pub installs_native_execution: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GatePolicy {
    pub required: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaterialPolicy {
    pub required: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseEvidence {
    pub schema_version: u32,
    pub product_version: String,
    pub updater_version: String,
    pub targets: Vec<TargetEvidence>,
    pub gates: BTreeMap<String, GateEvidence>,
    pub state: StateEvidence,
    pub plugin: PluginPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TargetEvidence {
    pub target: String,
    pub manifest: String,
    pub manifest_sha256: String,
    pub archive: String,
    pub archive_sha256: String,
    pub codesign_verified: bool,
    pub hardened_runtime: bool,
    pub renderer_sandboxed: bool,
    pub notarized: bool,
    pub stapled: bool,
    pub machine_verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GateEvidence {
    pub status: GateStatus,
    pub evidence: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Passed,
    Failed,
    Pending,
    Skipped,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StateEvidence {
    pub canonical_rollouts_preserved: bool,
    pub repository_checkpoints_preserved: bool,
    pub side_by_side_projection_generations: bool,
    pub maintenance_lease_verified: bool,
    pub rollback_barriers_verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseRecord {
    pub schema_version: u32,
    pub product_version: String,
    pub minimum_macos: String,
    pub updater: UpdaterPolicy,
    pub manifests: BTreeMap<String, FileIdentity>,
    pub archives: BTreeMap<String, FileIdentity>,
    pub materials: BTreeMap<String, FileIdentity>,
    pub gates: BTreeMap<String, GateEvidence>,
    pub canonical_paths: Vec<String>,
    pub projection_root: String,
    pub plugin: PluginPolicy,
    pub record_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsignedReleaseRecord {
    schema_version: u32,
    product_version: String,
    minimum_macos: String,
    updater: UpdaterPolicy,
    manifests: BTreeMap<String, FileIdentity>,
    archives: BTreeMap<String, FileIdentity>,
    materials: BTreeMap<String, FileIdentity>,
    gates: BTreeMap<String, GateEvidence>,
    canonical_paths: Vec<String>,
    projection_root: String,
    plugin: PluginPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileIdentity {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PublicationEvidence {
    pub record_sha256: String,
    pub immutable_artifacts_published: bool,
    pub immutable_evidence_published: bool,
    pub published_assets: BTreeMap<String, String>,
    pub metadata_signed: bool,
    pub appcast: String,
    pub appcast_sha256: String,
    pub signature: String,
    pub signature_sha256: String,
}

pub fn read_release_policy(root: &Path) -> Result<ReleasePolicy, ProductError> {
    let repository_path = root.join("product/release.toml");
    let path = if repository_path.exists() {
        repository_path
    } else {
        root.join("release.toml")
    };
    let policy: ReleasePolicy = toml::from_str(&fs::read_to_string(&path).map_err(|error| {
        ProductError::Message(format!("failed to read {}: {error}", path.display()))
    })?)?;
    validate_policy(&policy)?;
    Ok(policy)
}

pub fn read_release_evidence(path: &Path) -> Result<ReleaseEvidence, ProductError> {
    Ok(serde_json::from_slice(&fs::read(path).map_err(
        |error| ProductError::Message(format!("failed to read {}: {error}", path.display())),
    )?)?)
}

pub fn build_release_record(
    candidate: &Path,
    policy: &ReleasePolicy,
    evidence: &ReleaseEvidence,
) -> Result<ReleaseRecord, ProductError> {
    validate_policy(policy)?;
    if evidence.schema_version != RELEASE_EVIDENCE_SCHEMA_VERSION {
        return message(format!(
            "release evidence schema {} is unsupported",
            evidence.schema_version
        ));
    }
    if evidence.updater_version != policy.updater.version {
        return message(format!(
            "updater evidence pins {}, expected {}",
            evidence.updater_version, policy.updater.version
        ));
    }
    if evidence.plugin != policy.plugin {
        return message("plugin baseline evidence does not match release policy");
    }
    if policy.plugin.baseline_active || policy.plugin.installs_native_execution {
        return message("the bundled Authoring plugin baseline must be inactive and non-native");
    }
    verify_state_evidence(&evidence.state)?;

    let expected_targets = policy
        .distribution
        .targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_targets = evidence
        .targets
        .iter()
        .map(|target| target.target.clone())
        .collect::<BTreeSet<_>>();
    if actual_targets.len() != evidence.targets.len() {
        return message("release evidence contains a duplicate target");
    }
    if actual_targets != expected_targets {
        return message(format!(
            "release targets {:?} do not match policy {:?}",
            actual_targets, expected_targets
        ));
    }

    let mut manifests = BTreeMap::new();
    let mut archives = BTreeMap::new();
    for target in &evidence.targets {
        verify_target_flags(target)?;
        let manifest_path = contained_path(candidate, &target.manifest)?;
        let manifest_bytes = fs::read(&manifest_path)?;
        verify_digest("manifest", &manifest_bytes, &target.manifest_sha256)?;
        let manifest: ReleaseManifest = serde_json::from_slice(&manifest_bytes)?;
        verify_manifest_identity(&manifest)?;
        if manifest.target != target.target {
            return message(format!(
                "manifest target {} does not match evidence {}",
                manifest.target, target.target
            ));
        }
        if manifest.product_version != evidence.product_version {
            return message(format!(
                "manifest version {} does not match release {}",
                manifest.product_version, evidence.product_version
            ));
        }
        if manifest.minimum_macos != policy.distribution.minimum_macos {
            return message(format!(
                "manifest minimum macOS {} does not match policy {}",
                manifest.minimum_macos, policy.distribution.minimum_macos
            ));
        }
        manifests.insert(
            target.target.clone(),
            file_identity(candidate, &target.manifest)?,
        );

        let archive_path = contained_path(candidate, &target.archive)?;
        let archive_bytes = fs::read(&archive_path)?;
        verify_digest("archive", &archive_bytes, &target.archive_sha256)?;
        archives.insert(
            target.target.clone(),
            file_identity(candidate, &target.archive)?,
        );
    }

    let required_gates = policy
        .gates
        .required
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_gates = evidence.gates.keys().cloned().collect::<BTreeSet<_>>();
    if actual_gates != required_gates {
        return message(format!(
            "release gates {:?} do not match required {:?}",
            actual_gates, required_gates
        ));
    }
    for (name, gate) in &evidence.gates {
        if gate.status != GateStatus::Passed {
            return message(format!("release gate `{name}` is not passed"));
        }
        let bytes = fs::read(contained_path(candidate, &gate.evidence)?)?;
        verify_digest(&format!("gate `{name}`"), &bytes, &gate.sha256)?;
    }

    let materials = policy
        .materials
        .required
        .iter()
        .map(|relative| Ok((relative.clone(), file_identity(candidate, relative)?)))
        .collect::<Result<BTreeMap<_, _>, ProductError>>()?;

    let unsigned = UnsignedReleaseRecord {
        schema_version: RELEASE_RECORD_SCHEMA_VERSION,
        product_version: evidence.product_version.clone(),
        minimum_macos: policy.distribution.minimum_macos.clone(),
        updater: policy.updater.clone(),
        manifests,
        archives,
        materials,
        gates: evidence.gates.clone(),
        canonical_paths: policy.state.canonical_paths.clone(),
        projection_root: policy.state.projection_root.clone(),
        plugin: policy.plugin.clone(),
    };
    let record_sha256 = digest(&serde_json::to_vec(&unsigned)?);
    Ok(ReleaseRecord {
        schema_version: unsigned.schema_version,
        product_version: unsigned.product_version,
        minimum_macos: unsigned.minimum_macos,
        updater: unsigned.updater,
        manifests: unsigned.manifests,
        archives: unsigned.archives,
        materials: unsigned.materials,
        gates: unsigned.gates,
        canonical_paths: unsigned.canonical_paths,
        projection_root: unsigned.projection_root,
        plugin: unsigned.plugin,
        record_sha256,
    })
}

pub fn verify_publication(
    candidate: &Path,
    record: &ReleaseRecord,
    evidence: &PublicationEvidence,
) -> Result<(), ProductError> {
    verify_release_record_identity(record)?;
    if evidence.record_sha256 != record.record_sha256 {
        return message("publication evidence references a different release record");
    }
    if !evidence.immutable_artifacts_published || !evidence.immutable_evidence_published {
        return message("artifacts and evidence must be immutable before appcast publication");
    }
    let expected_assets = published_assets(record);
    if evidence.published_assets != expected_assets {
        return message("published asset identities do not match the sealed release record");
    }
    if !evidence.metadata_signed {
        return message("updater metadata is not signed");
    }
    for (label, relative, expected) in [
        ("appcast", &evidence.appcast, &evidence.appcast_sha256),
        (
            "appcast signature",
            &evidence.signature,
            &evidence.signature_sha256,
        ),
    ] {
        let bytes = fs::read(contained_path(candidate, relative)?)?;
        if bytes.is_empty() {
            return message(format!("{label} is empty"));
        }
        verify_digest(label, &bytes, expected)?;
    }
    Ok(())
}

pub fn write_release_record(path: &Path, record: &ReleaseRecord) -> Result<(), ProductError> {
    atomic_write_json(path, record)
}

pub fn verify_release_record_identity(record: &ReleaseRecord) -> Result<(), ProductError> {
    let unsigned = UnsignedReleaseRecord {
        schema_version: record.schema_version,
        product_version: record.product_version.clone(),
        minimum_macos: record.minimum_macos.clone(),
        updater: record.updater.clone(),
        manifests: record.manifests.clone(),
        archives: record.archives.clone(),
        materials: record.materials.clone(),
        gates: record.gates.clone(),
        canonical_paths: record.canonical_paths.clone(),
        projection_root: record.projection_root.clone(),
        plugin: record.plugin.clone(),
    };
    let actual = digest(&serde_json::to_vec(&unsigned)?);
    if actual != record.record_sha256 {
        return message(format!(
            "release record identity mismatch: expected {}, calculated {actual}",
            record.record_sha256
        ));
    }
    Ok(())
}

fn validate_policy(policy: &ReleasePolicy) -> Result<(), ProductError> {
    if policy.schema_version != RELEASE_POLICY_SCHEMA_VERSION {
        return message(format!(
            "release policy schema {} is unsupported",
            policy.schema_version
        ));
    }
    if policy.updater.implementation.trim().is_empty()
        || policy.updater.version.trim().is_empty()
        || !policy.updater.full_app_only
    {
        return message("release updater must be exactly pinned and full-app only");
    }
    if policy.distribution.targets.len() != 2
        || !policy
            .distribution
            .targets
            .iter()
            .any(|target| target == "aarch64-apple-darwin")
        || !policy
            .distribution
            .targets
            .iter()
            .any(|target| target == "x86_64-apple-darwin")
    {
        return message("release policy must contain arm64 and x86_64 macOS targets");
    }
    if policy.distribution.retained_predecessors != 2
        || policy.distribution.automatic_rollback_attempts != 1
    {
        return message("release policy must retain two predecessors and allow one auto rollback");
    }
    if policy.state.canonical_paths.is_empty()
        || policy.gates.required.is_empty()
        || policy.materials.required.is_empty()
    {
        return message(
            "release policy must declare canonical paths, materials, and non-waivable gates",
        );
    }
    Ok(())
}

fn published_assets(record: &ReleaseRecord) -> BTreeMap<String, String> {
    let mut assets = BTreeMap::new();
    for identity in record
        .manifests
        .values()
        .chain(record.archives.values())
        .chain(record.materials.values())
    {
        assets.insert(identity.path.clone(), identity.sha256.clone());
    }
    for gate in record.gates.values() {
        assets.insert(gate.evidence.clone(), gate.sha256.clone());
    }
    assets
}

fn verify_state_evidence(evidence: &StateEvidence) -> Result<(), ProductError> {
    if !evidence.canonical_rollouts_preserved
        || !evidence.repository_checkpoints_preserved
        || !evidence.side_by_side_projection_generations
        || !evidence.maintenance_lease_verified
        || !evidence.rollback_barriers_verified
    {
        return message("state preservation evidence is incomplete");
    }
    Ok(())
}

fn verify_target_flags(target: &TargetEvidence) -> Result<(), ProductError> {
    if !target.codesign_verified
        || !target.hardened_runtime
        || !target.renderer_sandboxed
        || !target.notarized
        || !target.stapled
        || !target.machine_verified
    {
        return message(format!(
            "distribution evidence for {} is incomplete",
            target.target
        ));
    }
    Ok(())
}

fn contained_path(root: &Path, relative: &str) -> Result<PathBuf, ProductError> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return message(format!(
            "release evidence path `{relative}` escapes the candidate"
        ));
    }
    Ok(root.join(path))
}

fn file_identity(root: &Path, relative: &str) -> Result<FileIdentity, ProductError> {
    let bytes = fs::read(contained_path(root, relative)?)?;
    Ok(FileIdentity {
        path: relative.into(),
        bytes: bytes.len() as u64,
        sha256: digest(&bytes),
    })
}

fn verify_digest(label: &str, bytes: &[u8], expected: &str) -> Result<(), ProductError> {
    let actual = digest(bytes);
    if actual != expected {
        return message(format!(
            "{label} digest mismatch: expected {expected}, calculated {actual}"
        ));
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn message<T>(value: impl Into<String>) -> Result<T, ProductError> {
    Err(ProductError::Message(value.into()))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProductInstallState {
    pub schema_version: u32,
    pub active: ReleaseSlot,
    pub pending: Option<PendingRelease>,
    pub retained_predecessors: Vec<ReleaseSlot>,
    pub automatic_rollbacks_used: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_rollback_manifest_sha256: Option<String>,
    pub projection_generations: BTreeMap<String, String>,
    pub transitions: Vec<UpdateTransition>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseSlot {
    pub product_version: String,
    pub manifest_sha256: String,
    pub snapshot_schema: String,
    pub projection_schema: String,
}

impl ReleaseSlot {
    pub fn from_manifest(manifest: &ReleaseManifest) -> Result<Self, ProductError> {
        verify_manifest_identity(manifest)?;
        let snapshot = manifest.schemas.get("snapshot").ok_or_else(|| {
            ProductError::Message("Product manifest has no snapshot schema".into())
        })?;
        let protocol = manifest.schemas.get("protocol").ok_or_else(|| {
            ProductError::Message("Product manifest has no protocol schema".into())
        })?;
        Ok(Self {
            product_version: manifest.product_version.clone(),
            manifest_sha256: manifest.manifest_sha256.clone(),
            snapshot_schema: snapshot.identity.clone(),
            projection_schema: format!("{}:{}", protocol.identity, protocol.sha256),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PendingRelease {
    pub release: ReleaseSlot,
    pub phase: UpdatePhase,
    pub predecessor_manifest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    Staged,
    FirstLaunchPending,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UpdateTransition {
    pub action: String,
    pub from_manifest_sha256: String,
    pub to_manifest_sha256: String,
    pub recorded_at: String,
}

impl ProductInstallState {
    pub fn new(active: ReleaseSlot) -> Self {
        let mut projection_generations = BTreeMap::new();
        projection_generations.insert(
            active.projection_schema.clone(),
            projection_generation(&active.manifest_sha256),
        );
        Self {
            schema_version: INSTALL_STATE_SCHEMA_VERSION,
            active,
            pending: None,
            retained_predecessors: Vec::new(),
            automatic_rollbacks_used: 0,
            automatic_rollback_manifest_sha256: None,
            projection_generations,
            transitions: Vec::new(),
        }
    }

    pub fn stage(&mut self, release: ReleaseSlot, recorded_at: &str) -> Result<(), ProductError> {
        self.validate()?;
        if self.pending.is_some() {
            return message("an update is already pending");
        }
        if release.manifest_sha256 == self.active.manifest_sha256 {
            return message("cannot stage the active release");
        }
        if self.automatic_rollback_manifest_sha256.as_deref()
            != Some(release.manifest_sha256.as_str())
        {
            self.automatic_rollbacks_used = 0;
        }
        self.projection_generations
            .entry(release.projection_schema.clone())
            .or_insert_with(|| projection_generation(&release.manifest_sha256));
        self.transitions.push(UpdateTransition {
            action: "stage".into(),
            from_manifest_sha256: self.active.manifest_sha256.clone(),
            to_manifest_sha256: release.manifest_sha256.clone(),
            recorded_at: recorded_at.into(),
        });
        self.pending = Some(PendingRelease {
            predecessor_manifest_sha256: self.active.manifest_sha256.clone(),
            release,
            phase: UpdatePhase::Staged,
        });
        Ok(())
    }

    pub fn activate(
        &mut self,
        retained_limit: usize,
        recorded_at: &str,
    ) -> Result<(), ProductError> {
        self.validate()?;
        let pending = self
            .pending
            .as_mut()
            .ok_or_else(|| ProductError::Message("no staged update exists".into()))?;
        if pending.phase != UpdatePhase::Staged {
            return message("the pending update is already activated");
        }
        let previous = std::mem::replace(&mut self.active, pending.release.clone());
        self.retained_predecessors.insert(0, previous.clone());
        self.retained_predecessors.truncate(retained_limit);
        pending.phase = UpdatePhase::FirstLaunchPending;
        self.transitions.push(UpdateTransition {
            action: "activate".into(),
            from_manifest_sha256: previous.manifest_sha256,
            to_manifest_sha256: self.active.manifest_sha256.clone(),
            recorded_at: recorded_at.into(),
        });
        Ok(())
    }

    pub fn abort_staged(&mut self, recorded_at: &str) -> Result<(), ProductError> {
        self.validate()?;
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| ProductError::Message("no staged update exists".into()))?;
        if pending.phase != UpdatePhase::Staged {
            return message("cannot abort an update after activation");
        }
        self.transitions.push(UpdateTransition {
            action: "abort_stage".into(),
            from_manifest_sha256: pending.release.manifest_sha256.clone(),
            to_manifest_sha256: self.active.manifest_sha256.clone(),
            recorded_at: recorded_at.into(),
        });
        self.pending = None;
        Ok(())
    }

    pub fn commit_first_launch(&mut self, recorded_at: &str) -> Result<(), ProductError> {
        self.validate()?;
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| ProductError::Message("no first launch is pending".into()))?;
        if pending.phase != UpdatePhase::FirstLaunchPending {
            return message("the staged update has not been activated");
        }
        let manifest = self.active.manifest_sha256.clone();
        self.transitions.push(UpdateTransition {
            action: "commit".into(),
            from_manifest_sha256: manifest.clone(),
            to_manifest_sha256: manifest,
            recorded_at: recorded_at.into(),
        });
        self.pending = None;
        self.automatic_rollbacks_used = 0;
        self.automatic_rollback_manifest_sha256 = None;
        Ok(())
    }

    pub fn rollback_failed_first_launch(
        &mut self,
        automatic_limit: u32,
        recorded_at: &str,
    ) -> Result<(), ProductError> {
        self.validate()?;
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| ProductError::Message("no first launch is pending".into()))?;
        if pending.phase != UpdatePhase::FirstLaunchPending {
            return message("rollback is only valid after activation");
        }
        if self.automatic_rollbacks_used >= automatic_limit {
            return message("automatic rollback limit is exhausted");
        }
        let predecessor_sha = pending.predecessor_manifest_sha256.clone();
        let index = self
            .retained_predecessors
            .iter()
            .position(|slot| slot.manifest_sha256 == predecessor_sha)
            .ok_or_else(|| ProductError::Message("rollback predecessor is not retained".into()))?;
        let predecessor = self.retained_predecessors.remove(index);
        let failed = std::mem::replace(&mut self.active, predecessor);
        let failed_manifest_sha256 = failed.manifest_sha256.clone();
        self.transitions.push(UpdateTransition {
            action: "automatic_rollback".into(),
            from_manifest_sha256: failed_manifest_sha256.clone(),
            to_manifest_sha256: self.active.manifest_sha256.clone(),
            recorded_at: recorded_at.into(),
        });
        self.pending = None;
        self.automatic_rollbacks_used += 1;
        self.automatic_rollback_manifest_sha256 = Some(failed_manifest_sha256);
        Ok(())
    }

    pub fn reverse_to_retained(
        &mut self,
        manifest_sha256: &str,
        allow_snapshot_schema_reverse: bool,
        retained_limit: usize,
        recorded_at: &str,
    ) -> Result<(), ProductError> {
        self.validate()?;
        if self.pending.is_some() {
            return message("cannot reverse while an update is pending");
        }
        let index = self
            .retained_predecessors
            .iter()
            .position(|slot| slot.manifest_sha256 == manifest_sha256)
            .ok_or_else(|| ProductError::Message("requested release is not retained".into()))?;
        let target = self.retained_predecessors[index].clone();
        if target.snapshot_schema != self.active.snapshot_schema && !allow_snapshot_schema_reverse {
            return message("snapshot schema rollback barrier requires an explicit migration");
        }
        self.retained_predecessors.remove(index);
        let previous = std::mem::replace(&mut self.active, target);
        self.retained_predecessors.insert(0, previous.clone());
        self.retained_predecessors.truncate(retained_limit);
        self.transitions.push(UpdateTransition {
            action: "reverse".into(),
            from_manifest_sha256: previous.manifest_sha256,
            to_manifest_sha256: self.active.manifest_sha256.clone(),
            recorded_at: recorded_at.into(),
        });
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProductError> {
        if self.schema_version != INSTALL_STATE_SCHEMA_VERSION {
            return message(format!(
                "install state schema {} is unsupported",
                self.schema_version
            ));
        }
        if self.active.manifest_sha256.trim().is_empty() {
            return message("active release has no manifest identity");
        }
        Ok(())
    }
}

pub fn read_install_state(path: &Path) -> Result<ProductInstallState, ProductError> {
    let state: ProductInstallState = serde_json::from_slice(&fs::read(path)?)?;
    state.validate()?;
    Ok(state)
}

pub fn write_install_state(path: &Path, state: &ProductInstallState) -> Result<(), ProductError> {
    state.validate()?;
    atomic_write_json(path, state)
}

fn projection_generation(manifest_sha256: &str) -> String {
    format!(
        "generation-{}",
        &manifest_sha256[..manifest_sha256.len().min(12)]
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MaintenanceLeaseRecord<'a> {
    operation: &'a str,
    holder: &'a str,
}

#[derive(Debug)]
pub struct MaintenanceLease {
    path: PathBuf,
}

impl Drop for MaintenanceLease {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn acquire_maintenance_lease(
    codex_home: &Path,
    operation: &str,
    holder: &str,
) -> Result<MaintenanceLease, ProductError> {
    if operation.trim().is_empty() || holder.trim().is_empty() {
        return message("maintenance lease operation and holder are required");
    }
    let path = codex_home.join("orchestra/maintenance.lock");
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| ProductError::Message("maintenance lease path has no parent".into()))?,
    )?;
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return message("Codex home already has an active maintenance lease");
        }
        Err(error) => return Err(error.into()),
    }
    if let Err(error) = atomic_write_json(
        &path.join("lease.json"),
        &MaintenanceLeaseRecord { operation, holder },
    ) {
        let _ = fs::remove_dir_all(&path);
        return Err(error);
    }
    Ok(MaintenanceLease { path })
}

fn atomic_write_json(path: &Path, value: &impl Serialize) -> Result<(), ProductError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtifactInput, ProductIdentity, ProductPins, SchemaPins, build_manifest};
    use tempfile::tempdir;

    fn policy() -> ReleasePolicy {
        ReleasePolicy {
            schema_version: 1,
            updater: UpdaterPolicy {
                implementation: "electron-updater".into(),
                version: "6.8.3".into(),
                full_app_only: true,
                metadata_digest: "sha512".into(),
            },
            distribution: DistributionPolicy {
                minimum_macos: "13.0".into(),
                targets: vec!["aarch64-apple-darwin".into(), "x86_64-apple-darwin".into()],
                retained_predecessors: 2,
                automatic_rollback_attempts: 1,
            },
            state: StatePolicy {
                canonical_paths: vec![
                    "$CODEX_HOME/sessions".into(),
                    ".codex/orchestra/runs".into(),
                ],
                projection_root: "$CODEX_HOME/orchestra/projections".into(),
            },
            plugin: PluginPolicy {
                baseline_active: false,
                installs_native_execution: false,
            },
            materials: MaterialPolicy {
                required: vec!["evidence/licenses.txt".into()],
            },
            gates: GatePolicy {
                required: vec!["repository".into(), "human-evidence".into()],
            },
        }
    }

    fn pins() -> ProductPins {
        ProductPins {
            product: ProductIdentity {
                version: "1.2.3".into(),
                minimum_macos: "13.0".into(),
            },
            sources: BTreeMap::from([
                ("codex".into(), "aaaaaaaa".into()),
                ("t3code".into(), "bbbbbbbb".into()),
            ]),
            schemas: SchemaPins {
                protocol: "protocol-v1".into(),
                snapshot: "snapshot-v1".into(),
            },
            evaluator: crate::EvaluatorPins {
                revision: "evaluator-v1".into(),
                adapter_abi: "adapter-v1".into(),
                canonicalizer: "jcs-v1".into(),
                issue_format: "issues-v1".into(),
            },
            capabilities: vec!["orchestra/query".into()],
            limits: BTreeMap::new(),
        }
    }

    fn sha(bytes: &[u8]) -> String {
        digest(bytes)
    }

    fn evidence(root: &Path) -> ReleaseEvidence {
        fs::create_dir_all(root.join("evidence")).unwrap();
        fs::write(root.join("evidence/licenses.txt"), "license evidence\n").unwrap();
        let mut targets = Vec::new();
        for target in ["aarch64-apple-darwin", "x86_64-apple-darwin"] {
            let directory = root.join(target);
            fs::create_dir_all(&directory).unwrap();
            let component = directory.join("codex");
            fs::write(&component, format!("codex-{target}")).unwrap();
            let manifest = build_manifest(
                pins(),
                target,
                &[ArtifactInput {
                    name: "codex".into(),
                    path: component,
                }],
            )
            .unwrap();
            let manifest_relative = format!("{target}/release-manifest.json");
            crate::write_manifest(&root.join(&manifest_relative), &manifest).unwrap();
            let archive_relative = format!("{target}/Orchestra.zip");
            fs::write(root.join(&archive_relative), format!("archive-{target}")).unwrap();
            targets.push(TargetEvidence {
                target: target.into(),
                manifest_sha256: sha(&fs::read(root.join(&manifest_relative)).unwrap()),
                manifest: manifest_relative,
                archive_sha256: sha(&fs::read(root.join(&archive_relative)).unwrap()),
                archive: archive_relative,
                codesign_verified: true,
                hardened_runtime: true,
                renderer_sandboxed: true,
                notarized: true,
                stapled: true,
                machine_verified: true,
            });
        }
        let mut gates = BTreeMap::new();
        for gate in ["repository", "human-evidence"] {
            let relative = format!("evidence/{gate}.json");
            fs::create_dir_all(root.join("evidence")).unwrap();
            fs::write(root.join(&relative), format!("{{\"gate\":\"{gate}\"}}\n")).unwrap();
            gates.insert(
                gate.into(),
                GateEvidence {
                    status: GateStatus::Passed,
                    sha256: sha(&fs::read(root.join(&relative)).unwrap()),
                    evidence: relative,
                },
            );
        }
        ReleaseEvidence {
            schema_version: 1,
            product_version: "1.2.3".into(),
            updater_version: "6.8.3".into(),
            targets,
            gates,
            state: StateEvidence {
                canonical_rollouts_preserved: true,
                repository_checkpoints_preserved: true,
                side_by_side_projection_generations: true,
                maintenance_lease_verified: true,
                rollback_barriers_verified: true,
            },
            plugin: PluginPolicy {
                baseline_active: false,
                installs_native_execution: false,
            },
        }
    }

    fn slot(version: &str, sha: &str, snapshot: &str, projection: &str) -> ReleaseSlot {
        ReleaseSlot {
            product_version: version.into(),
            manifest_sha256: sha.into(),
            snapshot_schema: snapshot.into(),
            projection_schema: projection.into(),
        }
    }

    #[test]
    fn repository_release_policy_is_valid_and_exactly_pinned() {
        let policy: ReleasePolicy =
            toml::from_str(include_str!("../../../product/release.toml")).unwrap();
        validate_policy(&policy).unwrap();
        assert_eq!(policy.updater.version, "6.8.3");
        assert!(policy.updater.full_app_only);
    }

    #[test]
    fn candidate_requires_both_complete_architectures_and_all_gates() {
        let temp = tempdir().unwrap();
        let evidence = evidence(temp.path());
        let record = build_release_record(temp.path(), &policy(), &evidence).unwrap();
        assert_eq!(record.manifests.len(), 2);
        assert_eq!(record.archives.len(), 2);
        assert_eq!(record.record_sha256.len(), 64);

        let mut incomplete = evidence;
        incomplete.targets[0].notarized = false;
        assert!(build_release_record(temp.path(), &policy(), &incomplete).is_err());
    }

    #[test]
    fn publication_is_blocked_until_assets_evidence_and_metadata_are_signed() {
        let temp = tempdir().unwrap();
        let record = build_release_record(temp.path(), &policy(), &evidence(temp.path())).unwrap();
        fs::write(temp.path().join("appcast.yml"), "version: 1.2.3\n").unwrap();
        fs::write(temp.path().join("appcast.sig"), "signature").unwrap();
        let mut publication = PublicationEvidence {
            record_sha256: record.record_sha256.clone(),
            immutable_artifacts_published: true,
            immutable_evidence_published: true,
            published_assets: published_assets(&record),
            metadata_signed: true,
            appcast: "appcast.yml".into(),
            appcast_sha256: sha(&fs::read(temp.path().join("appcast.yml")).unwrap()),
            signature: "appcast.sig".into(),
            signature_sha256: sha(&fs::read(temp.path().join("appcast.sig")).unwrap()),
        };
        verify_publication(temp.path(), &record, &publication).unwrap();
        publication.immutable_evidence_published = false;
        assert!(verify_publication(temp.path(), &record, &publication).is_err());
    }

    #[test]
    fn failed_first_launch_rolls_back_once_and_keeps_projection_generations() {
        let mut state = ProductInstallState::new(slot("1", "aaaaaaaaaaaa", "s1", "p1"));
        state
            .stage(slot("2", "bbbbbbbbbbbb", "s1", "p2"), "stage")
            .unwrap();
        state.activate(2, "activate").unwrap();
        state.rollback_failed_first_launch(1, "rollback").unwrap();
        assert_eq!(state.active.product_version, "1");
        assert_eq!(state.automatic_rollbacks_used, 1);
        assert!(state.projection_generations.contains_key("p1"));
        assert!(state.projection_generations.contains_key("p2"));
        state
            .stage(slot("2", "bbbbbbbbbbbb", "s1", "p2"), "retry")
            .unwrap();
        state.activate(2, "reactivate").unwrap();
        assert!(state.rollback_failed_first_launch(1, "again").is_err());
    }

    #[test]
    fn staged_release_can_be_aborted_without_changing_the_active_release() {
        let active = slot("1", "aaaaaaaaaaaa", "s1", "p1");
        let mut state = ProductInstallState::new(active.clone());
        state
            .stage(slot("2", "bbbbbbbbbbbb", "s1", "p2"), "stage")
            .unwrap();

        state.abort_staged("abort").unwrap();

        assert_eq!(state.active, active);
        assert!(state.pending.is_none());
        assert_eq!(state.transitions.last().unwrap().action, "abort_stage");
    }

    #[test]
    fn explicit_reverse_honors_snapshot_schema_barrier() {
        let mut state = ProductInstallState::new(slot("1", "aaaaaaaaaaaa", "s1", "p1"));
        state
            .stage(slot("2", "bbbbbbbbbbbb", "s2", "p2"), "stage")
            .unwrap();
        state.activate(2, "activate").unwrap();
        state.commit_first_launch("commit").unwrap();
        assert!(
            state
                .reverse_to_retained("aaaaaaaaaaaa", false, 2, "reverse")
                .is_err()
        );
        state
            .reverse_to_retained("aaaaaaaaaaaa", true, 2, "reverse")
            .unwrap();
        assert_eq!(state.active.product_version, "1");
    }

    #[test]
    fn codex_home_maintenance_lease_is_exclusive_and_released_on_drop() {
        let temp = tempdir().unwrap();
        let first = acquire_maintenance_lease(temp.path(), "projection", "release-1").unwrap();
        assert!(acquire_maintenance_lease(temp.path(), "rollback", "release-2").is_err());
        drop(first);
        acquire_maintenance_lease(temp.path(), "rollback", "release-2").unwrap();
    }
}
