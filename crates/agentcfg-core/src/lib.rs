//! Core workflow API for agentcfg.

use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowResult<T> {
    pub workflow: &'static str,
    pub status: WorkflowStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub blockers: Vec<Diagnostic>,
    /// Result-level follow-up steps that are useful even when no Diagnostic caused them.
    pub suggested_actions: Vec<SuggestedAction>,
    /// Events emitted during workflow execution and retained for non-streaming presentations.
    pub progress_events: Vec<ProgressEvent>,
    pub data: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStatus {
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub context: Vec<(String, String)>,
    /// Follow-up steps tied to this specific Diagnostic.
    pub suggested_actions: Vec<SuggestedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuggestedAction {
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressEvent {
    pub phase: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigShowRequest {
    pub install_level: InstallLevel,
    pub project_root: PathBuf,
}

impl ConfigShowRequest {
    pub fn project(project_root: impl Into<PathBuf>) -> Self {
        Self {
            install_level: InstallLevel::Project,
            project_root: project_root.into(),
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
}

/// Local Agent Configuration File state; later parsing slices can add authored, invalid, and unreadable states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigLayerState {
    Missing,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLevel {
    Project,
}

pub fn config_show(request: ConfigShowRequest) -> WorkflowResult<ConfigShowData> {
    let shared_project_path = request.project_root.join("agentcfg.toml");
    let user_project_path = request.project_root.join(".agentcfg").join("agentcfg.toml");

    WorkflowResult {
        workflow: "config_show",
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers: Vec::new(),
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: ConfigShowData {
            install_level: request.install_level,
            config_layers: vec![
                ConfigLayerReport {
                    id: ConfigLayerId::SharedProject,
                    name: "Shared Project Config",
                    state: config_layer_state(&shared_project_path),
                    path: shared_project_path,
                },
                ConfigLayerReport {
                    id: ConfigLayerId::UserProject,
                    name: "User Project Config",
                    state: config_layer_state(&user_project_path),
                    path: user_project_path,
                },
            ],
        },
    }
}

fn config_layer_state(path: &std::path::Path) -> ConfigLayerState {
    if path.exists() {
        ConfigLayerState::Empty
    } else {
        ConfigLayerState::Missing
    }
}
