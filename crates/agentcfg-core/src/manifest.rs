//! Manifest model for owned Installed Artifacts and Discovery Requirements.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    Client, ClientDiscoveryLocation, ConfigLayerKind, ConfigSourceId, DiscoveryName, InstallLevel,
    SourceSkillName, TreeDigest,
};

/// Persisted Manifest document.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestFile {
    pub installed_artifacts: Vec<InstalledArtifactRecord>,
    pub discovery_requirements: Vec<DiscoveryRequirementRecord>,
}

/// Manifest evidence loaded from Managed State.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestSnapshot {
    pub file: ManifestFile,
}

/// Physical discovery identity for one Installed Artifact.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ArtifactKey {
    pub install_level: InstallLevel,
    pub client_discovery_location: ClientDiscoveryLocation,
    pub discovery_name: DiscoveryName,
}

/// Identity for the Config Layer and Client that require an artifact.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RequirementKey {
    pub config_layer: ConfigLayerKind,
    pub install_level: InstallLevel,
    pub client: Client,
    pub client_discovery_location: ClientDiscoveryLocation,
    pub discovery_name: DiscoveryName,
}

/// Persisted Manifest record for an Installed Artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InstalledArtifactRecord {
    pub key: ArtifactKey,
    pub discovery_path: PathBuf,
    pub target: PathBuf,
    pub digest: TreeDigest,
}

/// Persisted Manifest record for one Discovery Requirement.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoveryRequirementRecord {
    pub key: RequirementKey,
    pub artifact_key: ArtifactKey,
    pub required_digest: TreeDigest,
    pub pinned_skill_ref: PinnedSkillRef,
}

/// Skill provenance retained for reporting and recovery diagnostics.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PinnedSkillRef {
    pub config_source_id: ConfigSourceId,
    pub source_skill_name: SourceSkillName,
    pub discovery_name: DiscoveryName,
}
