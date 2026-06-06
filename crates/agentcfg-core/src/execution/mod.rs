//! Mutation boundary for Apply and Prune filesystem changes.

pub mod apply;
pub mod prune;
pub mod skills;

pub use apply::ApplyResult;
pub use prune::PruneResult;

use crate::{
    AgentcfgResult,
    lockfile::LockfileChanges,
    planning::{ApplyPlan, PrunePlan},
};

pub fn apply(lockfile_changes: LockfileChanges, plan: ApplyPlan) -> AgentcfgResult<ApplyResult> {
    apply::execute(lockfile_changes, plan)
}

pub fn prune(plan: PrunePlan) -> AgentcfgResult<PruneResult> {
    prune::execute(plan)
}
