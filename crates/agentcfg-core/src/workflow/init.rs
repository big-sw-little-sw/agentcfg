use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::config::parse_config_str;
use crate::config_paths::{ConfigFilePaths, discover_project_root};
use crate::registry::{SkillTargetRoot, project_skill_target_roots, user_skill_target_roots};
use crate::scope::ConfigLayer;
use crate::workflow::context::WorkflowContext;
use crate::workflow::types::{
    ExistingTargetArtifact, InitRequest, InitResult, InitWarning, IoErrorSummary,
    ProjectRootDiscoveryFailed, TargetReadFailure,
};
use crate::{Error, InitError, Result};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SkillTargetInspection {
    existing_artifacts: Vec<ExistingTargetArtifact>,
    read_failures: Vec<TargetReadFailure>,
    discovery_failures: Vec<ProjectRootDiscoveryFailed>,
}

pub fn init(request: InitRequest) -> Result<InitResult> {
    let context = WorkflowContext::from_process()?;
    init_with_context(request, &context)
}

pub(crate) fn init_with_context(
    request: InitRequest,
    context: &WorkflowContext,
) -> Result<InitResult> {
    let config_paths = init_config_paths(request.config_layer, context)?;
    let contents = starter_config_contents(request.config_layer);

    parse_config_str(
        request.config_layer,
        config_paths.config_file().to_path_buf(),
        &contents,
    )?;
    create_config_file(config_paths.config_file(), request.config_layer, &contents)?;

    Ok(InitResult {
        config_file: config_paths.config_file().to_path_buf(),
        warnings: init_warnings_from(inspect_existing_skill_targets(
            request.config_layer,
            context,
        )),
    })
}

fn init_config_paths(layer: ConfigLayer, context: &WorkflowContext) -> Result<ConfigFilePaths> {
    match layer {
        ConfigLayer::SharedProject | ConfigLayer::UserProject => {
            let project_root = discover_project_root(&context.cwd)?;
            Ok(match layer {
                ConfigLayer::SharedProject => ConfigFilePaths::for_shared_project(project_root),
                ConfigLayer::UserProject => ConfigFilePaths::for_user_project(project_root),
                ConfigLayer::User => unreachable!(),
            })
        }
        ConfigLayer::User => Ok(ConfigFilePaths::for_user_config_home(
            context.user_config_home()?,
        )),
    }
}

fn starter_config_contents(layer: ConfigLayer) -> String {
    format!(
        "scope = \"{}\"\n\n[skills]\nclients = \"all\"\n",
        layer.persisted_scope()
    )
}

fn create_config_file(path: &Path, layer: ConfigLayer, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::AlreadyExists {
                InitError::ConfigAlreadyExists {
                    path: path.to_path_buf(),
                    layer,
                }
                .into()
            } else {
                Error::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;

    file.write_all(contents.as_bytes())
        .and_then(|_| file.flush())
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn inspect_existing_skill_targets(
    layer: ConfigLayer,
    context: &WorkflowContext,
) -> SkillTargetInspection {
    let mut inspection = SkillTargetInspection::default();

    let roots = match layer {
        ConfigLayer::SharedProject | ConfigLayer::UserProject => {
            match discover_project_root(&context.cwd) {
                Ok(project_root) => project_skill_target_roots(&project_root),
                Err(error) => {
                    inspection.discovery_failures.push(ProjectRootDiscoveryFailed {
                        start_dir: context.cwd.clone(),
                        error: io_error_summary(error),
                    });
                    return inspection;
                }
            }
        }
        ConfigLayer::User => {
            let Some(home_dir) = context.home_dir() else {
                return inspection;
            };
            user_skill_target_roots(home_dir)
        }
    };

    for root in roots {
        match scan_target_root(&root) {
            Ok(artifacts) => inspection.existing_artifacts.extend(artifacts),
            Err(read_failure) => inspection.read_failures.push(read_failure),
        }
    }

    inspection
}

fn init_warnings_from(inspection: SkillTargetInspection) -> Vec<InitWarning> {
    inspection
        .discovery_failures
        .into_iter()
        .map(InitWarning::ProjectRootDiscoveryFailed)
        .chain(
            inspection
                .read_failures
                .into_iter()
                .map(InitWarning::TargetReadFailure),
        )
        .chain(
            inspection
                .existing_artifacts
                .into_iter()
                .map(InitWarning::ExistingTargetArtifact),
        )
        .collect()
}

fn scan_target_root(
    root: &SkillTargetRoot,
) -> std::result::Result<Vec<ExistingTargetArtifact>, TargetReadFailure> {
    match root.path.try_exists() {
        Ok(false) => return Ok(Vec::new()),
        Ok(true) => {}
        Err(source) => return Err(scan_failure(root, source)),
    }

    let entries = fs::read_dir(&root.path).map_err(|source| scan_failure(root, source))?;

    let mut artifacts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| scan_failure(root, source))?;
        let file_type = entry.file_type().map_err(|source| scan_failure(root, source))?;
        if !file_type.is_dir() {
            continue;
        }

        let skill_dir = entry.path();
        if !skill_dir.join("SKILL.md").is_file() {
            continue;
        }

        artifacts.push(ExistingTargetArtifact {
            clients: root.clients.clone(),
            install_scope: root.install_scope,
            path: skill_dir,
        });
    }

    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn scan_failure(root: &SkillTargetRoot, source: std::io::Error) -> TargetReadFailure {
    TargetReadFailure {
        clients: root.clients.clone(),
        install_scope: root.install_scope,
        path: root.path.clone(),
        error: source.into(),
    }
}

fn io_error_summary(error: Error) -> IoErrorSummary {
    match error {
        Error::Io { source, .. } => source.into(),
        Error::PathDiscovery(path_discovery) => match path_discovery {
            crate::PathDiscoveryError::MarkerInspection { source, .. } => source.into(),
        },
        other => IoErrorSummary {
            kind: std::io::ErrorKind::Other,
            message: other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config_str;
    use crate::scope::InstallScope;
    use std::path::PathBuf;

    #[test]
    fn starter_configs_parse_for_all_layers() {
        for layer in [
            ConfigLayer::SharedProject,
            ConfigLayer::UserProject,
            ConfigLayer::User,
        ] {
            let config =
                parse_config_str(layer, "config.toml", &starter_config_contents(layer)).unwrap();

            assert_eq!(config.layer(), layer);
        }
    }

    #[test]
    fn default_init_creates_user_project_config() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::UserProject), &context).unwrap();

        let config_file = temp.path().join(".agentcfg").join("config.toml");
        assert_eq!(result.config_file, config_file);
        assert!(temp.path().join(".agentcfg").is_dir());
        assert_config_layer(&config_file, ConfigLayer::UserProject);
    }

    #[test]
    fn project_init_creates_shared_project_config_without_agentcfg_dir() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        let config_file = temp.path().join("agentcfg.toml");
        assert_eq!(result.config_file, config_file);
        assert!(!temp.path().join(".agentcfg").exists());
        assert_config_layer(&config_file, ConfigLayer::SharedProject);
    }

    #[test]
    fn user_init_creates_user_config_without_project_agentcfg_dir() {
        let temp = tempfile::tempdir().unwrap();
        let config_home = temp.path().join("xdg-config");
        let context = WorkflowContext::new(
            temp.path(),
            Some(config_home.clone()),
            Some(temp.path().join("home")),
        );

        let result = init_with_context(InitRequest::new(ConfigLayer::User), &context).unwrap();

        let config_file = config_home.join("agentcfg").join("config.toml");
        assert_eq!(result.config_file, config_file);
        assert!(!temp.path().join(".agentcfg").exists());
        assert_config_layer(&config_file, ConfigLayer::User);
    }

    #[test]
    fn existing_config_is_not_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let agentcfg_dir = temp.path().join(".agentcfg");
        let config_file = agentcfg_dir.join("config.toml");
        fs::create_dir(&agentcfg_dir).unwrap();
        fs::write(&config_file, "existing").unwrap();
        let context = context_for_project(temp.path());

        let error =
            init_with_context(InitRequest::new(ConfigLayer::UserProject), &context).unwrap_err();

        assert!(matches!(
            error,
            Error::Init(InitError::ConfigAlreadyExists {
                ref path,
                layer: ConfigLayer::UserProject,
            }) if path == &config_file
        ));
        assert_eq!(fs::read_to_string(config_file).unwrap(), "existing");
    }

    #[test]
    fn unmanaged_project_artifacts_are_reported_and_not_modified() {
        let temp = tempfile::tempdir().unwrap();
        let agents_skill = temp.path().join(".agents").join("skills").join("review");
        let claude_skill = temp.path().join(".claude").join("skills").join("docs");
        fs::create_dir_all(&agents_skill).unwrap();
        fs::create_dir_all(&claude_skill).unwrap();
        fs::write(agents_skill.join("SKILL.md"), "review").unwrap();
        fs::write(claude_skill.join("SKILL.md"), "docs").unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        let artifacts = existing_target_artifacts(&result);
        assert_eq!(
            artifacts.len(),
            2,
            "artifacts were not grouped: {artifacts:?}"
        );
        assert_artifact(
            &artifacts,
            &["codex", "cursor", "opencode", "pi"],
            &agents_skill,
        );
        assert_artifact(&artifacts, &["claude"], &claude_skill);
        assert_eq!(
            fs::read_to_string(agents_skill.join("SKILL.md")).unwrap(),
            "review"
        );
        assert!(!temp.path().join(".cline").exists());
    }

    #[test]
    fn non_skill_directories_are_not_reported_as_unmanaged_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let agents_skills = temp.path().join(".agents").join("skills");
        let stray_file = agents_skills.join(".gitkeep");
        fs::create_dir_all(&agents_skills).unwrap();
        fs::write(stray_file, "").unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        assert!(existing_target_artifacts(&result).is_empty());
    }

    #[test]
    fn missing_target_roots_are_not_created() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        assert!(result.warnings.is_empty());
        assert!(!temp.path().join(".agents").exists());
        assert!(!temp.path().join(".claude").exists());
        assert!(!temp.path().join(".cline").exists());
    }

    #[cfg(unix)]
    #[test]
    fn target_scan_failure_is_reported_as_warning_after_config_creation() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let agents_skills = temp.path().join(".agents").join("skills");
        fs::create_dir_all(&agents_skills).unwrap();
        let original_permissions = fs::metadata(&agents_skills).unwrap().permissions();
        fs::set_permissions(&agents_skills, fs::Permissions::from_mode(0o000)).unwrap();
        let context = context_for_project(temp.path());

        let result = init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context);

        fs::set_permissions(&agents_skills, original_permissions).unwrap();
        let result = result.unwrap();
        let config_file = temp.path().join("agentcfg.toml");
        assert!(config_file.is_file());
        assert!(
            result.warnings.iter().any(|warning| matches!(
                warning,
                InitWarning::TargetReadFailure(read_failure)
                    if read_failure.path == agents_skills
                        && read_failure.clients == ["codex", "cursor", "opencode", "pi"]
            )),
            "missing scan warning in {:?}",
            result.warnings
        );
    }

    #[cfg(unix)]
    #[test]
    fn target_scan_metadata_failure_is_reported_as_warning_after_config_creation() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let agents_dir = temp.path().join(".agents");
        let agents_skills = agents_dir.join("skills");
        fs::create_dir_all(&agents_skills).unwrap();
        let original_permissions = fs::metadata(&agents_dir).unwrap().permissions();
        fs::set_permissions(&agents_dir, fs::Permissions::from_mode(0o000)).unwrap();
        let context = context_for_project(temp.path());

        let result = init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context);

        fs::set_permissions(&agents_dir, original_permissions).unwrap();
        let result = result.unwrap();
        let config_file = temp.path().join("agentcfg.toml");
        assert!(config_file.is_file());
        assert!(
            result.warnings.iter().any(|warning| matches!(
                warning,
                InitWarning::TargetReadFailure(read_failure)
                    if read_failure.path == agents_skills
                        && read_failure.clients == ["codex", "cursor", "opencode", "pi"]
            )),
            "missing scan warning in {:?}",
            result.warnings
        );
    }

    fn context_for_project(project_root: &Path) -> WorkflowContext {
        WorkflowContext::new(
            project_root,
            None::<PathBuf>,
            Some(project_root.join("home")),
        )
    }

    fn assert_config_layer(path: &Path, layer: ConfigLayer) {
        let contents = fs::read_to_string(path).unwrap();
        assert_eq!(
            parse_config_str(layer, path, &contents).unwrap().layer(),
            layer
        );
    }

    fn existing_target_artifacts(result: &InitResult) -> Vec<ExistingTargetArtifact> {
        result
            .warnings
            .iter()
            .filter_map(|warning| match warning {
                InitWarning::ExistingTargetArtifact(artifact) => Some(artifact.clone()),
                InitWarning::TargetReadFailure(_) | InitWarning::ProjectRootDiscoveryFailed(_) => {
                    None
                }
            })
            .collect()
    }

    fn assert_artifact(artifacts: &[ExistingTargetArtifact], clients: &[&str], path: &Path) {
        assert!(
            artifacts.iter().any(|artifact| artifact.clients == clients
                && artifact.install_scope == InstallScope::Project
                && artifact.path == path),
            "missing artifact for {clients:?} at {} in {artifacts:?}",
            path.display()
        );
    }
}
