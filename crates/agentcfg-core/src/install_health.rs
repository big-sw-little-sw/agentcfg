//! Glossary anchors for **Status**, **Prune**, and **Doctor** install-health vocabulary.
//!
//! Behavior is implemented in `workflow` and related modules in later milestones.
//! This module documents terms from `UBIQUITOUS-LANGUAGE.md` so diagnostics stay aligned.

// **Unmanaged Artifact** — filesystem entry at a Client Discovery Location not in the Manifest.
// **Stale Discovery Requirement** — Manifest requirement no longer in Desired State.
// **Unsatisfied Discovery Requirement** — Desired State requirement without a valid Installed Artifact.
// **Stale Installed Artifact** — Manifest-recorded Installed Artifact with no remaining Discovery Requirements.
// **Broken Symlink** / **Unexpected Symlink Target** — symlink filesystem diagnostics only.

#[cfg(test)]
mod tests {
    #[test]
    fn install_health_glossary_stale_discovery_requirement_anchor() {
        const _: &str = "Stale Discovery Requirement";
    }

    #[test]
    fn install_health_glossary_stale_installed_artifact_anchor() {
        const _: &str = "Stale Installed Artifact";
    }
}
