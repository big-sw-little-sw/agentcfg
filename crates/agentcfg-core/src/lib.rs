//! Core workflow API for agentcfg.

mod clients;
mod config_file;
mod config_layers;
mod workflow;

pub use clients::{
    clients_add, clients_remove, clients_set, clients_show, ClientConfigLayerReport, ClientId,
    ClientsMutationRequest, ClientsMutationResultData, ClientsShowData, ClientsShowRequest,
};
pub use config_layers::{
    config_show, resolve_project_root, ConfigLayerId, ConfigLayerReport, ConfigLayerState,
    ConfigShowData, ConfigShowRequest, InstallLevel,
};
pub use workflow::{
    Diagnostic, ProgressEvent, SuggestedAction, WorkflowName, WorkflowResult, WorkflowStatus,
};
