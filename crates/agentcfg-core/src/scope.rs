//! Shared selectors for Config Layers and Install Level.
//!
//! Keep this module limited to stable selector vocabulary. Behavior-specific
//! policy types belong in the module that owns that behavior.

/// A **Config Layer**: Shared Project Config, User Project Config, or User Config.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLayer {
    SharedProject,
    UserProject,
    User,
}

impl ConfigLayer {
    /// Returns the **Persisted Scope Value** for this layer (`shared-project`, etc.).
    ///
    /// This is not the Install Level.
    pub fn persisted_scope(self) -> &'static str {
        match self {
            ConfigLayer::SharedProject => "shared-project",
            ConfigLayer::UserProject => "user-project",
            ConfigLayer::User => "user",
        }
    }

    /// Parses a **Persisted Scope Value** into a Config Layer.
    pub fn from_persisted_scope(value: &str) -> Option<Self> {
        match value {
            "shared-project" => Some(ConfigLayer::SharedProject),
            "user-project" => Some(ConfigLayer::UserProject),
            "user" => Some(ConfigLayer::User),
            _ => None,
        }
    }
}

/// **Install Level**: Project Level vs User Level for preview, apply, prune, and status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallLevel {
    /// Project Level — Shared Project Config and User Project Config are active.
    Project,
    /// User Level — User Config is the only active Config Layer.
    User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_layers_map_to_persisted_scope_strings() {
        assert_eq!(
            ConfigLayer::SharedProject.persisted_scope(),
            "shared-project"
        );
        assert_eq!(ConfigLayer::UserProject.persisted_scope(), "user-project");
        assert_eq!(ConfigLayer::User.persisted_scope(), "user");
    }

    #[test]
    fn persisted_scope_strings_map_to_config_layers() {
        assert_eq!(
            ConfigLayer::from_persisted_scope("shared-project"),
            Some(ConfigLayer::SharedProject)
        );
        assert_eq!(
            ConfigLayer::from_persisted_scope("user-project"),
            Some(ConfigLayer::UserProject)
        );
        assert_eq!(
            ConfigLayer::from_persisted_scope("user"),
            Some(ConfigLayer::User)
        );
        assert_eq!(ConfigLayer::from_persisted_scope("project"), None);
        assert_eq!(ConfigLayer::from_persisted_scope("sharedProject"), None);
        assert_eq!(ConfigLayer::from_persisted_scope("userProject"), None);
    }
}
