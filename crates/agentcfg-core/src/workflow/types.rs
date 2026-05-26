use std::path::PathBuf;

use crate::scope::{ConfigLayer, InstallScope};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceResolutionPolicy {
    UseLocked,
    RefreshSources,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IoErrorSummary {
    pub kind: std::io::ErrorKind,
    pub message: String,
}

impl From<std::io::Error> for IoErrorSummary {
    fn from(source: std::io::Error) -> Self {
        Self {
            kind: source.kind(),
            message: source.to_string(),
        }
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitResult {
    pub config_file: PathBuf,
    pub warnings: Vec<InitWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InitWarning {
    ExistingTargetArtifact(ExistingTargetArtifact),
    TargetReadFailure(TargetReadFailure),
    ProjectRootDiscoveryFailed(ProjectRootDiscoveryFailed),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ExistingTargetArtifact {
    pub clients: Vec<&'static str>,
    pub install_scope: InstallScope,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct TargetReadFailure {
    pub clients: Vec<&'static str>,
    pub install_scope: InstallScope,
    pub path: PathBuf,
    pub error: IoErrorSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ProjectRootDiscoveryFailed {
    pub start_dir: PathBuf,
    pub error: IoErrorSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PlanRequest {
    pub install_scope: InstallScope,
    pub source_resolution: SourceResolutionPolicy,
    /// When non-empty, narrows the command to these configured client ids.
    pub clients: Vec<String>,
}

impl PlanRequest {
    pub fn new(install_scope: InstallScope, source_resolution: SourceResolutionPolicy) -> Self {
        Self {
            install_scope,
            source_resolution,
            clients: Vec::new(),
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
    pub clients: Vec<String>,
}

impl SyncRequest {
    pub fn new(install_scope: InstallScope, source_resolution: SourceResolutionPolicy) -> Self {
        Self {
            install_scope,
            source_resolution,
            clients: Vec::new(),
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
    pub clients: Vec<String>,
}

impl PruneRequest {
    pub fn new(install_scope: InstallScope) -> Self {
        Self {
            install_scope,
            clients: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusRequest {
    pub install_scope: InstallScope,
    pub clients: Vec<String>,
}

impl StatusRequest {
    pub fn new(install_scope: InstallScope) -> Self {
        Self {
            install_scope,
            clients: Vec::new(),
        }
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
