//! Removes stale Managed State when Manifest evidence proves it is safe.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, execution::PruneResult, planning::PrunePlan,
    resolution::LockfileConfigCheck,
};

/// Command request for Prune.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneRequest {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
}

/// Complete Prune command plan for execution and later rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneCommandPlan {
    pub lockfile_check: LockfileConfigCheck,
    pub prune_plan: PrunePlan,
}

/// Complete Prune command result for later rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneCommandResult {
    pub plan: PruneCommandPlan,
    pub outcome: super::CommandExecutionOutcome<PruneResult>,
}

pub fn run(_request: PruneRequest) -> AgentcfgResult<PruneCommandResult> {
    unimplemented!("prune workflow is not implemented yet")
}
