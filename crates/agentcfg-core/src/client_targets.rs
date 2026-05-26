use std::path::{Path, PathBuf};

use crate::scope::InstallScope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SkillTargetRoot {
    pub(crate) clients: Vec<&'static str>,
    pub(crate) install_scope: InstallScope,
    pub(crate) path: PathBuf,
}

impl SkillTargetRoot {
    fn new(clients: Vec<&'static str>, install_scope: InstallScope, path: PathBuf) -> Self {
        Self {
            clients,
            install_scope,
            path,
        }
    }
}

pub(crate) fn project_skill_target_roots(project_root: &Path) -> Vec<SkillTargetRoot> {
    skill_target_roots(project_root, InstallScope::Project)
}

pub(crate) fn user_skill_target_roots(home_dir: &Path) -> Vec<SkillTargetRoot> {
    skill_target_roots(home_dir, InstallScope::User)
}

fn skill_target_roots(base: &Path, install_scope: InstallScope) -> Vec<SkillTargetRoot> {
    let raw_roots = [
        ("codex", base.join(".agents").join("skills")),
        ("pi", base.join(".agents").join("skills")),
        ("opencode", base.join(".agents").join("skills")),
        ("cursor", base.join(".agents").join("skills")),
        ("claude", base.join(".claude").join("skills")),
        ("cline", base.join(".cline").join("skills")),
    ];

    let mut roots = Vec::<SkillTargetRoot>::new();
    for (client, path) in raw_roots {
        if let Some(root) = roots
            .iter_mut()
            .find(|root| root.install_scope == install_scope && root.path == path)
        {
            root.clients.push(client);
        } else {
            roots.push(SkillTargetRoot::new(vec![client], install_scope, path));
        }
    }

    for root in &mut roots {
        root.clients.sort_unstable();
    }

    roots
}
