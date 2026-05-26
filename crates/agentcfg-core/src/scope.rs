//! Shared selectors for config layers and project-vs-user installation scope.
//!
//! Keep this module limited to stable selector vocabulary. Behavior-specific
//! policy types belong in the module that owns that behavior.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLayer {
    SharedProject,
    UserProject,
    User,
}

impl ConfigLayer {
    pub fn persisted_scope(self) -> &'static str {
        match self {
            ConfigLayer::SharedProject => "shared-project",
            ConfigLayer::UserProject => "user-project",
            ConfigLayer::User => "user",
        }
    }

    pub fn from_persisted_scope(value: &str) -> Option<Self> {
        match value {
            "shared-project" => Some(ConfigLayer::SharedProject),
            "user-project" => Some(ConfigLayer::UserProject),
            "user" => Some(ConfigLayer::User),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallScope {
    Project,
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
