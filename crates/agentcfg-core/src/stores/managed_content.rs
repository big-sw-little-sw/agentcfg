//! Writes content-addressed Managed Skill Content.

use std::path::PathBuf;

use crate::{AgentcfgResult, TreeDigest};

/// Prepared skill content ready to copy into Managed State.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedSkillContentWrite {
    pub prepared_content_root: PathBuf,
    pub expected_digest: TreeDigest,
}

pub fn write(_content: ManagedSkillContentWrite) -> AgentcfgResult<()> {
    unimplemented!("managed content writing is not implemented yet")
}
