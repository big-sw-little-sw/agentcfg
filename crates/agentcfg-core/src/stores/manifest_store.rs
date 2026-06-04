//! Reads and writes Manifest state.

use crate::{AgentcfgResult, InstallLevel, manifest::ManifestSnapshot};

pub fn load_for_install_level(_install_level: InstallLevel) -> AgentcfgResult<ManifestSnapshot> {
    unimplemented!("manifest loading is not implemented yet")
}

pub fn write(_snapshot: ManifestSnapshot) -> AgentcfgResult<()> {
    unimplemented!("manifest writing is not implemented yet")
}
