//! Reports consistency of the current managed install state.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, lock_planner::ExistingLockState,
    reconciler::StatusReport,
};

/// Command request for Status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusRequest {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
}

/// Complete Status report for later terminal rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusCommandReport {
    pub existing_lock_state: ExistingLockState,
    pub install_status: StatusReport,
}

pub fn run(_request: StatusRequest) -> AgentcfgResult<StatusCommandReport> {
    unimplemented!("status workflow is not implemented yet")
}
