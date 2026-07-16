//! Orchestra's host-independent workflow compiler and runtime.
//!
//! Workflow TypeScript is parsed as a restricted data language. It is never
//! evaluated by a JavaScript engine and cannot execute user code.

mod automation;
mod automation_run;
mod compiler;
mod compiler_artifact;
mod context;
mod evaluator;
mod host;
mod inputs;
mod linear;
mod plan;
mod query;
mod runtime;
mod skills;
mod state;
mod validate;

pub use automation::{
    AutomationAgentProfile, AutomationCodexPolicy, AutomationDiagnostic, AutomationDiagnosticCode,
    AutomationEffect, AutomationHooksProfile, AutomationIssue, AutomationIssueBlocker,
    AutomationOrchestraProfile, AutomationPollingProfile, AutomationProfile, AutomationSecretKind,
    AutomationSecretReference, AutomationTrackerProfile, AutomationValidationRequest,
    AutomationValidationResult, AutomationValidationSeverity, AutomationWorkflowInput,
    AutomationWorkflowPreview, AutomationWorkspaceProfile, InheritedCodexPolicy,
    validate_automation_profile,
};
pub use automation_run::{
    AutomationClaimLiveness, AutomationClaimReconciliation, AutomationClaimStatus,
    AutomationCoordinationResult, AutomationEffectExecution, AutomationEffectReceipt,
    AutomationEffectStatus, AutomationGatePolicy, AutomationIssueClaim, AutomationQueueCategory,
    AutomationQueueCounts, AutomationQueueItem, AutomationQueuePage, AutomationQueueProjectionItem,
    AutomationQueueStatus, AutomationReconciliationStatus, AutomationRetryKind,
    AutomationRetrySchedule, AutomationRootCheckpoint, AutomationRootStatus, AutomationRunError,
    AutomationRunStart, AutomationRunStore, AutomationTrackerCommentRequest,
    automation_claim_liveness, automation_queue_counts, automation_queue_page,
};
pub use compiler::{CompileError, compile_workflow};
pub use compiler_artifact::{
    ArtifactCompileError, CompatibilityTuple, CompileLimits, CompileRequest, CompiledWorkflow,
    CustomCodec, ModuleIdentity, SchemaBundle, compile_artifact,
};
pub use context::{
    ContextBundle, ContextError, materialize_context, materialize_context_with_inputs,
};
pub use evaluator::{
    Evaluator, EvaluatorError, EvaluatorFailure, EvaluatorLimits, EvaluatorProvenance,
    ValidationIssue, ValidationOutcome, ValidationRequest, ValuePathSegment, canonical_json,
    canonical_sha256,
};
pub use host::{
    AgentHandle, AgentOutcome, AgentStatus, CommandOutcome, NativeHost, ResolvedSkill,
    SkillIdentity, SkillSourceKind, SkillToolDependency, SpawnRequest,
};
pub use inputs::{
    InputError, ResolvedInputs, RunInputs, resolve_inputs, resolve_template, verify_inputs,
};
pub use linear::{
    LinearIssuePage, LinearReadError, normalize_linear_issue, normalize_linear_issue_page,
};
pub use plan::*;
pub use query::{
    AgentReference, BoundedText, EvidenceKind, EvidencePage, EvidenceReference,
    ExecutionHistoryRecord, ExecutionHistorySource, ExecutionQueryBudget, ExecutionQueryError,
    ExecutionQueryLimits, ExecutionQueryResult, ExecutionQueryService, ExecutionSelector,
    HistoryCursor, HistoryPage, HistoryReadRequest, NoExecutionHistory, OutputProjection,
    OutputsPage, RunDigest, RunProjection, StepCounts, StepProjection, StepsPage,
};
pub use runtime::{OrchestraRuntime, RunError, RunOutcome, repository_revision};
pub use skills::{SkillArtifact, SkillError, SkillManifest, SkillManifestEntry};
pub use state::{PromotionStatus, RunCheckpoint, RunStatus, StepCheckpoint, StepStatus};
pub use validate::{ValidationError, validate_plan};
