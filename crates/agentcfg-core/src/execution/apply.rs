//! Apply execution ordering and private preflight checks.

use crate::{AgentcfgResult, lockfile::LockfileChanges, planning::ApplyPlan};

/// Result of an attempted Apply execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyResult {
    pub skills: super::skills::SkillApplyResult,
}

pub(crate) fn execute(
    _lockfile_changes: LockfileChanges,
    _plan: ApplyPlan,
) -> AgentcfgResult<ApplyResult> {
    unimplemented!("apply execution is not implemented yet")
}
