//! User workflow entrypoints for the CLI and future frontends.
//!
//! These functions are orchestration boundaries, not the lower-level
//! config, planning, apply, status, or diagnostic APIs.

mod context;
mod init;
mod stubs;
mod types;

pub use crate::scope::{ConfigLayer, InstallScope};
pub use types::{
    DoctorRequest, DoctorResult, ExistingTargetArtifact, InitRequest, InitResult, InitWarning,
    IoErrorSummary, PlanRequest, PlanResult, PruneRequest, PruneResult, ProjectRootDiscoveryFailed,
    SourceResolutionPolicy, StatusRequest, StatusResult, SyncRequest, SyncResult,
    TargetReadFailure,
};

pub fn init(request: InitRequest) -> crate::Result<InitResult> {
    init::init(request)
}

pub use stubs::{doctor, plan, prune, status, sync};
