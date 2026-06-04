//! Resolves Desired State against lockfiles into locked install intent.

use crate::{
    AgentcfgResult,
    lockfile::{ExistingLocks, LockfileChange},
    state::{
        desired::DesiredState,
        locked::{LockedDesiredState, ProposedLockedDesiredState},
    },
};

/// What Preview or Apply will use after matching Desired State with lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockPlan {
    pub proposed_locked: ProposedLockedDesiredState,
    pub lockfile_changes: Vec<LockfileChange>,
    pub diagnostics: Vec<LockPlanningDiagnostic>,
    pub blocking_diagnostics: Vec<BlockingDesiredStateDiagnostic>,
}

/// The locked state already available for commands that do not refresh Skill Sources.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExistingLockState {
    pub locked_desired: LockedDesiredState,
    pub diagnostics: Vec<LockPlanningDiagnostic>,
    pub blocking_diagnostics: Vec<BlockingDesiredStateDiagnostic>,
}

/// A warning or mismatch found while checking Skill Sources or lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockPlanningDiagnostic {}

/// A Desired State problem that must be fixed before planning can continue.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockingDesiredStateDiagnostic {}

pub fn build_preview_lock_plan(
    _desired: DesiredState,
    _existing_locks: ExistingLocks,
    _refresh_sources: bool,
) -> AgentcfgResult<LockPlan> {
    unimplemented!("preview lock planning is not implemented yet")
}

pub fn build_apply_lock_plan(
    _desired: DesiredState,
    _existing_locks: ExistingLocks,
    _refresh_sources: bool,
) -> AgentcfgResult<LockPlan> {
    unimplemented!("apply lock planning is not implemented yet")
}

pub fn build_status_lock_state(
    _desired: DesiredState,
    _existing_locks: ExistingLocks,
) -> AgentcfgResult<ExistingLockState> {
    unimplemented!("status lock loading is not implemented yet")
}

pub fn build_prune_lock_state(
    _desired: DesiredState,
    _existing_locks: ExistingLocks,
) -> AgentcfgResult<ExistingLockState> {
    unimplemented!("prune lock loading is not implemented yet")
}
