//! Prune execution ordering and private safety checks.

use crate::{AgentcfgResult, planning::PrunePlan};

/// Result of an attempted Prune execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneResult {
    pub skills: super::skills::SkillPruneResult,
}

pub(crate) fn execute(_plan: PrunePlan) -> AgentcfgResult<PruneResult> {
    unimplemented!("prune execution is not implemented yet")
}
