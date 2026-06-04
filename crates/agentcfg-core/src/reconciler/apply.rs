//! Apply plan shaping for install writes and blockers.

use crate::{
    AgentcfgResult,
    state::{current::CurrentState, locked::ProposedLockedDesiredState},
};

/// Reconciler inputs for Apply planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyInput {
    pub proposed_locked: ProposedLockedDesiredState,
    pub current: CurrentState,
}

/// Structured Apply mutations and blockers for execution and rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyPlan {}

pub(crate) fn reconcile(_input: ApplyInput) -> AgentcfgResult<ApplyPlan> {
    unimplemented!("apply reconciliation is not implemented yet")
}
