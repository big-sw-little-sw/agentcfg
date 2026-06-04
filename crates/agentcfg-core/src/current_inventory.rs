//! Normalizes Manifest, Managed State, and Client Discovery Location evidence.

use crate::{AgentcfgResult, ClientSelection, InstallLevel, state::current::CurrentState};

/// Limits inventory reads to one Install Level and selected Clients.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventorySelection {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
}

pub fn read(_selection: InventorySelection) -> AgentcfgResult<CurrentState> {
    unimplemented!("current inventory reading is not implemented yet")
}
