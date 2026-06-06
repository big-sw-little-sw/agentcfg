//! Status report shaping for managed install consistency.

use crate::{AgentcfgResult, installation::ObservedInstallation, resolution::LockfilePinnedConfig};

/// Planning inputs for Status reporting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusInput {
    pub lockfile_pinned: LockfilePinnedConfig,
    pub observed_installation: ObservedInstallation,
}

/// Structured install-state consistency findings for later rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusReport {
    pub skills: super::skills::SkillStatusReport,
}

pub(crate) fn plan(_input: StatusInput) -> AgentcfgResult<StatusReport> {
    unimplemented!("status planning is not implemented yet")
}
