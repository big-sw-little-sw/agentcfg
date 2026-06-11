//! Config Layer and Install Level resolution shared by workflows.

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Serialize;

use crate::workflow::{WorkflowName, WorkflowResult, WorkflowStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigShowRequest {
    pub install_level: InstallLevel,
    pub project_root: PathBuf,
    pub user_config_home: Option<PathBuf>,
}

impl ConfigShowRequest {
    pub fn for_project_root(project_root: impl Into<PathBuf>) -> Self {
        Self {
            install_level: InstallLevel::Project,
            project_root: project_root.into(),
            user_config_home: None,
        }
    }

    pub fn for_project_cwd(cwd: impl Into<PathBuf>) -> Self {
        Self::for_project_root(resolve_project_root(cwd.into()))
    }

    pub fn for_user_config_home(user_config_home: impl Into<PathBuf>) -> Self {
        Self {
            install_level: InstallLevel::User,
            project_root: PathBuf::new(),
            user_config_home: Some(user_config_home.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigShowData {
    pub install_level: InstallLevel,
    pub config_layers: Vec<ConfigLayerReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigLayerReport {
    pub id: ConfigLayerId,
    pub name: &'static str,
    pub path: PathBuf,
    pub state: ConfigLayerState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigLayerId {
    SharedProject,
    UserProject,
    User,
}

impl ConfigLayerId {
    pub fn label(self) -> &'static str {
        match self {
            Self::SharedProject => "Shared Project Config",
            Self::UserProject => "User Project Config",
            Self::User => "User Config",
        }
    }

    pub(crate) fn install_level(self) -> InstallLevel {
        match self {
            Self::SharedProject | Self::UserProject => InstallLevel::Project,
            Self::User => InstallLevel::User,
        }
    }

    pub(crate) fn agent_config_path(
        self,
        project_root: &Path,
        user_config_home: Option<&Path>,
    ) -> PathBuf {
        match self {
            Self::SharedProject => project_root.join("agentcfg.toml"),
            Self::UserProject => project_root.join(".agentcfg").join("agentcfg.toml"),
            Self::User => user_config_path(user_config_home.expect("user config home")),
        }
    }
}

impl fmt::Display for ConfigLayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SharedProject => "shared-project",
            Self::UserProject => "user-project",
            Self::User => "user",
        })
    }
}

impl FromStr for ConfigLayerId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "shared-project" => Ok(Self::SharedProject),
            "user-project" => Ok(Self::UserProject),
            "user" => Ok(Self::User),
            _ => Err(()),
        }
    }
}

/// Local Agent Configuration File state; later parsing slices can add authored, invalid, and unreadable states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigLayerState {
    Missing,
    Empty,
    Authored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLevel {
    Project,
    User,
}

impl InstallLevel {
    pub fn default_mutation_layer(self) -> ConfigLayerId {
        match self {
            Self::Project => ConfigLayerId::UserProject,
            Self::User => ConfigLayerId::User,
        }
    }
}

impl fmt::Display for InstallLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Project => "project",
            Self::User => "user",
        })
    }
}

impl FromStr for InstallLevel {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "project" => Ok(Self::Project),
            "user" => Ok(Self::User),
            _ => Err(()),
        }
    }
}

pub fn config_show(request: ConfigShowRequest) -> WorkflowResult<ConfigShowData> {
    let config_layers = config_layer_reports(
        request.install_level,
        &request.project_root,
        request.user_config_home.as_deref(),
    );

    WorkflowResult {
        workflow: WorkflowName::ConfigShow,
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers: Vec::new(),
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: ConfigShowData {
            install_level: request.install_level,
            config_layers,
        },
    }
}

pub(crate) fn config_layer_reports(
    install_level: InstallLevel,
    project_root: &Path,
    user_config_home: Option<&Path>,
) -> Vec<ConfigLayerReport> {
    match install_level {
        InstallLevel::Project => project_config_layers(project_root)
            .into_iter()
            .map(|(id, path)| ConfigLayerReport {
                id,
                name: id.label(),
                state: config_layer_state(&path),
                path,
            })
            .collect(),
        InstallLevel::User => {
            let path = ConfigLayerId::User.agent_config_path(project_root, user_config_home);
            vec![ConfigLayerReport {
                id: ConfigLayerId::User,
                name: ConfigLayerId::User.label(),
                state: config_layer_state(&path),
                path,
            }]
        }
    }
}

pub(crate) fn config_layer_path(
    install_level: InstallLevel,
    config_layer_id: ConfigLayerId,
    project_root: &Path,
    user_config_home: Option<&Path>,
) -> PathBuf {
    if config_layer_id.install_level() != install_level {
        panic!("Config Layer does not belong to Install Level");
    }

    config_layer_id.agent_config_path(project_root, user_config_home)
}

pub(crate) fn user_config_path(user_config_home: &Path) -> PathBuf {
    user_config_home.join("agentcfg").join("agentcfg.toml")
}

pub fn resolve_project_root(cwd: PathBuf) -> PathBuf {
    for ancestor in cwd.ancestors() {
        if ancestor.join(".git").exists() {
            return ancestor.to_path_buf();
        }
    }

    cwd
}

fn project_config_layers(project_root: &Path) -> Vec<(ConfigLayerId, PathBuf)> {
    vec![
        (
            ConfigLayerId::SharedProject,
            ConfigLayerId::SharedProject.agent_config_path(project_root, None),
        ),
        (
            ConfigLayerId::UserProject,
            ConfigLayerId::UserProject.agent_config_path(project_root, None),
        ),
    ]
}

fn config_layer_state(path: &Path) -> ConfigLayerState {
    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(contents) if contents.trim().is_empty() => ConfigLayerState::Empty,
            Ok(_) | Err(_) => ConfigLayerState::Authored,
        }
    } else {
        ConfigLayerState::Missing
    }
}
