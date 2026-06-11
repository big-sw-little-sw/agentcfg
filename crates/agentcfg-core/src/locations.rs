//! Config Layer and Install Level path resolution.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{ConfigLayerId, InstallLevel};

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
            ConfigLayerId::SharedProject => Ok(self.project_root.join("agentcfg.toml")),
            ConfigLayerId::UserProject => {
                Ok(self.project_root.join(".agentcfg").join("agentcfg.toml"))
            }
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
    let git_root = start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf);
    let marker_root = start
        .ancestors()
        .find(|dir| has_project_markers(dir))
        .map(Path::to_path_buf);

    if let Some(root) = git_root {
        DiscoveredProjectRoot {
            root,
            anchor: Some(ProjectAnchorSource::GitRoot),
        }
    } else if let Some(root) = marker_root {
        DiscoveredProjectRoot {
            root,
            anchor: Some(ProjectAnchorSource::ProjectMarkers),
        }
    } else {
        DiscoveredProjectRoot {
            root: start.to_path_buf(),
            anchor: None,
        }
    }
}

pub fn resolve_project_root(start: &Path) -> DiscoveredProjectRoot {
    discover_project_root(start)
}

pub fn has_project_markers(dir: &Path) -> bool {
    dir.join("agentcfg.toml").is_file()
        || dir.join(".agentcfg").join("agentcfg.toml").is_file()
        || dir.join(".agentcfg").is_dir()
}

pub fn user_config_path() -> Result<PathBuf, UserConfigPathError> {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg_config_home)
            .join("agentcfg")
            .join("agentcfg.toml"));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".config")
            .join("agentcfg")
            .join("agentcfg.toml"));
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
        ConfigLayerId::SharedProject => "agentcfg.toml",
        ConfigLayerId::UserProject => ".agentcfg/agentcfg.toml",
        ConfigLayerId::User => "agentcfg.toml",
    }
}

pub fn persisted_config_layer_value(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "shared-project",
        ConfigLayerId::UserProject => "user-project",
        ConfigLayerId::User => "user",
    }
}
