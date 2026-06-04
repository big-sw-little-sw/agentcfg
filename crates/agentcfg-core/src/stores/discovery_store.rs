//! Creates and inspects Client Discovery Location artifacts.

use crate::{AgentcfgResult, ClientDiscoveryLocation, DiscoveryName, TreeDigest};

/// Observable entries under selected Client Discovery Locations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoverySnapshot {}

/// Describes one discovery symlink installation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverySymlinkRequest {
    pub location: ClientDiscoveryLocation,
    pub discovery_name: DiscoveryName,
    pub target_digest: TreeDigest,
}

pub fn inspect_locations(
    _locations: Vec<ClientDiscoveryLocation>,
) -> AgentcfgResult<DiscoverySnapshot> {
    unimplemented!("discovery location inspection is not implemented yet")
}

pub fn install_symlink(_request: DiscoverySymlinkRequest) -> AgentcfgResult<()> {
    unimplemented!("discovery symlink installation is not implemented yet")
}
