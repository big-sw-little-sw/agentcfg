use std::path::{Path, PathBuf};

use agentcfg_core::{
    build_workflow_context, discover_project_root, has_project_markers, ProjectAnchorSource,
    ProjectRootError,
};

#[test]
fn discover_project_root_prefers_git_root_over_closer_markers() {
    let fixture = Fixture::new("git-over-markers");
    let project = fixture.root.join("project");
    let repo = project.join("repo");
    let nested = repo.join("packages").join("app");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    std::fs::create_dir_all(project.join(".agentcfg")).expect("create marker dir");
    std::fs::create_dir_all(repo.join(".git")).expect("create git dir");

    let discovered = discover_project_root(&nested);
    assert_eq!(discovered.root, repo);
    assert_eq!(discovered.anchor, Some(ProjectAnchorSource::GitRoot));
}

#[test]
fn discover_project_root_finds_shared_config_marker_when_no_git() {
    assert_marker_discovery("shared-config-marker", |project| {
        std::fs::write(project.join("agentcfg.toml"), "").expect("write shared config");
    });
}

#[test]
fn discover_project_root_finds_user_config_marker_when_no_git() {
    assert_marker_discovery("user-config-marker", |project| {
        std::fs::create_dir_all(project.join(".agentcfg")).expect("create config dir");
        std::fs::write(project.join(".agentcfg/agentcfg.toml"), "").expect("write user config");
    });
}

#[test]
fn discover_project_root_finds_config_directory_marker_when_no_git() {
    assert_marker_discovery("config-dir-marker", |project| {
        std::fs::create_dir_all(project.join(".agentcfg")).expect("create config dir");
    });
}

fn assert_marker_discovery(name: &str, setup: impl FnOnce(&Path)) {
    let fixture = Fixture::new(name);
    let project = fixture.root.join("project");
    let nested = project.join("apps").join("service");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    setup(&project);

    let discovered = discover_project_root(&nested);
    assert_eq!(discovered.root, project);
    assert_eq!(discovered.anchor, Some(ProjectAnchorSource::ProjectMarkers));
}

#[test]
fn discover_project_root_is_unanchored_without_git_or_markers() {
    let fixture = Fixture::new("unanchored");
    let nested = fixture.root.join("scratch").join("work");
    std::fs::create_dir_all(&nested).expect("create nested dir");

    let discovered = discover_project_root(&nested);
    assert_eq!(discovered.root, nested);
    assert_eq!(discovered.anchor, None);
}

#[test]
fn build_workflow_context_uses_explicit_project_root_override() {
    let fixture = Fixture::new("explicit-override");
    let project = fixture.root.join("marked-project");
    let other = fixture.root.join("other-project");
    std::fs::create_dir_all(&project).expect("create project");
    std::fs::create_dir_all(other.join(".agentcfg")).expect("create marker");
    std::fs::create_dir_all(project.join("nested")).expect("create nested");

    let context =
        build_workflow_context(project.join("nested"), Some(other.clone())).expect("build context");

    assert_eq!(context.project_root, other);
    assert_eq!(context.anchor, Some(ProjectAnchorSource::ExplicitOverride));
}

#[test]
fn build_workflow_context_rejects_missing_explicit_project_root() {
    let fixture = Fixture::new("missing-override");
    let missing = fixture.root.join("missing");

    let error = build_workflow_context(fixture.root.clone(), Some(missing.clone()))
        .expect_err("missing path");

    assert_eq!(error, ProjectRootError::NotFound(missing));
}

#[test]
fn has_project_markers_detects_shared_user_and_config_directory_markers() {
    let fixture = Fixture::new("marker-detection");
    let root = fixture.root.join("project");
    std::fs::create_dir_all(&root).expect("create project");

    assert!(!has_project_markers(&root));

    std::fs::write(root.join("agentcfg.toml"), "").expect("write shared config");
    assert!(has_project_markers(&root));

    let user_only = fixture.root.join("user-only");
    std::fs::create_dir_all(user_only.join(".agentcfg")).expect("create config dir");
    std::fs::write(user_only.join(".agentcfg/agentcfg.toml"), "").expect("write user config");
    assert!(has_project_markers(&user_only));

    let dir_only = fixture.root.join("dir-only");
    std::fs::create_dir_all(dir_only.join(".agentcfg")).expect("create config dir");
    assert!(has_project_markers(&dir_only));
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir()
            .join("agentcfg-tests")
            .join(format!("project-root-{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).expect("remove previous fixture");
        }
        std::fs::create_dir_all(&root).expect("create fixture root");
        Self { root }
    }
}
