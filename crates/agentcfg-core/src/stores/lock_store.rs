//! Reads and writes persisted lockfiles.

use crate::{
    AgentcfgResult,
    config::LoadedConfigDoc,
    lockfile::{LockfileChanges, Lockfiles},
};

pub fn load_for_config_docs(_config_docs: &[LoadedConfigDoc]) -> AgentcfgResult<Lockfiles> {
    unimplemented!("lockfile loading is not implemented yet")
}

pub fn write_lockfile_changes(_changes: LockfileChanges) -> AgentcfgResult<()> {
    unimplemented!("lockfile writing is not implemented yet")
}
