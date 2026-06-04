//! Preview report shaping for planned install changes.

use crate::{
    AgentcfgResult,
    state::{current::CurrentState, locked::ProposedLockedDesiredState},
};

/// Reconciler inputs for read-only Preview reporting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewInput {
    pub proposed_locked: ProposedLockedDesiredState,
    pub current: CurrentState,
}

/// Structured Preview findings for later terminal rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreviewReport {}

pub(crate) fn reconcile(_input: PreviewInput) -> AgentcfgResult<PreviewReport> {
    unimplemented!("preview reconciliation is not implemented yet")
}
