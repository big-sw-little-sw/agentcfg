//! User workflow entrypoints for the CLI and future frontends.
//!
//! These functions are orchestration boundaries, not the lower-level
//! config, planning, apply, status, or diagnostic APIs. Those focused APIs
//! should be added when they are needed by implemented behavior.

use crate::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigScope {
    Project,
    UserProject,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetScope {
    Project,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceResolutionMode {
    Locked,
    Upgrade,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitRequest {
    pub scope: ConfigScope,
}

impl InitRequest {
    pub fn new(scope: ConfigScope) -> Self {
        Self { scope }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PlanRequest {
    pub target_scope: TargetScope,
    pub resolution: SourceResolutionMode,
}

impl PlanRequest {
    pub fn new(target_scope: TargetScope, resolution: SourceResolutionMode) -> Self {
        Self {
            target_scope,
            resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PlanResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct SyncRequest {
    pub target_scope: TargetScope,
    pub resolution: SourceResolutionMode,
}

impl SyncRequest {
    pub fn new(target_scope: TargetScope, resolution: SourceResolutionMode) -> Self {
        Self {
            target_scope,
            resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct SyncResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneRequest {
    pub target_scope: TargetScope,
}

impl PruneRequest {
    pub fn new(target_scope: TargetScope) -> Self {
        Self { target_scope }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusRequest {
    pub target_scope: TargetScope,
}

impl StatusRequest {
    pub fn new(target_scope: TargetScope) -> Self {
        Self { target_scope }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusResult {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct DoctorRequest {}

impl DoctorRequest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct DoctorResult {}

pub fn init(_request: InitRequest) -> Result<InitResult> {
    Ok(InitResult {})
}

pub fn plan(_request: PlanRequest) -> Result<PlanResult> {
    Ok(PlanResult {})
}

pub fn sync(_request: SyncRequest) -> Result<SyncResult> {
    Ok(SyncResult {})
}

pub fn prune(_request: PruneRequest) -> Result<PruneResult> {
    Ok(PruneResult {})
}

pub fn status(_request: StatusRequest) -> Result<StatusResult> {
    Ok(StatusResult {})
}

pub fn doctor(_request: DoctorRequest) -> Result<DoctorResult> {
    Ok(DoctorResult {})
}
