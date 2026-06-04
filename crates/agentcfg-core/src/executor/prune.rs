//! Prune execution ordering and private safety checks.

use crate::{AgentcfgResult, reconciler::PrunePlan};

/// Result of an attempted Prune execution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneExecutionResult {}

pub(crate) fn execute(_plan: PrunePlan) -> AgentcfgResult<PruneExecutionResult> {
    unimplemented!("prune execution is not implemented yet")
}
