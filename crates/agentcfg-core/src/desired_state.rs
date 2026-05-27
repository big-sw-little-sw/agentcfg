//! Glossary anchors for **Desired State**, **Locked Desired State**, **Lockfile**,
//! **Manifest**, and **Configured Item** vocabulary.
//!
//! Preview and apply orchestrate resolution of **Locked Desired State** into
//! **Managed State** and **Client Discovery Locations**. V1 stubs do not yet
//! materialize those outcomes; this module documents the terms only.

/// One kind of agent-facing thing managed by `agentcfg`.
///
/// **Configured Item**: V1 has one Configured Item kind — [`Skill`](ConfiguredItemKind::Skill).
/// Skill-specific code stays skill-specific until another kind exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfiguredItemKind {
    Skill,
}

#[cfg(test)]
mod glossary_tests {
    use super::ConfiguredItemKind;

    const DESIRED_STATE_GLOSSARY: &str = concat!(
        "Desired State is the outcome active Config Layers ask agentcfg to make true, ",
        "before source resolutions are fixed. Locked Desired State fixes those resolutions ",
        "so Apply can repeat the same result. Preview shows changes; apply materializes ",
        "Locked Desired State."
    );

    const LOCKFILE_GLOSSARY: &str = concat!(
        "Lockfile records Locked Desired State for Configured Items that need repeatable ",
        "source resolution, such as skills from path or git Skill Sources."
    );

    const MANIFEST_GLOSSARY: &str = concat!(
        "Manifest is agentcfg-owned state that records Installed Artifacts and the ",
        "Discovery Requirements that keep them present at Client Discovery Locations."
    );

    #[test]
    fn desired_state_glossary_documents_outcome_before_resolution() {
        assert!(
            DESIRED_STATE_GLOSSARY.contains("Desired State"),
            "glossary distinguishes Desired State from Locked Desired State"
        );
        assert!(
            DESIRED_STATE_GLOSSARY.contains("Locked Desired State"),
            "glossary documents resolution before apply"
        );
        assert_eq!(ConfiguredItemKind::Skill, ConfiguredItemKind::Skill);
    }

    #[test]
    fn lockfile_glossary_records_locked_desired_state_for_configured_items() {
        assert!(
            LOCKFILE_GLOSSARY.contains("Locked Desired State"),
            "lockfiles record locked outcomes for configured items"
        );
        assert!(
            LOCKFILE_GLOSSARY.contains("Configured Items"),
            "lockfiles are per configured item kind needing repeatable resolution"
        );
    }

    #[test]
    fn manifest_glossary_records_installed_artifacts_and_discovery_requirements() {
        assert!(
            MANIFEST_GLOSSARY.contains("Installed Artifacts"),
            "manifest owns installed artifact records"
        );
        assert!(
            MANIFEST_GLOSSARY.contains("Discovery Requirements"),
            "manifest tracks requirements that keep artifacts present"
        );
    }
}
