//! Prune plan shaping for stale requirements and artifacts.

use crate::{AgentcfgResult, installation::ObservedInstallation, resolution::LockfilePinnedConfig};

/// Planning inputs for Prune.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneInput {
    pub lockfile_pinned: LockfilePinnedConfig,
    pub observed_installation: ObservedInstallation,
}

/// Structured stale removals and skips for execution and rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PrunePlan {
    pub skills: super::skills::SkillPrunePlan,
}

pub(crate) fn plan(_input: PruneInput) -> AgentcfgResult<PrunePlan> {
    unimplemented!("prune planning is not implemented yet")
}
