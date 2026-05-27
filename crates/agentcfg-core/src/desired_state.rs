//! **Desired State** and **Locked Desired State** for **Configured Items**.
//!
//! Active Config Layers express Desired State; lockfiles record Locked Desired State
//! after Skill Source resolutions are fixed. Implementation of resolution and preview
//! operation generation lives in later milestones.

use crate::layer_level::ConfigLayer;

/// One kind of agent-facing thing managed by `agentcfg`.
///
/// V1 uses skill-specific modules; this enum documents the shared term without
/// introducing generic Configured Item manager traits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfiguredItemKind {
    Skill,
}

/// Skill Source id namespaced by Config Layer for lockfiles and collision detection.
///
/// Persisted form: `{persisted_scope_value}:{skill_source_id}` (see `persisted_key`).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NamespacedSkillSourceId {
    layer: ConfigLayer,
    skill_source_id: String,
}

impl NamespacedSkillSourceId {
    pub fn new(layer: ConfigLayer, skill_source_id: impl Into<String>) -> Self {
        Self {
            layer,
            skill_source_id: skill_source_id.into(),
        }
    }

    pub fn layer(&self) -> ConfigLayer {
        self.layer
    }

    pub fn skill_source_id(&self) -> &str {
        &self.skill_source_id
    }

    pub fn persisted_key(&self) -> String {
        format!(
            "{}:{}",
            self.layer.persisted_scope_value(),
            self.skill_source_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespaced_skill_source_id_persisted_key_includes_layer() {
        let id = NamespacedSkillSourceId::new(ConfigLayer::SharedProject, "local");
        assert_eq!(id.persisted_key(), "shared-project:local");
    }
}
