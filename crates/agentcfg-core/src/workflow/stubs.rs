use crate::{Result, UnsupportedError};

use super::types::{
    ApplyRequest, ApplyResult, DoctorRequest, DoctorResult, PreviewRequest, PreviewResult,
    PruneRequest, PruneResult, StatusRequest, StatusResult,
};

pub fn preview(_request: PreviewRequest) -> Result<PreviewResult> {
    workflow_not_implemented("preview")
}

pub fn apply(_request: ApplyRequest) -> Result<ApplyResult> {
    workflow_not_implemented("apply")
}

pub fn prune(_request: PruneRequest) -> Result<PruneResult> {
    workflow_not_implemented("prune")
}

pub fn status(_request: StatusRequest) -> Result<StatusResult> {
    workflow_not_implemented("status")
}

pub fn doctor(_request: DoctorRequest) -> Result<DoctorResult> {
    workflow_not_implemented("doctor")
}

fn workflow_not_implemented<T>(workflow: &'static str) -> Result<T> {
    Err(UnsupportedError::WorkflowNotImplemented { workflow }.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer_level::InstallLevel;
    use crate::{Error, UnsupportedError};

    use super::super::types::{ApplyRequest, PreviewRequest, SkillSourceResolutionPolicy};

    #[test]
    fn preview_returns_workflow_not_implemented() {
        let error = preview(PreviewRequest::new(
            InstallLevel::Project,
            SkillSourceResolutionPolicy::UseLocked,
        ))
        .unwrap_err();

        assert!(matches!(
            error,
            Error::Unsupported(UnsupportedError::WorkflowNotImplemented {
                workflow: "preview"
            })
        ));
    }

    #[test]
    fn apply_returns_workflow_not_implemented() {
        let error = apply(ApplyRequest::new(
            InstallLevel::Project,
            SkillSourceResolutionPolicy::UseLocked,
        ))
        .unwrap_err();

        assert!(matches!(
            error,
            Error::Unsupported(UnsupportedError::WorkflowNotImplemented {
                workflow: "apply"
            })
        ));
    }
}
