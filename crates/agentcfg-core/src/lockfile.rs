//! Persisted lockfile model for repeatable Skill Source resolutions.

/// Lockfiles loaded for the active Config Layers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lockfiles {}

/// Planned persisted lockfile writes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockfileChanges {}
