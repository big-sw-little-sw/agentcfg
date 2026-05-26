use std::path::{Path, PathBuf};

use crate::scope::InstallScope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientSkillTargetRoot {
    pub(crate) client: &'static str,
    pub(crate) install_scope: InstallScope,
    pub(crate) path: PathBuf,
}

impl ClientSkillTargetRoot {
    fn new(client: &'static str, install_scope: InstallScope, path: PathBuf) -> Self {
        Self {
            client,
            install_scope,
            path,
        }
    }
}

pub(crate) fn project_skill_target_roots(project_root: &Path) -> Vec<ClientSkillTargetRoot> {
    skill_target_roots(project_root, InstallScope::Project)
}

pub(crate) fn user_skill_target_roots(home_dir: &Path) -> Vec<ClientSkillTargetRoot> {
    skill_target_roots(home_dir, InstallScope::User)
}

fn skill_target_roots(base: &Path, install_scope: InstallScope) -> Vec<ClientSkillTargetRoot> {
    vec![
        ClientSkillTargetRoot::new("codex", install_scope, base.join(".agents").join("skills")),
        ClientSkillTargetRoot::new("pi", install_scope, base.join(".agents").join("skills")),
        ClientSkillTargetRoot::new(
            "opencode",
            install_scope,
            base.join(".agents").join("skills"),
        ),
        ClientSkillTargetRoot::new("cursor", install_scope, base.join(".agents").join("skills")),
        ClientSkillTargetRoot::new("claude", install_scope, base.join(".claude").join("skills")),
        ClientSkillTargetRoot::new("cline", install_scope, base.join(".cline").join("skills")),
    ]
}
