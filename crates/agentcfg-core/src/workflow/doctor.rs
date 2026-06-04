//! Reports local readiness for configuration and managed install workflows.

use crate::AgentcfgResult;

/// Command request for Doctor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DoctorRequest {}

/// Readiness diagnostics for later terminal rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DoctorReport {}

pub fn run(_request: DoctorRequest) -> AgentcfgResult<DoctorReport> {
    unimplemented!("doctor workflow is not implemented yet")
}
