use agentcfg_core::{
    clients_add, clients_remove, clients_set, clients_show, ClientConfigLayerReport, ClientId,
    ClientsMutationRequest, ClientsMutationResultData, ClientsShowData, ClientsShowRequest,
    ConfigLayerId, InstallLevel, WorkflowResult, WorkflowStatus,
};
use clap::{Args, Subcommand, ValueEnum};

use crate::{render_json, CliError, CliResult, OutputFormat};

type ClientsMutationWorkflow =
    fn(ClientsMutationRequest) -> WorkflowResult<ClientsMutationResultData>;

#[derive(Debug, Subcommand)]
pub(crate) enum ClientsCommand {
    Show(ClientsShowArgs),
    Set(ClientsMutationArgs),
    Add(ClientsMutationArgs),
    Remove(ClientsMutationArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ClientsShowArgs {
    #[arg(long, value_enum, default_value = "project")]
    level: InstallLevelArg,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Args)]
pub(crate) struct ClientsMutationArgs {
    #[arg(value_name = "client", required = true)]
    clients: Vec<String>,
    #[arg(long, value_enum, default_value = "project")]
    level: InstallLevelArg,
    #[arg(long, value_enum)]
    config_layer: Option<ConfigLayerArg>,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum InstallLevelArg {
    Project,
    User,
}

impl From<InstallLevelArg> for InstallLevel {
    fn from(level: InstallLevelArg) -> Self {
        match level {
            InstallLevelArg::Project => Self::Project,
            InstallLevelArg::User => Self::User,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ConfigLayerArg {
    SharedProject,
    UserProject,
    User,
}

impl From<ConfigLayerArg> for ConfigLayerId {
    fn from(config_layer: ConfigLayerArg) -> Self {
        match config_layer {
            ConfigLayerArg::SharedProject => Self::SharedProject,
            ConfigLayerArg::UserProject => Self::UserProject,
            ConfigLayerArg::User => Self::User,
        }
    }
}

pub(crate) fn run(command: ClientsCommand) -> i32 {
    match command {
        ClientsCommand::Show(args) => run_clients_show(args),
        ClientsCommand::Set(args) => run_clients_mutation(args, clients_set),
        ClientsCommand::Add(args) => run_clients_mutation(args, clients_add),
        ClientsCommand::Remove(args) => run_clients_mutation(args, clients_remove),
    }
}

fn run_clients_show(args: ClientsShowArgs) -> i32 {
    match try_run_clients_show(args) {
        Ok(exit_code) => exit_code,
        Err(error) => error.print(),
    }
}

fn try_run_clients_show(args: ClientsShowArgs) -> CliResult<i32> {
    let request = build_clients_show_request(args.level)?;
    let result = clients_show(request);
    match args.format {
        OutputFormat::Text => print!("{}", render_clients_text(&result)),
        OutputFormat::Json => print!("{}", render_json(&result)),
    }
    Ok(exit_code(&result))
}

fn run_clients_mutation(args: ClientsMutationArgs, workflow: ClientsMutationWorkflow) -> i32 {
    match try_run_clients_mutation(args, workflow) {
        Ok(exit_code) => exit_code,
        Err(error) => error.print(),
    }
}

fn try_run_clients_mutation(
    args: ClientsMutationArgs,
    workflow: ClientsMutationWorkflow,
) -> CliResult<i32> {
    let clients = parse_clients(&args.clients)?;
    let request = build_clients_mutation_request(args.level, args.config_layer, clients)?;
    let result = workflow(request);
    match args.format {
        OutputFormat::Text => print!("{}", render_clients_mutation_text(&result)),
        OutputFormat::Json => print!("{}", render_json(&result)),
    }
    Ok(exit_code(&result))
}

fn render_clients_text(result: &WorkflowResult<ClientsShowData>) -> String {
    let mut output = String::new();
    output.push_str("Default Client Selection\n");
    output.push_str(&format!("Install Level: {}\n", result.data.install_level));
    output.push_str("Config Layers:\n");

    for layer in &result.data.config_layers {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            layer.name,
            default_clients_label(layer),
            layer.path.display()
        ));
    }

    output
}

fn render_clients_mutation_text(result: &WorkflowResult<ClientsMutationResultData>) -> String {
    let mut output = String::new();
    output.push_str("Default Client Selection updated\n");
    output.push_str(&format!(
        "Config Layer: {}\n",
        result.data.config_layer_id.label()
    ));
    output.push_str(&format!(
        "Default Clients: {}\n",
        client_list_label(&result.data.default_clients)
    ));
    for action in &result.suggested_actions {
        output.push_str(&format!("Next: {} ({})\n", action.command, action.reason));
    }
    output
}

fn exit_code<T>(result: &WorkflowResult<T>) -> i32 {
    match result.status {
        WorkflowStatus::Success => 0,
        WorkflowStatus::Blocked => 1,
    }
}

fn default_clients_label(layer: &ClientConfigLayerReport) -> String {
    client_list_label(&layer.default_clients)
}

fn client_list_label(clients: &[ClientId]) -> String {
    if clients.is_empty() {
        "none".to_string()
    } else {
        clients
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn parse_clients(values: &[String]) -> CliResult<Vec<ClientId>> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<ClientId>()
                .map_err(|()| CliError::invalid_input(format!("unknown Client `{value}`")))
        })
        .collect()
}

fn build_clients_show_request(level: InstallLevelArg) -> CliResult<ClientsShowRequest> {
    match InstallLevel::from(level) {
        InstallLevel::Project => Ok(ClientsShowRequest::for_project_cwd(current_dir()?)),
        InstallLevel::User => Ok(ClientsShowRequest::for_user_config_home(user_config_home()?)),
    }
}

fn build_clients_mutation_request(
    level: InstallLevelArg,
    config_layer: Option<ConfigLayerArg>,
    clients: Vec<ClientId>,
) -> CliResult<ClientsMutationRequest> {
    validate_config_layer_level(level, config_layer)?;
    let mut request = match InstallLevel::from(level) {
        InstallLevel::Project => ClientsMutationRequest::for_project_cwd(current_dir()?, clients),
        InstallLevel::User => {
            ClientsMutationRequest::for_user_config_home(user_config_home()?, clients)
        }
    };
    if let Some(config_layer) = config_layer {
        request = request.with_config_layer(config_layer.into());
    }

    Ok(request)
}

fn validate_config_layer_level(
    level: InstallLevelArg,
    config_layer: Option<ConfigLayerArg>,
) -> CliResult<()> {
    match (level, config_layer) {
        (InstallLevelArg::Project, Some(ConfigLayerArg::User)) => Err(CliError::invalid_input(
            "--config-layer user can only be used with --level user",
        )),
        (
            InstallLevelArg::User,
            Some(config_layer @ (ConfigLayerArg::SharedProject | ConfigLayerArg::UserProject)),
        ) => Err(CliError::invalid_input(format!(
            "--config-layer {} can only be used with --level project",
            ConfigLayerId::from(config_layer)
        ))),
        _ => Ok(()),
    }
}

fn current_dir() -> CliResult<std::path::PathBuf> {
    std::env::current_dir()
        .map_err(|error| CliError::runtime(format!("cannot determine current directory: {error}")))
}

fn user_config_home() -> CliResult<std::path::PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty())
    {
        return Ok(config_home.into());
    }

    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join(".config"))
        .ok_or_else(|| CliError::runtime("cannot determine User Config path: HOME is not set"))
}
