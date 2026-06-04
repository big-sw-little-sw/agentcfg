//! Locked Desired State and proposed locked state for repeatable installs.

/// Locked install intent loaded from existing lockfiles.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockedDesiredState {
    pub skills: LockedDesiredSkillResources,
}

/// Locked install intent proposed by preview or apply planning.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProposedLockedDesiredState {
    pub skills: LockedDesiredSkillResources,
}

/// Locked Skill resources after Skill Source resolution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockedDesiredSkillResources {}
