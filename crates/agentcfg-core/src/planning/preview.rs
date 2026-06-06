//! Preview report shaping for planned installation changes.

use crate::{AgentcfgResult, installation::ObservedInstallation, resolution::PlannedPinnedConfig};

/// Planning inputs for read-only Preview reporting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewInput {
    pub planned_pinned: PlannedPinnedConfig,
    pub observed_installation: ObservedInstallation,
}

/// Structured Preview findings for later terminal rendering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreviewReport {
    pub skills: super::skills::SkillPreviewReport,
}

pub(crate) fn plan(_input: PreviewInput) -> AgentcfgResult<PreviewReport> {
    unimplemented!("preview planning is not implemented yet")
}
