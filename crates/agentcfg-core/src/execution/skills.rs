//! Skill-specific execution results for Apply and Prune.

/// Result of Skill installation writes attempted during Apply.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillApplyResult {}

/// Result of Skill stale-state removals attempted during Prune.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillPruneResult {}
