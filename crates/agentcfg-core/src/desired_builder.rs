//! Builds Desired State from active Config Layers and command filters.

use crate::{
    AgentcfgResult, ClientSelection, InstallLevel, config::ConfigLayer,
    state::desired::DesiredState,
};

pub fn build(
    _config_layers: &[ConfigLayer],
    _install_level: InstallLevel,
    _clients: ClientSelection,
) -> AgentcfgResult<DesiredState> {
    unimplemented!("desired state building is not implemented yet")
}
