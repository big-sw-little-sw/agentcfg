//! User workflow entrypoints for the CLI and future frontends.
//!
//! These functions are orchestration boundaries, not the lower-level
//! config, planning, apply, status, or diagnostic APIs. Those focused APIs
//! should be added when they are needed by implemented behavior.

use crate::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLayer {
    SharedProject,
    PersonalProject,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallScope {
    Project,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceResolutionPolicy {
    UseLocked,
    RefreshSources,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitRequest {
    pub config_layer: ConfigLayer,
}

impl InitRequest {
    pub fn new(config_layer: ConfigLayer) -> Self {
        Self { config_layer }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PlanRequest {
    pub install_scope: InstallScope,
    pub source_resolution: SourceResolutionPolicy,
}

impl PlanRequest {
    pub fn new(install_scope: InstallScope, source_resolution: SourceResolutionPolicy) -> Self {
        Self {
            install_scope,
            source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PlanResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct SyncRequest {
    pub install_scope: InstallScope,
    pub source_resolution: SourceResolutionPolicy,
}

impl SyncRequest {
    pub fn new(install_scope: InstallScope, source_resolution: SourceResolutionPolicy) -> Self {
        Self {
            install_scope,
            source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct SyncResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneRequest {
    pub install_scope: InstallScope,
}

impl PruneRequest {
    pub fn new(install_scope: InstallScope) -> Self {
        Self { install_scope }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusRequest {
    pub install_scope: InstallScope,
}

impl StatusRequest {
    pub fn new(install_scope: InstallScope) -> Self {
        Self { install_scope }
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
