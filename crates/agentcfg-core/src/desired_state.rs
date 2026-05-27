//! **Desired State**, **Locked Desired State**, **Lockfile**, **Manifest**, and
//! **Configured Item** vocabulary.
//!
//! Preview and apply orchestrate resolution of **Locked Desired State** into
//! **Managed State** and **Client Discovery Locations**. V1 stubs do not yet
//! materialize those outcomes.

/// One kind of agent-facing thing managed by `agentcfg`.
///
/// **Configured Item**: V1 has one Configured Item kind — [`Skill`](ConfiguredItemKind::Skill).
/// Skill-specific code stays skill-specific until another kind exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfiguredItemKind {
    Skill,
}
