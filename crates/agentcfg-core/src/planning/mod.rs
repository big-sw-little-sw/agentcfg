//! Command-specific planning for pinned and observed installation state.

pub mod apply;
mod classifiers;
pub mod preview;
pub mod prune;
pub mod skills;
pub mod status;

pub use apply::{ApplyInput, ApplyPlan};
pub use preview::{PreviewInput, PreviewReport};
pub use prune::{PruneInput, PrunePlan};
pub use status::{StatusInput, StatusReport};

use crate::AgentcfgResult;

pub fn preview(input: PreviewInput) -> AgentcfgResult<PreviewReport> {
    preview::plan(input)
}

pub fn apply(input: ApplyInput) -> AgentcfgResult<ApplyPlan> {
    apply::plan(input)
}

pub fn prune(input: PruneInput) -> AgentcfgResult<PrunePlan> {
    prune::plan(input)
}

pub fn status(input: StatusInput) -> AgentcfgResult<StatusReport> {
    status::plan(input)
}
