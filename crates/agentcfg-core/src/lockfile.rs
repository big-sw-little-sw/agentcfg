//! Persisted lockfile model for repeatable Skill Source resolutions.

/// Lockfiles loaded for the active Config Layers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExistingLocks {}

/// A planned persisted lockfile write.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockfileChange {}
