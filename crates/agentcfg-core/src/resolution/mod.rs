//! Resolves Config Requests and Lockfiles into pinned install intent.

pub mod skills;

use crate::{
    AgentcfgResult,
    config::ConfigRequest,
    lockfile::{LockfileChanges, Lockfiles},
};

/// Pinned configuration with repeatable Skill Source resolutions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PinnedConfig {
    pub skills: skills::PinnedSkillConfig,
}

/// Pinned configuration loaded from existing Lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockfilePinnedConfig(pub PinnedConfig);

/// Pinned configuration planned for Preview or Apply.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlannedPinnedConfig(pub PinnedConfig);

/// Resolution outcome for commands that may create or update Lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResolutionPlan {
    pub planned_pinned: PlannedPinnedConfig,
    pub lockfile_changes: LockfileChanges,
    pub diagnostics: Vec<ResolutionDiagnostic>,
    pub blocking_diagnostics: Vec<BlockingConfigRequestDiagnostic>,
}

/// Check of active Config Requests against already-persisted Lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockfileConfigCheck {
    pub lockfile_pinned: LockfilePinnedConfig,
    pub diagnostics: Vec<ResolutionDiagnostic>,
    pub blocking_diagnostics: Vec<BlockingConfigRequestDiagnostic>,
}

/// A non-blocking source-resolution or config/lockfile diagnostic.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResolutionDiagnostic {}

/// A Config Request problem that must stop before installation planning.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockingConfigRequestDiagnostic {}

pub fn build_preview_resolution_plan(
    _request: ConfigRequest,
    _lockfiles: Lockfiles,
    _refresh_sources: bool,
) -> AgentcfgResult<ResolutionPlan> {
    unimplemented!("preview resolution planning is not implemented yet")
}

pub fn build_apply_resolution_plan(
    _request: ConfigRequest,
    _lockfiles: Lockfiles,
    _refresh_sources: bool,
) -> AgentcfgResult<ResolutionPlan> {
    unimplemented!("apply resolution planning is not implemented yet")
}

pub fn check_status_lockfiles(
    _request: ConfigRequest,
    _lockfiles: Lockfiles,
) -> AgentcfgResult<LockfileConfigCheck> {
    unimplemented!("status lockfile checking is not implemented yet")
}

pub fn check_prune_lockfiles(
    _request: ConfigRequest,
    _lockfiles: Lockfiles,
) -> AgentcfgResult<LockfileConfigCheck> {
    unimplemented!("prune lockfile checking is not implemented yet")
}
