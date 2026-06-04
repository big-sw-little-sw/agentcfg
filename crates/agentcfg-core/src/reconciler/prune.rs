//! Prune plan shaping for stale requirements and artifacts.

use crate::{
    AgentcfgResult,
    state::{current::CurrentState, locked::LockedDesiredState},
};

/// Reconciler inputs for Prune planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneInput {
    pub locked_desired: LockedDesiredState,
    pub current: CurrentState,
}

/// Structured stale removals and skips for execution and rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PrunePlan {}

pub(crate) fn reconcile(_input: PruneInput) -> AgentcfgResult<PrunePlan> {
    unimplemented!("prune reconciliation is not implemented yet")
}
