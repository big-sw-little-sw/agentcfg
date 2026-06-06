//! Apply plan shaping for installation writes and blockers.

use crate::{AgentcfgResult, installation::ObservedInstallation, resolution::PlannedPinnedConfig};

/// Planning inputs for Apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyInput {
    pub planned_pinned: PlannedPinnedConfig,
    pub observed_installation: ObservedInstallation,
}

/// Structured Apply mutations and blockers for execution and rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyPlan {
    pub skills: super::skills::SkillApplyPlan,
}

pub(crate) fn plan(_input: ApplyInput) -> AgentcfgResult<ApplyPlan> {
    unimplemented!("apply planning is not implemented yet")
}
