use std::path::PathBuf;

use agentcfg_core::{
    build_workflow_context, clients_add, clients_remove, clients_set, clients_show, config_show,
    deselect_skill, init, layer_relative_path_label, parse_client_name, select_skill, Client,
    ClientsAddRequest, ClientsMutationData, ClientsRemoveRequest, ClientsSetRequest,
    ClientsShowData, ClientsShowRequest, ConfigLayerId, ConfigLayerState, ConfigShowData,
    ConfigShowRequest, DeselectSkillRequest, Diagnostic, InitData, InitRequest, InstallLevel,
    PersistedClientSelection, SelectSkillRequest, SkillMutationData, WorkflowContext,
    WorkflowResult,
};
use clap::{Args, Parser, Subcommand, ValueEnum};

fn main() {
    let exit_code = run();
    std::process::exit(exit_code);
}

fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = error.exit_code();
            if let Err(print_error) = error.print() {
                eprintln!("error: cannot print CLI error: {print_error}");
            }
            return exit_code;
        }
    };

    match cli.command {
        Command::Config { command } => match command {
            ConfigCommand::Show(args) => run_config_show(args),
        },
        Command::Clients { command } => match command {
            ClientsCommand::Show(args) => run_clients_show(args),
            ClientsCommand::Set(args) => run_clients_set(args),
            ClientsCommand::Add(args) => run_clients_add(args),
            ClientsCommand::Remove(args) => run_clients_remove(args),
        },
        Command::Init(args) => run_init(args),
        Command::Skills { command } => match command {
            SkillsCommand::Select(args) => run_skills_select(args),
            SkillsCommand::Deselect(args) => run_skills_deselect(args),
        },
    }
}

#[derive(Debug, Parser)]
#[command(name = "agentcfg")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Clients {
        #[command(subcommand)]
        command: ClientsCommand,
    },
    Init(InitArgs),
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show(ConfigShowArgs),
}

#[derive(Debug, Subcommand)]
enum SkillsCommand {
    Select(SkillsSelectArgs),
    Deselect(SkillsDeselectArgs),
}

#[derive(Debug, Subcommand)]
enum ClientsCommand {
    Show(ClientsLevelArgs),
    Set(ClientsMutationArgs),
    Add(ClientsMutationArgs),
    Remove(ClientsMutationArgs),
}

#[derive(Debug, Args)]
struct WorkflowArgs {
    #[arg(
        long,
        help = "Override Project Root discovery with an explicit directory. Project Root otherwise comes from git discovery, existing project markers, or init."
    )]
    project_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ConfigShowArgs {
    #[command(flatten)]
    workflow: WorkflowArgs,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct ClientsLevelArgs {
    #[command(flatten)]
    workflow: WorkflowArgs,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long, value_enum, default_value = "project")]
    level: LevelArg,
    #[arg(long, value_enum)]
    config_layer: Option<ConfigLayerArg>,
}

#[derive(Debug, Args)]
struct ClientsMutationArgs {
    #[command(flatten)]
    workflow: WorkflowArgs,
    #[arg(required = true)]
    clients: Vec<String>,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long, value_enum, default_value = "project")]
    level: LevelArg,
    #[arg(long, value_enum)]
    config_layer: Option<ConfigLayerArg>,
}

#[derive(Debug, Args)]
struct SkillsMutationArgs {
    #[command(flatten)]
    workflow: WorkflowArgs,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
    #[arg(long, value_enum, default_value = "project")]
    level: LevelArg,
    #[arg(long, value_enum)]
    config_layer: Option<ConfigLayerArg>,
    #[arg(
        long = "id",
        help = "Skill Configuration Entry Id for selecting or creating an entry."
    )]
    entry_id: Option<String>,
    #[arg(
        long,
        help = "Skill Source locator such as a local path, GitHub shorthand, or full git URL."
    )]
    source: Option<String>,
    #[arg(
        long = "ref",
        help = "Git Source Ref for git-backed Skill Sources such as GitHub shorthand or full git URLs."
    )]
    git_ref: Option<String>,
}

#[derive(Debug, Args)]
struct SkillsSelectArgs {
    #[command(flatten)]
    mutation: SkillsMutationArgs,
    #[arg(help = "Source Skill Name to add to Skill Selection.")]
    source_skill_name: String,
}

#[derive(Debug, Args)]
struct SkillsDeselectArgs {
    #[command(flatten)]
    mutation: SkillsMutationArgs,
    #[arg(help = "Source Skill Name to remove from Skill Selection.")]
    source_skill_name: String,
}

#[derive(Debug, Args)]
struct InitArgs {
    #[command(flatten)]
    workflow: WorkflowArgs,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LevelArg {
    Project,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ConfigLayerArg {
    #[value(name = "shared-project")]
    SharedProject,
    #[value(name = "user-project")]
    UserProject,
}

fn run_config_show(args: ConfigShowArgs) -> i32 {
    let context = match workflow_context(args.workflow.project_root) {
        Ok(context) => context,
        Err(code) => return code,
    };
    let result = config_show(ConfigShowRequest::project(context));
    render_workflow(args.format, &result, render_config_show_text)
}

fn run_clients_show(args: ClientsLevelArgs) -> i32 {
    let context = match workflow_context(args.workflow.project_root) {
        Ok(context) => context,
        Err(code) => return code,
    };
    let result = clients_show(ClientsShowRequest {
        install_level: args.level.into(),
        context,
        config_layer: args.config_layer.map(Into::into),
    });
    render_workflow(args.format, &result, render_clients_show_text)
}

fn run_clients_set(args: ClientsMutationArgs) -> i32 {
    run_clients_mutation(args, clients_set)
}

fn run_clients_add(args: ClientsMutationArgs) -> i32 {
    run_clients_mutation(args, |request| {
        clients_add(ClientsAddRequest {
            install_level: request.install_level,
            context: request.context,
            config_layer: request.config_layer,
            clients: request.clients,
        })
    })
}

fn run_clients_remove(args: ClientsMutationArgs) -> i32 {
    run_clients_mutation(args, |request| {
        clients_remove(ClientsRemoveRequest {
            install_level: request.install_level,
            context: request.context,
            config_layer: request.config_layer,
            clients: request.clients,
        })
    })
}

fn run_skills_select(args: SkillsSelectArgs) -> i32 {
    let context = match workflow_context(args.mutation.workflow.project_root) {
        Ok(context) => context,
        Err(code) => return code,
    };

    let result = select_skill(SelectSkillRequest {
        install_level: args.mutation.level.into(),
        context,
        config_layer: args.mutation.config_layer.map(Into::into),
        source_skill_name: args.source_skill_name,
        entry_id: args.mutation.entry_id,
        source: args.mutation.source,
        git_ref: args.mutation.git_ref,
    });
    render_workflow(args.mutation.format, &result, render_select_skill_text)
}

fn run_skills_deselect(args: SkillsDeselectArgs) -> i32 {
    let context = match workflow_context(args.mutation.workflow.project_root) {
        Ok(context) => context,
        Err(code) => return code,
    };

    let result = deselect_skill(DeselectSkillRequest {
        install_level: args.mutation.level.into(),
        context,
        config_layer: args.mutation.config_layer.map(Into::into),
        source_skill_name: args.source_skill_name,
        entry_id: args.mutation.entry_id,
        source: args.mutation.source,
        git_ref: args.mutation.git_ref,
    });
    render_workflow(args.mutation.format, &result, render_deselect_skill_text)
}

fn run_init(args: InitArgs) -> i32 {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("error: cannot determine current directory: {error}");
            return 1;
        }
    };

    let result = init(InitRequest {
        cwd,
        explicit_project_root: args.workflow.project_root,
    });
    render_workflow(args.format, &result, render_init_text)
}

fn run_clients_mutation<F>(args: ClientsMutationArgs, workflow: F) -> i32
where
    F: FnOnce(ClientsSetRequest) -> WorkflowResult<ClientsMutationData>,
{
    let context = match workflow_context(args.workflow.project_root) {
        Ok(context) => context,
        Err(code) => return code,
    };

    let clients = match parse_clients(&args.clients) {
        Ok(clients) => clients,
        Err(message) => {
            eprintln!("error: {message}");
            return 1;
        }
    };

    let request = ClientsSetRequest {
        install_level: args.level.into(),
        context,
        config_layer: args.config_layer.map(Into::into),
        clients,
    };

    let result = workflow(request);
    render_workflow(args.format, &result, render_clients_mutation_text)
}

fn parse_clients(names: &[String]) -> Result<Vec<Client>, String> {
    names.iter().map(|name| parse_client_name(name)).collect()
}

fn workflow_context(project_root: Option<PathBuf>) -> Result<WorkflowContext, i32> {
    let cwd = std::env::current_dir().map_err(|error| {
        eprintln!("error: cannot determine current directory: {error}");
        1
    })?;
    build_workflow_context(cwd, project_root).map_err(|error| {
        eprintln!("error: {error}");
        1
    })
}

fn render_workflow<T>(
    format: OutputFormat,
    result: &WorkflowResult<T>,
    render_text: fn(&WorkflowResult<T>) -> String,
) -> i32
where
    T: serde::Serialize,
{
    if !result.blockers.is_empty() {
        match format {
            OutputFormat::Text => {
                for blocker in &result.blockers {
                    eprintln!("error: {}", blocker.message);
                    render_blocker_suggestions(blocker);
                }
            }
            OutputFormat::Json => print!("{}", render_json(result)),
        }
        return 1;
    }

    match format {
        OutputFormat::Text => print!("{}", render_text(result)),
        OutputFormat::Json => print!("{}", render_json(result)),
    }
    0
}

fn render_blocker_suggestions(blocker: &Diagnostic) {
    for action in &blocker.suggested_actions {
        eprintln!("hint: {} — {}", action.command, action.reason);
    }
}

fn render_config_show_text(result: &WorkflowResult<ConfigShowData>) -> String {
    let mut output = render_diagnostics_text(&result.diagnostics);
    output.push_str("Agent Configuration\n");
    output.push_str(&format!(
        "Install Level: {}\n",
        install_level_label(result.data.install_level)
    ));
    output.push_str("Config Layers:\n");

    for layer in &result.data.config_layers {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            layer.name,
            config_layer_state_label(layer.state),
            layer_relative_path_label(layer.id)
        ));
    }

    output
}

fn render_clients_show_text(result: &WorkflowResult<ClientsShowData>) -> String {
    let mut output = render_diagnostics_text(&result.diagnostics);
    output.push_str("Default Client Selection\n");
    output.push_str(&format!(
        "Install Level: {}\n",
        install_level_label(result.data.install_level)
    ));
    output.push_str("Config Layers:\n");

    for layer in &result.data.config_layers {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            layer.name,
            default_clients_label(layer.default_clients.as_ref()),
            layer_relative_path_label(layer.id)
        ));
    }

    output
}

fn render_select_skill_text(result: &WorkflowResult<SkillMutationData>) -> String {
    let mut output = String::new();
    output.push_str("Skill selected\n");
    output.push_str(&format!(
        "Install Level: {}\n",
        install_level_label(result.data.install_level)
    ));
    output.push_str(&format!(
        "Config Layer: {}\n",
        result.data.config_layer.name
    ));
    if let Some(entry_id) = &result.data.entry_id {
        output.push_str(&format!("Entry Id: {entry_id}\n"));
    }
    output.push_str(&format!("Skill Source: {}\n", result.data.source));
    if let Some(git_ref) = &result.data.git_ref {
        output.push_str(&format!("Git Source Ref: {git_ref}\n"));
    }
    output.push_str(&format!(
        "Source Skill Name: {}\n",
        result.data.source_skill_name
    ));
    output.push_str(&format!(
        "Clients: {}\n",
        default_clients_label(Some(&result.data.clients))
    ));
    output.push_str("Change client selection: agentcfg skills clients ...\n");

    if result.data.changed {
        for action in &result.suggested_actions {
            output.push_str(&format!("Next: {} — {}\n", action.command, action.reason));
        }
    }

    output
}

fn render_deselect_skill_text(result: &WorkflowResult<SkillMutationData>) -> String {
    let mut output = String::new();
    output.push_str("Skill deselected\n");
    output.push_str(&format!(
        "Install Level: {}\n",
        install_level_label(result.data.install_level)
    ));
    output.push_str(&format!(
        "Config Layer: {}\n",
        result.data.config_layer.name
    ));
    if let Some(entry_id) = &result.data.entry_id {
        output.push_str(&format!("Entry Id: {entry_id}\n"));
    }
    output.push_str(&format!("Skill Source: {}\n", result.data.source));
    if let Some(git_ref) = &result.data.git_ref {
        output.push_str(&format!("Git Source Ref: {git_ref}\n"));
    }
    output.push_str(&format!(
        "Source Skill Name: {}\n",
        result.data.source_skill_name
    ));

    if result.data.changed {
        for action in &result.suggested_actions {
            output.push_str(&format!("Next: {} — {}\n", action.command, action.reason));
        }
    }

    output
}

fn render_clients_mutation_text(result: &WorkflowResult<ClientsMutationData>) -> String {
    let mut output = String::new();
    output.push_str("Default Client Selection updated\n");
    output.push_str(&format!(
        "Install Level: {}\n",
        install_level_label(result.data.install_level)
    ));
    output.push_str(&format!(
        "Config Layer: {}\n",
        result.data.config_layer.name
    ));
    output.push_str(&format!(
        "Clients: {}\n",
        default_clients_label(Some(&result.data.default_clients))
    ));

    if result.data.changed {
        for action in &result.suggested_actions {
            output.push_str(&format!("Next: {} — {}\n", action.command, action.reason));
        }
    }

    output
}

fn render_init_text(result: &WorkflowResult<InitData>) -> String {
    let mut output = String::new();
    output.push_str("Project initialized\n");
    output.push_str(&format!(
        "Project Root: {}\n",
        result.data.project_root.display()
    ));
    if result.data.created_markers {
        output.push_str("Project Markers: created\n");
    } else {
        output.push_str("Project Markers: already present\n");
    }
    output
}

fn render_diagnostics_text(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();
    for diagnostic in diagnostics {
        output.push_str(&format!("note: {}\n", diagnostic.message));
        for action in &diagnostic.suggested_actions {
            output.push_str(&format!("hint: {} — {}\n", action.command, action.reason));
        }
    }
    output
}

fn render_json<T: serde::Serialize>(result: &WorkflowResult<T>) -> String {
    format!(
        "{}\n",
        serde_json::to_string(result).expect("workflow result serializes")
    )
}

fn install_level_label(install_level: InstallLevel) -> &'static str {
    match install_level {
        InstallLevel::Project => "project",
        InstallLevel::User => "user",
    }
}

fn config_layer_state_label(state: ConfigLayerState) -> &'static str {
    match state {
        ConfigLayerState::Missing => "missing",
        ConfigLayerState::Empty => "empty",
    }
}

fn default_clients_label(clients: Option<&PersistedClientSelection>) -> String {
    match clients {
        None => "none".to_string(),
        Some(PersistedClientSelection::All) => "all".to_string(),
        Some(PersistedClientSelection::Explicit(values)) if values.is_empty() => "none".to_string(),
        Some(PersistedClientSelection::Explicit(values)) => values
            .iter()
            .map(|client| client.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    }
}

impl From<LevelArg> for InstallLevel {
    fn from(level: LevelArg) -> Self {
        match level {
            LevelArg::Project => Self::Project,
            LevelArg::User => Self::User,
        }
    }
}

impl From<ConfigLayerArg> for ConfigLayerId {
    fn from(layer: ConfigLayerArg) -> Self {
        match layer {
            ConfigLayerArg::SharedProject => Self::SharedProject,
            ConfigLayerArg::UserProject => Self::UserProject,
        }
    }
}
