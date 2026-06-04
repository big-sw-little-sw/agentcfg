//! Current State observations normalized from local evidence.

/// Observable install-state facts normalized for reconciliation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrentState {
    pub skills: CurrentSkillResources,
}

/// Current Skill artifact facts from Managed State and Client Discovery Locations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrentSkillResources {}
