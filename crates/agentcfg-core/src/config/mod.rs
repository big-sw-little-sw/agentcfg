//! Agent Configuration documents and request building.

pub mod skills;

use crate::{AgentcfgResult, ClientSelection, ConfigLayerKind, InstallLevel};

/// One persisted Agent Configuration document before active-layer request building.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigDoc {
    pub skills: skills::SkillConfigDoc,
}

/// A loaded Config Layer document with its layer identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedConfigDoc {
    pub kind: ConfigLayerKind,
    pub doc: ConfigDoc,
}

/// One active Config Layer after command-level filtering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigLayerRequest {
    pub kind: ConfigLayerKind,
}

/// Active configuration intent after layer, Install Level, and client selection rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigRequest {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
    pub layers: Vec<ConfigLayerRequest>,
    pub skills: skills::SkillConfigRequest,
}

pub fn build_request(
    _config_docs: &[LoadedConfigDoc],
    _install_level: InstallLevel,
    _clients: ClientSelection,
) -> AgentcfgResult<ConfigRequest> {
    unimplemented!("config request building is not implemented yet")
}
