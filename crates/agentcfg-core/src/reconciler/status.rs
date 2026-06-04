//! Status report shaping for managed install consistency.

use crate::{
    AgentcfgResult,
    state::{current::CurrentState, locked::LockedDesiredState},
};

/// Reconciler inputs for Status reporting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusInput {
    pub locked_desired: LockedDesiredState,
    pub current: CurrentState,
}

/// Structured install-state consistency findings for later rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusReport {}

pub(crate) fn reconcile(_input: StatusInput) -> AgentcfgResult<StatusReport> {
    unimplemented!("status reconciliation is not implemented yet")
}
