//! Built-in client identifiers and skill target path registry (V1).

mod targets;

pub(crate) use targets::{
    SkillTargetRoot, project_skill_target_roots, user_skill_target_roots,
};

/// Client ids supported by this `agentcfg` version for `[skills].clients`.
pub const SUPPORTED_CLIENT_IDS: &[&str] =
    &["codex", "pi", "opencode", "cursor", "claude", "cline"];

/// Returns the canonical client id when `id` is supported (trimmed, case-sensitive).
pub fn normalize_client_id(id: &str) -> Option<&'static str> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return None;
    }
    SUPPORTED_CLIENT_IDS
        .iter()
        .copied()
        .find(|supported| *supported == trimmed)
}
