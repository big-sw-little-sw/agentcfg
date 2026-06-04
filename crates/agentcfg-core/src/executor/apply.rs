//! Apply execution ordering and private preflight checks.

use crate::{AgentcfgResult, lockfile::LockfileChange, reconciler::ApplyPlan};

/// Result of an attempted Apply execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyExecutionResult {}

pub(crate) fn execute(
    _lockfile_changes: Vec<LockfileChange>,
    _plan: ApplyPlan,
) -> AgentcfgResult<ApplyExecutionResult> {
    unimplemented!("apply execution is not implemented yet")
}
