//! Reads and writes persisted lockfiles.

use crate::{
    AgentcfgResult,
    config::ConfigLayer,
    lockfile::{ExistingLocks, LockfileChange},
};

pub fn load_for_config_layers(_config_layers: &[ConfigLayer]) -> AgentcfgResult<ExistingLocks> {
    unimplemented!("lockfile loading is not implemented yet")
}

pub fn write_lockfile_changes(_changes: Vec<LockfileChange>) -> AgentcfgResult<()> {
    unimplemented!("lockfile writing is not implemented yet")
}
