use std::path::{Path, PathBuf};

use crate::scope::InstallLevel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SkillTargetRoot {
    pub(crate) clients: Vec<&'static str>,
    pub(crate) install_level: InstallLevel,
    pub(crate) path: PathBuf,
}

impl SkillTargetRoot {
    fn new(clients: Vec<&'static str>, install_level: InstallLevel, path: PathBuf) -> Self {
        Self {
            clients,
            install_level,
            path,
        }
    }
}

pub(crate) fn project_skill_target_roots(project_root: &Path) -> Vec<SkillTargetRoot> {
    skill_target_roots(project_root, InstallLevel::Project)
}

pub(crate) fn user_skill_target_roots(home_dir: &Path) -> Vec<SkillTargetRoot> {
    skill_target_roots(home_dir, InstallLevel::User)
}

fn skill_target_roots(base: &Path, install_level: InstallLevel) -> Vec<SkillTargetRoot> {
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
            .find(|root| root.install_level == install_level && root.path == path)
        {
            root.clients.push(client);
        } else {
            roots.push(SkillTargetRoot::new(vec![client], install_level, path));
        }
    }

    for root in &mut roots {
        root.clients.sort_unstable();
    }

    roots
}
