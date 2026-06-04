//! Desired State before Skill Source resolutions are fixed.

/// Active configured intent before Skill Source resolution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesiredState {
    pub skills: DesiredSkillResources,
}

/// Desired Skill resources selected by active Config Layers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesiredSkillResources {}
