use super::types::{
    DoctorRequest, DoctorResult, PlanRequest, PlanResult, PruneRequest, PruneResult,
    StatusRequest, StatusResult, SyncRequest, SyncResult,
};
use crate::Result;

pub fn plan(_request: PlanRequest) -> Result<PlanResult> {
    Ok(PlanResult {})
}

pub fn sync(_request: SyncRequest) -> Result<SyncResult> {
    Ok(SyncResult {})
}

pub fn prune(_request: PruneRequest) -> Result<PruneResult> {
    Ok(PruneResult {})
}

pub fn status(_request: StatusRequest) -> Result<StatusResult> {
    Ok(StatusResult {})
}

pub fn doctor(_request: DoctorRequest) -> Result<DoctorResult> {
    Ok(DoctorResult {})
}
