//! Glossary anchors for install-health workflows and safety diagnostics.
//!
//! **Status** reports whether managed install state is consistent for an Install Level.
//! **Doctor** reports whether the local environment and configuration are ready to work;
//! it does not replace **Status** for install-state consistency.
//!
//! **Prune** removes **Stale Discovery Requirements** and **Stale Installed Artifacts**
//! from managed state only when removal is safe.
//!
//! **Stale Discovery Requirement**: a Discovery Requirement recorded in the Manifest
//! that is no longer present in Desired State.
//!
//! **Unsatisfied Discovery Requirement**: a Discovery Requirement in Desired State
//! without a valid Installed Artifact.
//!
//! **Stale Installed Artifact**: an Installed Artifact recorded in the Manifest with
//! no remaining Discovery Requirements.
//!
//! **Unexpected Symlink Target** and **Broken Symlink** apply only to symlink
//! destinations for Installed Artifacts, not generic client-target language.

#[cfg(test)]
mod glossary_tests {
    const STATUS_GLOSSARY: &str = concat!(
        "Status reports whether the current managed install state is consistent ",
        "for an Install Level. It covers Installed Artifacts, Stale Installed Artifacts, ",
        "Unsatisfied Discovery Requirements, and symlink diagnostics."
    );

    const PRUNE_GLOSSARY: &str = concat!(
        "Prune removes Stale Discovery Requirements and Stale Installed Artifacts ",
        "from managed state only when agentcfg can prove removal is safe."
    );

    const DOCTOR_GLOSSARY: &str = concat!(
        "Doctor reports whether the local environment and configuration are capable ",
        "of working. It checks tooling readiness, not managed install consistency."
    );

    const STALE_DISCOVERY_REQUIREMENT_GLOSSARY: &str = concat!(
        "Stale Discovery Requirement is a Manifest requirement no longer present ",
        "in Desired State."
    );

    const UNSATISFIED_DISCOVERY_REQUIREMENT_GLOSSARY: &str = concat!(
        "Unsatisfied Discovery Requirement is a Desired State requirement without ",
        "a valid Installed Artifact."
    );

    const STALE_INSTALLED_ARTIFACT_GLOSSARY: &str = concat!(
        "Stale Installed Artifact is a Manifest Installed Artifact with no remaining ",
        "Discovery Requirements."
    );

    const SYMLINK_DIAGNOSTIC_GLOSSARY: &str = concat!(
        "Unexpected Symlink Target differs from the Manifest destination for an ",
        "Installed Artifact. Broken Symlink means the symlink destination does not exist."
    );

    #[test]
    fn status_glossary_reports_managed_install_state_consistency() {
        assert!(
            STATUS_GLOSSARY.contains("Status"),
            "status workflow name is documented"
        );
        assert!(
            STATUS_GLOSSARY.contains("managed install state"),
            "status covers install consistency"
        );
    }

    #[test]
    fn prune_glossary_removes_stale_discovery_requirements_and_stale_installed_artifacts() {
        assert!(
            PRUNE_GLOSSARY.contains("Prune"),
            "prune workflow name is documented"
        );
        assert!(
            PRUNE_GLOSSARY.contains("Stale Discovery Requirements"),
            "prune removes stale discovery requirements"
        );
        assert!(
            PRUNE_GLOSSARY.contains("Stale Installed Artifacts"),
            "prune removes stale installed artifacts"
        );
    }

    #[test]
    fn doctor_glossary_reports_environment_and_configuration_readiness() {
        assert!(
            DOCTOR_GLOSSARY.contains("Doctor"),
            "doctor workflow name is documented"
        );
        assert!(
            DOCTOR_GLOSSARY.contains("environment"),
            "doctor covers environment readiness"
        );
        assert!(
            !DOCTOR_GLOSSARY.contains("Status"),
            "doctor does not conflate with status workflow"
        );
    }

    #[test]
    fn stale_discovery_requirement_glossary_documents_manifest_requirement_removed_from_desired_state(
    ) {
        assert!(
            STALE_DISCOVERY_REQUIREMENT_GLOSSARY.contains("Stale Discovery Requirement"),
            "stale discovery requirement term is documented"
        );
        assert!(
            STALE_DISCOVERY_REQUIREMENT_GLOSSARY.contains("Desired State"),
            "stale requirements are compared to desired state"
        );
    }

    #[test]
    fn unsatisfied_discovery_requirement_glossary_documents_desired_state_without_installed_artifact(
    ) {
        assert!(
            UNSATISFIED_DISCOVERY_REQUIREMENT_GLOSSARY.contains("Unsatisfied Discovery Requirement"),
            "unsatisfied discovery requirement term is documented"
        );
        assert!(
            UNSATISFIED_DISCOVERY_REQUIREMENT_GLOSSARY.contains("Installed Artifact"),
            "unsatisfied requirements lack valid installed artifacts"
        );
    }

    #[test]
    fn stale_installed_artifact_glossary_documents_manifest_artifact_without_requirements() {
        assert!(
            STALE_INSTALLED_ARTIFACT_GLOSSARY.contains("Stale Installed Artifact"),
            "stale installed artifact term is documented"
        );
        assert!(
            STALE_INSTALLED_ARTIFACT_GLOSSARY.contains("Discovery Requirements"),
            "stale artifacts have no remaining requirements"
        );
    }

    #[test]
    fn symlink_diagnostic_glossary_documents_unexpected_symlink_target_and_broken_symlink() {
        assert!(
            SYMLINK_DIAGNOSTIC_GLOSSARY.contains("Unexpected Symlink Target"),
            "unexpected symlink target term is documented"
        );
        assert!(
            SYMLINK_DIAGNOSTIC_GLOSSARY.contains("Broken Symlink"),
            "broken symlink term is documented"
        );
    }
}
