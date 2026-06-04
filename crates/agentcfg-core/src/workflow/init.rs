//! Initializes the selected Config Layer.

use crate::{AgentcfgResult, ConfigLayerKind};

/// Command request for creating one Config Layer file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitRequest {
    pub target_layer: ConfigLayerKind,
}

pub fn run(_request: InitRequest) -> AgentcfgResult<()> {
    unimplemented!("init workflow is not implemented yet")
}
