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
mod stubs;
mod types;

pub use crate::layer_level::{ConfigLayer, InstallLevel};
pub use types::{
    ApplyRequest, ApplyResult, ClientDiscoveryLocationReadFailure, DoctorRequest, DoctorResult,
    InitRequest, InitResult, InitWarning, PreviewRequest, PreviewResult, PruneRequest, PruneResult,
    SkillSourceResolutionPolicy, StatusRequest, StatusResult, UnmanagedArtifact,
};

pub use init::init;
pub use stubs::{apply, doctor, preview, prune, status};
