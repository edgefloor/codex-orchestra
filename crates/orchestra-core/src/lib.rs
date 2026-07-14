//! Orchestra's host-independent workflow compiler and runtime.
//!
//! Workflow TypeScript is parsed as a restricted data language. It is never
//! evaluated by a JavaScript engine and cannot execute user code.

mod compiler;
mod context;
mod host;
mod plan;
mod runtime;
mod state;
mod validate;

pub use compiler::{CompileError, compile_workflow};
pub use context::{ContextBundle, ContextError, materialize_context};
pub use host::{AgentHandle, AgentOutcome, AgentStatus, CommandOutcome, NativeHost, SpawnRequest};
pub use plan::*;
pub use runtime::{OrchestraRuntime, RunError, RunOutcome};
pub use state::{PromotionStatus, RunCheckpoint, RunStatus, StepCheckpoint, StepStatus};
pub use validate::{ValidationError, validate_plan};
