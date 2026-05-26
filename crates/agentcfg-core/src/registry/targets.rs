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
    const ROOTS: &[(&str, &[&str])] = &[
        (".agents/skills", &["codex", "pi", "opencode", "cursor"]),
        (".claude/skills", &["claude"]),
        (".cline/skills", &["cline"]),
    ];

    let mut roots = Vec::<SkillTargetRoot>::new();
    for (relative, clients) in ROOTS {
        let path = base.join(relative);
        if let Some(root) = roots
            .iter_mut()
            .find(|root| root.install_scope == install_scope && root.path == path)
        {
            root.clients.extend_from_slice(clients);
        } else {
            roots.push(SkillTargetRoot::new(
                clients.to_vec(),
                install_scope,
                path,
            ));
        }
    }

    for root in &mut roots {
        root.clients.sort_unstable();
        root.clients.dedup();
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_clients_that_share_a_target_path() {
        let roots = project_skill_target_roots(Path::new("/repo"));
        let agents = roots
            .iter()
            .find(|root| root.path == Path::new("/repo/.agents/skills"))
            .expect("agents root");

        assert_eq!(
            agents.clients,
            ["codex", "cursor", "opencode", "pi"]
        );
    }

    #[test]
    fn keeps_distinct_paths_for_native_client_layouts() {
        let roots = project_skill_target_roots(Path::new("/repo"));
        let paths = roots.iter().map(|root| root.path.as_path()).collect::<Vec<_>>();

        assert!(paths.contains(&Path::new("/repo/.agents/skills")));
        assert!(paths.contains(&Path::new("/repo/.claude/skills")));
        assert!(paths.contains(&Path::new("/repo/.cline/skills")));
    }
}
