//! Command-specific lifecycle policy for desired and observed install state.

pub mod apply;
mod classifiers;
pub mod preview;
pub mod prune;
pub mod status;

pub use apply::{ApplyInput, ApplyPlan};
pub use preview::{PreviewInput, PreviewReport};
pub use prune::{PruneInput, PrunePlan};
pub use status::{StatusInput, StatusReport};

use crate::AgentcfgResult;

pub fn preview(input: PreviewInput) -> AgentcfgResult<PreviewReport> {
    preview::reconcile(input)
}

pub fn apply(input: ApplyInput) -> AgentcfgResult<ApplyPlan> {
    apply::reconcile(input)
}

pub fn prune(input: PruneInput) -> AgentcfgResult<PrunePlan> {
    prune::reconcile(input)
}

pub fn status(input: StatusInput) -> AgentcfgResult<StatusReport> {
    status::reconcile(input)
}
