//! Builds and executes Apply changes for locked desired install state.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, executor::ApplyExecutionResult,
    lockfile::LockfileChange, reconciler::ApplyPlan,
};

/// Command request for Apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyRequest {
    pub install_level: InstallLevel,
    pub refresh_sources: bool,
    pub clients: ClientSelection,
}

/// Complete Apply command plan for execution and later rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyCommandPlan {
    pub lockfile_changes: Vec<LockfileChange>,
    pub reconciler_plan: ApplyPlan,
}

/// Complete Apply command result for later rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyCommandResult {
    pub plan: ApplyCommandPlan,
    pub outcome: super::CommandExecutionOutcome<ApplyExecutionResult>,
}

pub fn run(_request: ApplyRequest) -> AgentcfgResult<ApplyCommandResult> {
    unimplemented!("apply workflow is not implemented yet")
}
