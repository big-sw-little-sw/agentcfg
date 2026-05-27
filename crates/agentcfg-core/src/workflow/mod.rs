//! User workflow entrypoints for the CLI and future frontends.
//!
//! Preview and apply orchestrate resolution of **Locked Desired State** into
//! **Managed State** and **Client Discovery Locations**.
//!
//! **Status** reports managed install-state consistency for an Install Level.
//! **Doctor** reports environment and configuration readiness; it does not
//! replace **Status** for install-state reporting.
//!
//! **Prune** removes **Stale Discovery Requirements** and **Stale Installed
//! Artifacts** from Managed State when removal is safe.
//!
//! These functions are orchestration boundaries, not the lower-level
//! config, preview operation, apply, status, or diagnostic APIs.

mod context;
mod init;
mod types;

use crate::{Result, UnsupportedError};

pub use crate::layer_level::{ConfigLayer, InstallLevel};
pub use types::{
    ApplyRequest, ApplyResult, ClientDiscoveryLocationReadFailure, DoctorRequest, DoctorResult,
    InitRequest, InitResult, InitWarning, PreviewRequest, PreviewResult, PruneRequest, PruneResult,
    SkillSourceResolutionPolicy, StatusRequest, StatusResult, UnmanagedArtifact,
};

pub use init::init;

pub fn preview(_request: PreviewRequest) -> Result<PreviewResult> {
    workflow_not_implemented()
}

pub fn apply(_request: ApplyRequest) -> Result<ApplyResult> {
    workflow_not_implemented()
}

pub fn prune(_request: PruneRequest) -> Result<PruneResult> {
    workflow_not_implemented()
}

pub fn status(_request: StatusRequest) -> Result<StatusResult> {
    workflow_not_implemented()
}

pub fn doctor(_request: DoctorRequest) -> Result<DoctorResult> {
    workflow_not_implemented()
}

fn workflow_not_implemented<T>() -> Result<T> {
    Err(UnsupportedError::Feature {
        feature: "workflow not implemented",
    }
    .into())
}
