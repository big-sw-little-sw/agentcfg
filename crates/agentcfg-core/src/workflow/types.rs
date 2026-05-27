use std::path::PathBuf;

use crate::layer_level::{ConfigLayer, InstallLevel};

/// How preview/apply move from **Desired State** to **Locked Desired State** via lockfiles.
///
/// Active Config Layers express **Desired State**; lockfiles record **Locked Desired State**
/// for Configured Items that need repeatable Skill Source resolution.
///
/// - [`UseLocked`]: use **Locked Desired State** from the active lockfile without Source Refresh.
/// - [`RefreshSources`]: perform **Source Refresh** to refresh Skill Source resolutions before
///   producing updated **Locked Desired State** and materializing **Managed Skill Content**.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillSourceResolutionPolicy {
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitResult {
    pub config_file: PathBuf,
    pub warnings: Vec<InitWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InitWarning {
    UnmanagedArtifact(UnmanagedArtifact),
    ClientDiscoveryLocationReadFailure(ClientDiscoveryLocationReadFailure),
    /// User-level Client Discovery Locations were not scanned (for example `HOME` unset while XDG overrides are set).
    UserClientDiscoveryLocationsNotScanned(UserClientDiscoveryLocationsNotScanned),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UserClientDiscoveryLocationsNotScanned {
    pub message: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UnmanagedArtifact {
    pub clients: Vec<&'static str>,
    pub install_level: InstallLevel,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ClientDiscoveryLocationReadFailure {
    pub clients: Vec<&'static str>,
    pub install_level: InstallLevel,
    pub path: PathBuf,
    pub error: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PreviewRequest {
    pub install_level: InstallLevel,
    pub skill_source_resolution: SkillSourceResolutionPolicy,
}

impl PreviewRequest {
    pub fn new(
        install_level: InstallLevel,
        skill_source_resolution: SkillSourceResolutionPolicy,
    ) -> Self {
        Self {
            install_level,
            skill_source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PreviewResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApplyRequest {
    pub install_level: InstallLevel,
    pub skill_source_resolution: SkillSourceResolutionPolicy,
}

impl ApplyRequest {
    pub fn new(
        install_level: InstallLevel,
        skill_source_resolution: SkillSourceResolutionPolicy,
    ) -> Self {
        Self {
            install_level,
            skill_source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApplyResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneRequest {
    pub install_level: InstallLevel,
}

impl PruneRequest {
    pub fn new(install_level: InstallLevel) -> Self {
        Self { install_level }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusRequest {
    pub install_level: InstallLevel,
}

impl StatusRequest {
    pub fn new(install_level: InstallLevel) -> Self {
        Self { install_level }
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
