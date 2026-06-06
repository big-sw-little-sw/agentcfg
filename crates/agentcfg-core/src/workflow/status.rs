//! Reports consistency between LockfilePinnedConfig and ObservedInstallation.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, planning::StatusReport,
    resolution::LockfileConfigCheck,
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
    pub lockfile_check: LockfileConfigCheck,
    pub status: StatusReport,
}

pub fn run(_request: StatusRequest) -> AgentcfgResult<StatusCommandReport> {
    unimplemented!("status workflow is not implemented yet")
}
