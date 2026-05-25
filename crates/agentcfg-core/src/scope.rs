//! Shared selectors for config layers and project-vs-user installation scope.
//!
//! Keep this module limited to stable selector vocabulary. Behavior-specific
//! policy types belong in the module that owns that behavior.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLayer {
    SharedProject,
    PersonalProject,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallScope {
    Project,
    User,
}
