//! Shared workflow result types returned by presentation-agnostic core workflows.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowResult<T> {
    pub workflow: WorkflowName,
    pub status: WorkflowStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub blockers: Vec<Diagnostic>,
    /// Result-level follow-up steps that are useful even when no Diagnostic caused them.
    pub suggested_actions: Vec<SuggestedAction>,
    /// Events emitted during workflow execution and retained for non-streaming presentations.
    pub progress_events: Vec<ProgressEvent>,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowName {
    ConfigShow,
    ClientsShow,
    ClientsSet,
    ClientsAdd,
    ClientsRemove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStatus {
    Success,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub context: Vec<(String, String)>,
    /// Follow-up steps tied to this specific Diagnostic.
    pub suggested_actions: Vec<SuggestedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuggestedAction {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressEvent {
    pub phase: String,
    pub message: String,
}
