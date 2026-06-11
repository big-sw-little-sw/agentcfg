//! Core workflow API for agentcfg.

mod client;
mod clients;
mod config_doc;
mod init;
mod locations;
mod project_anchor;

use std::path::PathBuf;

use serde::Serialize;

pub use client::{all_clients, parse_client_name, Client};
pub use clients::{
    clients_add, clients_remove, clients_set, clients_show, resolve_mutation_layer,
    ClientsAddRequest, ClientsLayerReport, ClientsMutationData, ClientsRemoveRequest,
    ClientsSetRequest, ClientsShowData, ClientsShowRequest,
};
pub use config_doc::{
    read_default_clients, write_default_clients, ConfigDocError, PersistedClientSelection,
    SCHEMA_VERSION,
};
pub use init::{init, InitData, InitRequest};
pub use locations::{
    active_config_layers, build_workflow_context, discover_project_root, has_project_markers,
    is_project_marker_root, layer_label, layer_relative_path_label, persisted_config_layer_value,
    user_config_path, DiscoveredProjectRoot, ProjectAnchorSource, ProjectRootError,
    UserConfigPathError, WorkflowContext,
};
pub use project_anchor::{project_anchor_blocker, project_unanchored_diagnostic};

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
    pub context: WorkflowContext,
}

impl ConfigShowRequest {
    pub fn project(context: WorkflowContext) -> Self {
        Self {
            install_level: InstallLevel::Project,
            context,
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
    User,
}

pub fn config_show(request: ConfigShowRequest) -> WorkflowResult<ConfigShowData> {
    let layers = active_config_layers(request.install_level);
    let mut blockers = Vec::new();
    let mut diagnostics = Vec::new();
    let mut config_layers = Vec::new();

    if request.install_level == InstallLevel::Project {
        if let Some(diagnostic) = project_unanchored_diagnostic(&request.context) {
            diagnostics.push(diagnostic);
        }
    }

    for layer in layers {
        match request.context.config_layer_path(layer) {
            Ok(path) => config_layers.push(ConfigLayerReport {
                id: layer,
                name: layer_label(layer),
                state: config_layer_state(&path),
                path,
            }),
            Err(error) => blockers.push(user_config_path_blocker(error)),
        }
    }

    WorkflowResult {
        workflow: "config_show",
        status: WorkflowStatus::Success,
        diagnostics,
        blockers,
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: ConfigShowData {
            install_level: request.install_level,
            config_layers,
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

fn user_config_path_blocker(error: UserConfigPathError) -> Diagnostic {
    Diagnostic {
        code: "user-config-path-unresolved".to_string(),
        message: format!("Cannot resolve User Config path: {error}"),
        context: vec![(
            "config-layer".to_string(),
            layer_relative_path_label(ConfigLayerId::User).to_string(),
        )],
        suggested_actions: Vec::new(),
    }
}
