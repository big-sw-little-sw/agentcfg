//! Reads and writes persisted Config Layer files.

use crate::{AgentcfgResult, ConfigLayerKind, InstallLevel, config::LoadedConfigDoc};

pub fn load_active_config_docs(
    _install_level: InstallLevel,
) -> AgentcfgResult<Vec<LoadedConfigDoc>> {
    unimplemented!("config document loading is not implemented yet")
}

pub fn create_config_doc_file(_layer: ConfigLayerKind) -> AgentcfgResult<()> {
    unimplemented!("config document writing is not implemented yet")
}
