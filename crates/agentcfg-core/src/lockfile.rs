//! Persisted lockfile model for repeatable Skill Source resolutions.

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

/// Persisted Lockfile document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LockfileFile {
    pub version: u32,
    #[serde(default)]
    pub skills: SkillLockfileSection,
}

/// Skill lockfile records are filled in by later source-resolution work.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SkillLockfileSection {}

/// Lockfiles loaded for the active Config Layers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lockfiles {
    pub files: Vec<LockfileFile>,
}

/// Planned persisted lockfile writes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockfileChanges {}
