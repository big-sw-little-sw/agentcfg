//! `agentcfg-core` — config, paths, Skill Sources, desired state, and workflows.
//!
//! See `AGENTS.md` and README § Concepts → code for a domain-term → module map.

pub mod config;
pub mod config_paths;
pub mod desired_state;
mod discovery_registry;
mod error;
pub mod install_health;
pub mod layer_level;
pub mod lockfile;
pub mod manifest;
pub mod skill_source;
pub mod workflow;

pub use desired_state::{ConfiguredItemKind, NamespacedSkillSourceId};
pub use error::{
    ConfigError, Error, InitError, InvalidSkillGroupDefinition, InvalidSkillGroupDefinitionReason,
    MissingIncludedSkill, MissingSkillGroup, MissingSkillGroupCause, MissingSkillGroupMember,
    PathEnvironmentError, Result, SkillSelectionError, SkillSourceError,
    SkillSourceMetadataParseError, UnsupportedError,
};
