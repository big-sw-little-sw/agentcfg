//! Built-in **Client Discovery Registry** catalog and **Client Discovery Locations**.
//!
//! Each [`ClientDiscoveryLocation`] groups one or more clients that share the same
//! filesystem path for skill discovery at a given [`InstallLevel`].
//!
//! Future manifest **Discovery Requirements** are keyed by Config Layer, Client, and
//! Install Level (not yet modeled as a Rust struct).

use std::path::{Path, PathBuf};

use crate::layer_level::InstallLevel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientDiscoveryLocation {
    pub(crate) clients: Vec<&'static str>,
    pub(crate) install_level: InstallLevel,
    pub(crate) path: PathBuf,
}

impl ClientDiscoveryLocation {
    fn new(clients: Vec<&'static str>, install_level: InstallLevel, path: PathBuf) -> Self {
        Self {
            clients,
            install_level,
            path,
        }
    }
}

pub(crate) fn project_client_discovery_locations(
    project_root: &Path,
) -> Vec<ClientDiscoveryLocation> {
    client_discovery_locations(project_root, InstallLevel::Project)
}

pub(crate) fn user_client_discovery_locations(home_dir: &Path) -> Vec<ClientDiscoveryLocation> {
    client_discovery_locations(home_dir, InstallLevel::User)
}

fn client_discovery_locations(
    base: &Path,
    install_level: InstallLevel,
) -> Vec<ClientDiscoveryLocation> {
    let raw_locations = [
        ("codex", base.join(".agents").join("skills")),
        ("pi", base.join(".agents").join("skills")),
        ("opencode", base.join(".agents").join("skills")),
        ("cursor", base.join(".agents").join("skills")),
        ("claude", base.join(".claude").join("skills")),
        ("cline", base.join(".cline").join("skills")),
    ];

    let mut locations = Vec::<ClientDiscoveryLocation>::new();
    for (client, path) in raw_locations {
        if let Some(location) = locations
            .iter_mut()
            .find(|location| location.install_level == install_level && location.path == path)
        {
            location.clients.push(client);
        } else {
            locations.push(ClientDiscoveryLocation::new(
                vec![client],
                install_level,
                path,
            ));
        }
    }

    for location in &mut locations {
        location.clients.sort_unstable();
    }

    locations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_discovery_registry_groups_shared_agents_path() {
        let temp = tempfile::tempdir().unwrap();
        let locations = user_client_discovery_locations(temp.path());

        let agents = locations
            .iter()
            .find(|location| location.path == temp.path().join(".agents").join("skills"))
            .expect("missing shared .agents/skills location");

        assert_eq!(
            agents.clients,
            ["codex", "cursor", "opencode", "pi"],
            "clients sharing .agents/skills were not grouped"
        );
        assert_eq!(agents.install_level, InstallLevel::User);
    }

    #[test]
    fn discovery_registry_groups_shared_agents_path() {
        let temp = tempfile::tempdir().unwrap();
        let locations = project_client_discovery_locations(temp.path());

        let agents = locations
            .iter()
            .find(|location| location.path == temp.path().join(".agents").join("skills"))
            .expect("missing shared .agents/skills location");

        assert_eq!(
            agents.clients,
            ["codex", "cursor", "opencode", "pi"],
            "clients sharing .agents/skills were not grouped"
        );
        assert_eq!(agents.install_level, InstallLevel::Project);
    }
}
