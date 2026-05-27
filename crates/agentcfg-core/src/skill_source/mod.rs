//! **Skill Source** discovery, **Skill Selection**, Skill Groups, and alias resolution.
//!
//! Path and git acquisition, hashing, and lockfile integration are implemented in later milestones.

mod path;

pub use path::{DiscoveredSkill, DiscoveryDepth, discover_path_skills};
