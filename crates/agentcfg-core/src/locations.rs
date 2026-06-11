//! Config Layer and Install Level path resolution.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{ConfigLayerId, InstallLevel};

/// Agent Configuration File basename shared by project and user config layers.
pub const AGENT_CONFIGURATION_FILE_NAME: &str = "agentcfg.toml";

/// Project-local configuration directory under Project Root.
pub const PROJECT_LOCAL_CONFIG_DIR_NAME: &str = ".agentcfg";

/// User Config directory name under XDG config home.
pub const USER_CONFIG_DIR_NAME: &str = "agentcfg";

/// User Project Config path relative to Project Root.
pub const USER_PROJECT_CONFIG_RELATIVE_PATH: &str = concat!(".agentcfg", "/", "agentcfg.toml");

pub fn shared_project_config_path(project_root: &Path) -> PathBuf {
    project_root.join(AGENT_CONFIGURATION_FILE_NAME)
}

pub fn project_local_config_dir(project_root: &Path) -> PathBuf {
    project_root.join(PROJECT_LOCAL_CONFIG_DIR_NAME)
}

pub fn user_project_config_path(project_root: &Path) -> PathBuf {
    project_local_config_dir(project_root).join(AGENT_CONFIGURATION_FILE_NAME)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserConfigPathError {
    #[error("cannot resolve user config path: neither XDG_CONFIG_HOME nor HOME is set")]
    MissingHomeEnv,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectRootError {
    #[error("project root does not exist: {}", .0.display())]
    NotFound(PathBuf),
    #[error("project root is not a directory: {}", .0.display())]
    NotDirectory(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAnchorSource {
    GitRoot,
    ProjectMarkers,
    ExplicitOverride,
    Init,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredProjectRoot {
    pub root: PathBuf,
    pub anchor: Option<ProjectAnchorSource>,
}

/// Workflow-scoped paths derived from the current working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContext {
    pub project_root: PathBuf,
    pub anchor: Option<ProjectAnchorSource>,
}

impl WorkflowContext {
    pub fn from_cwd() -> Result<Self, std::io::Error> {
        let cwd = std::env::current_dir()?;
        Ok(build_workflow_context(cwd, None).expect("automatic discovery does not fail"))
    }

    pub fn from_project_root(project_root: PathBuf) -> Self {
        Self {
            project_root,
            anchor: Some(ProjectAnchorSource::ExplicitOverride),
        }
    }

    pub fn is_anchored(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn config_layer_path(&self, layer: ConfigLayerId) -> Result<PathBuf, UserConfigPathError> {
        match layer {
            ConfigLayerId::SharedProject => Ok(shared_project_config_path(&self.project_root)),
            ConfigLayerId::UserProject => Ok(user_project_config_path(&self.project_root)),
            ConfigLayerId::User => user_config_path(),
        }
    }
}

pub fn build_workflow_context(
    cwd: PathBuf,
    explicit_project_root: Option<PathBuf>,
) -> Result<WorkflowContext, ProjectRootError> {
    if let Some(root) = explicit_project_root {
        if !root.exists() {
            return Err(ProjectRootError::NotFound(root));
        }
        if !root.is_dir() {
            return Err(ProjectRootError::NotDirectory(root));
        }
        return Ok(WorkflowContext {
            project_root: root,
            anchor: Some(ProjectAnchorSource::ExplicitOverride),
        });
    }

    let discovered = discover_project_root(&cwd);
    Ok(WorkflowContext {
        project_root: discovered.root,
        anchor: discovered.anchor,
    })
}

pub fn discover_project_root(start: &Path) -> DiscoveredProjectRoot {
    if let Some(root) = start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
    {
        return DiscoveredProjectRoot {
            root,
            anchor: Some(ProjectAnchorSource::GitRoot),
        };
    }

    if let Some(root) = start
        .ancestors()
        .find(|dir| is_project_marker_root(dir))
        .map(Path::to_path_buf)
    {
        return DiscoveredProjectRoot {
            root,
            anchor: Some(ProjectAnchorSource::ProjectMarkers),
        };
    }

    DiscoveredProjectRoot {
        root: start.to_path_buf(),
        anchor: None,
    }
}

/// Whether `dir` is a Project Root evidenced by project markers.
///
/// The project-local configuration directory itself is never treated as Project Root.
pub fn is_project_marker_root(dir: &Path) -> bool {
    if dir
        .file_name()
        .is_some_and(|name| name == PROJECT_LOCAL_CONFIG_DIR_NAME)
    {
        return false;
    }
    has_project_markers(dir)
}

pub fn has_project_markers(dir: &Path) -> bool {
    shared_project_config_path(dir).is_file()
        || user_project_config_path(dir).is_file()
        || project_local_config_dir(dir).is_dir()
}

pub fn user_config_path() -> Result<PathBuf, UserConfigPathError> {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg_config_home)
            .join(USER_CONFIG_DIR_NAME)
            .join(AGENT_CONFIGURATION_FILE_NAME));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".config")
            .join(USER_CONFIG_DIR_NAME)
            .join(AGENT_CONFIGURATION_FILE_NAME));
    }

    Err(UserConfigPathError::MissingHomeEnv)
}

pub fn active_config_layers(install_level: InstallLevel) -> Vec<ConfigLayerId> {
    match install_level {
        InstallLevel::Project => vec![ConfigLayerId::SharedProject, ConfigLayerId::UserProject],
        InstallLevel::User => vec![ConfigLayerId::User],
    }
}

pub fn layer_label(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "Shared Project Config",
        ConfigLayerId::UserProject => "User Project Config",
        ConfigLayerId::User => "User Config",
    }
}

pub fn layer_relative_path_label(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => AGENT_CONFIGURATION_FILE_NAME,
        ConfigLayerId::UserProject => USER_PROJECT_CONFIG_RELATIVE_PATH,
        ConfigLayerId::User => AGENT_CONFIGURATION_FILE_NAME,
    }
}

pub fn persisted_config_layer_value(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "shared-project",
        ConfigLayerId::UserProject => "user-project",
        ConfigLayerId::User => "user",
    }
}
