//! Reads and writes persisted Config Layer files.

use crate::{AgentcfgResult, ConfigLayerKind, InstallLevel, config::ConfigLayer};

pub fn load_active_layers(_install_level: InstallLevel) -> AgentcfgResult<Vec<ConfigLayer>> {
    unimplemented!("config layer loading is not implemented yet")
}

pub fn create_config_layer_file(_layer: ConfigLayerKind) -> AgentcfgResult<()> {
    unimplemented!("config layer writing is not implemented yet")
}
