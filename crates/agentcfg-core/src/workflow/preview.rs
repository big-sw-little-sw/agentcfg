//! Builds read-only Preview results without mutating Managed State.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, lock_planner::LockPlan,
    reconciler::PreviewReport,
};

/// Command request for Preview.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewRequest {
    pub install_level: InstallLevel,
    pub refresh_sources: bool,
    pub clients: ClientSelection,
}

/// Complete Preview command plan for later rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreviewCommandPlan {
    pub lock_plan: LockPlan,
    pub install_preview: PreviewReport,
}

pub fn run(_request: PreviewRequest) -> AgentcfgResult<PreviewCommandPlan> {
    unimplemented!("preview workflow is not implemented yet")
}
