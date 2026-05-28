//! **Skill Source** discovery, **Skill Selection**, Skill Groups, and alias resolution.
//!
//! Path and git acquisition, hashing, and lockfile integration are implemented in later milestones.

mod groups;
mod path;
mod selection;

pub use path::{
    DiscoveredSkill, DiscoveredSkillsInPathSource, DiscoveryDepth, discover_skills_in_source,
};
pub use selection::{
    EmptyDiscovery, SelectedSkill, SkillSelection, SkillSelectionInput, SkillSelectionWarning,
    resolve_skill_selection,
};
