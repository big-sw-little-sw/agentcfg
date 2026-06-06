//! Observes Managed State and Client Discovery Locations.

pub mod skills;

use crate::{AgentcfgResult, ClientSelection, InstallLevel};

/// Limits installation observation to one Install Level and selected Clients.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallationSelection {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
}

/// Observable install-state facts normalized from local evidence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObservedInstallation {
    pub skills: skills::ObservedSkillInstallation,
}

pub fn observe(_selection: InstallationSelection) -> AgentcfgResult<ObservedInstallation> {
    unimplemented!("installation observation is not implemented yet")
}
