//! Mutation boundary for Apply and Prune filesystem changes.

pub mod apply;
pub mod prune;

pub use apply::ApplyExecutionResult;
pub use prune::PruneExecutionResult;

use crate::{
    AgentcfgResult,
    lockfile::LockfileChange,
    reconciler::{ApplyPlan, PrunePlan},
};

pub fn apply(
    lockfile_changes: Vec<LockfileChange>,
    plan: ApplyPlan,
) -> AgentcfgResult<ApplyExecutionResult> {
    apply::execute(lockfile_changes, plan)
}

pub fn prune(plan: PrunePlan) -> AgentcfgResult<PruneExecutionResult> {
    prune::execute(plan)
}
