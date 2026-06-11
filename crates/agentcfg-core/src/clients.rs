//! Default Client Selection workflows and V1 Client catalog.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::Serialize;

use crate::config_file::{read_agent_config, set_default_clients, write_agent_config};
use crate::config_layers::{
    config_layer_path, resolve_project_root, user_config_path, ConfigLayerId, ConfigLayerState,
    InstallLevel,
};
use crate::workflow::{Diagnostic, SuggestedAction, WorkflowName, WorkflowResult, WorkflowStatus};

const INSTALL_ACTION_REASON: &str = "Materialize changed Default Client Selection.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientId {
    Codex,
    Pi,
    Opencode,
    ClaudeCode,
    Cline,
    Cursor,
}

impl FromStr for ClientId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "codex" => Ok(Self::Codex),
            "pi" => Ok(Self::Pi),
            "opencode" => Ok(Self::Opencode),
            "claude-code" => Ok(Self::ClaudeCode),
            "cline" => Ok(Self::Cline),
            "cursor" => Ok(Self::Cursor),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Codex => "codex",
            Self::Pi => "pi",
            Self::Opencode => "opencode",
            Self::ClaudeCode => "claude-code",
            Self::Cline => "cline",
            Self::Cursor => "cursor",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsShowRequest {
    pub install_level: InstallLevel,
    pub project_root: PathBuf,
    pub user_config_home: Option<PathBuf>,
}

impl ClientsShowRequest {
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
pub struct ClientsShowData {
    pub install_level: InstallLevel,
    pub config_layers: Vec<ClientConfigLayerReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientConfigLayerReport {
    pub id: ConfigLayerId,
    pub name: &'static str,
    pub path: PathBuf,
    pub state: ConfigLayerState,
    pub default_clients: Vec<ClientId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientsMutationRequest {
    pub install_level: InstallLevel,
    pub project_root: PathBuf,
    pub user_config_home: Option<PathBuf>,
    pub config_layer_id: Option<ConfigLayerId>,
    pub clients: Vec<ClientId>,
}

impl ClientsMutationRequest {
    pub fn for_project_root(project_root: impl Into<PathBuf>, clients: Vec<ClientId>) -> Self {
        Self {
            install_level: InstallLevel::Project,
            project_root: project_root.into(),
            user_config_home: None,
            config_layer_id: None,
            clients,
        }
    }

    pub fn for_project_cwd(cwd: impl Into<PathBuf>, clients: Vec<ClientId>) -> Self {
        Self::for_project_root(resolve_project_root(cwd.into()), clients)
    }

    pub fn for_user_config_home(
        user_config_home: impl Into<PathBuf>,
        clients: Vec<ClientId>,
    ) -> Self {
        Self {
            install_level: InstallLevel::User,
            project_root: PathBuf::new(),
            user_config_home: Some(user_config_home.into()),
            config_layer_id: Some(ConfigLayerId::User),
            clients,
        }
    }

    pub fn with_config_layer(mut self, config_layer_id: ConfigLayerId) -> Self {
        self.config_layer_id = Some(config_layer_id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientsMutationResultData {
    pub install_level: InstallLevel,
    pub config_layer_id: ConfigLayerId,
    pub config_path: PathBuf,
    pub default_clients: Vec<ClientId>,
}

pub fn clients_show(request: ClientsShowRequest) -> WorkflowResult<ClientsShowData> {
    let (config_layers, blockers) = match request.install_level {
        InstallLevel::Project => {
            let shared_project_path =
                ConfigLayerId::SharedProject.agent_config_path(&request.project_root, None);
            let user_project_path =
                ConfigLayerId::UserProject.agent_config_path(&request.project_root, None);
            let (shared_project, mut blockers) =
                client_config_layer_report(ConfigLayerId::SharedProject, shared_project_path);
            let (user_project, user_project_blockers) =
                client_config_layer_report(ConfigLayerId::UserProject, user_project_path);
            blockers.extend(user_project_blockers);
            (vec![shared_project, user_project], blockers)
        }
        InstallLevel::User => {
            let (user_config, blockers) = client_config_layer_report(
                ConfigLayerId::User,
                user_config_path(
                    request
                        .user_config_home
                        .as_deref()
                        .expect("user config home"),
                ),
            );
            (vec![user_config], blockers)
        }
    };
    let status = if blockers.is_empty() {
        WorkflowStatus::Success
    } else {
        WorkflowStatus::Blocked
    };

    WorkflowResult {
        workflow: WorkflowName::ClientsShow,
        status,
        diagnostics: Vec::new(),
        blockers,
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: ClientsShowData {
            install_level: request.install_level,
            config_layers,
        },
    }
}

pub fn clients_set(request: ClientsMutationRequest) -> WorkflowResult<ClientsMutationResultData> {
    mutate_default_clients(request, WorkflowName::ClientsSet, |_, requested| requested)
}

pub fn clients_add(request: ClientsMutationRequest) -> WorkflowResult<ClientsMutationResultData> {
    mutate_default_clients(
        request,
        WorkflowName::ClientsAdd,
        |mut current, requested| {
            for client in requested {
                if !current.contains(&client) {
                    current.push(client);
                }
            }
            current
        },
    )
}

pub fn clients_remove(
    request: ClientsMutationRequest,
) -> WorkflowResult<ClientsMutationResultData> {
    mutate_default_clients(
        request,
        WorkflowName::ClientsRemove,
        |mut current, requested| {
            current.retain(|client| !requested.contains(client));
            current
        },
    )
}

fn mutate_default_clients(
    request: ClientsMutationRequest,
    workflow: WorkflowName,
    mutation: impl FnOnce(Vec<ClientId>, Vec<ClientId>) -> Vec<ClientId>,
) -> WorkflowResult<ClientsMutationResultData> {
    let config_layer_id = request
        .config_layer_id
        .unwrap_or(request.install_level.default_mutation_layer());
    let config_path = config_layer_path(
        request.install_level,
        config_layer_id,
        &request.project_root,
        request.user_config_home.as_deref(),
    );
    let read = read_agent_config(&config_path);
    let current_clients = read.config.default_clients.clone();
    let default_clients = mutation(current_clients, request.clients);
    let blockers = read.blockers;

    if blockers.is_empty() {
        let mut doc = read.doc;
        set_default_clients(&mut doc, &default_clients);
        write_agent_config(&config_path, doc).expect("write Agent Configuration File");
    }

    clients_mutation_result(
        workflow,
        request.install_level,
        config_layer_id,
        config_path,
        default_clients,
        blockers,
    )
}

fn clients_mutation_result(
    workflow: WorkflowName,
    install_level: InstallLevel,
    config_layer_id: ConfigLayerId,
    config_path: PathBuf,
    default_clients: Vec<ClientId>,
    blockers: Vec<Diagnostic>,
) -> WorkflowResult<ClientsMutationResultData> {
    let status = if blockers.is_empty() {
        WorkflowStatus::Success
    } else {
        WorkflowStatus::Blocked
    };
    let suggested_actions = if blockers.is_empty() {
        vec![SuggestedAction {
            command: format!("agentcfg install --level {install_level}"),
            reason: INSTALL_ACTION_REASON.to_string(),
        }]
    } else {
        Vec::new()
    };

    WorkflowResult {
        workflow,
        status,
        diagnostics: Vec::new(),
        blockers,
        suggested_actions,
        progress_events: Vec::new(),
        data: ClientsMutationResultData {
            install_level,
            config_layer_id,
            config_path,
            default_clients,
        },
    }
}

fn client_config_layer_report(
    id: ConfigLayerId,
    path: PathBuf,
) -> (ClientConfigLayerReport, Vec<Diagnostic>) {
    let read = read_agent_config(&path);

    (
        ClientConfigLayerReport {
            id,
            name: id.label(),
            path,
            state: read.state,
            default_clients: read.config.default_clients,
        },
        read.blockers,
    )
}
