//! Parsed Agent Configuration models before active-layer resolution.

use crate::ConfigLayerKind;

/// One loaded Config Layer before it is compiled into Desired State.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigLayer {
    pub kind: ConfigLayerKind,
}
