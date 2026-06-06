//! Skill-specific planning contracts for Preview, Apply, Prune, and Status.

/// Planned Skill installation mutations and blockers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillApplyPlan {}

/// Read-only Skill installation findings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillPreviewReport {}

/// Planned stale Skill removals and skips.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillPrunePlan {}

/// Skill installation consistency findings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillStatusReport {}
