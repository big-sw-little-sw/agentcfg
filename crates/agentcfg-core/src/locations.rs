//! Config Layer and Install Level path resolution.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{ConfigLayerId, InstallLevel};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UserConfigPathError {
    #[error("cannot resolve user config path: neither XDG_CONFIG_HOME nor HOME is set")]
    MissingHomeEnv,
}

/// Workflow-scoped paths derived from the current working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContext {
    pub project_root: PathBuf,
}

impl WorkflowContext {
    pub fn from_cwd() -> Result<Self, std::io::Error> {
        let cwd = std::env::current_dir()?;
        Ok(Self {
            project_root: resolve_project_root(&cwd),
        })
    }

    pub fn from_project_root(project_root: PathBuf) -> Self {
        Self { project_root }
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

pub fn resolve_project_root(start: &Path) -> PathBuf {
    start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| start.to_path_buf())
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
