mod clients;

use agentcfg_core::{
    config_show, ConfigLayerId, ConfigLayerState, ConfigShowData, ConfigShowRequest, WorkflowResult,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

pub(crate) type CliResult<T> = Result<T, CliError>;

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
            ConfigCommand::Show(args) => run_config_show(args.format),
        },
        Command::Clients { command } => clients::run(command),
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
        command: clients::ClientsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show(ConfigShowArgs),
}

#[derive(Debug, Args)]
struct ConfigShowArgs {
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

fn run_config_show(format: OutputFormat) -> i32 {
    match try_run_config_show(format) {
        Ok(exit_code) => exit_code,
        Err(error) => error.print(),
    }
}

fn try_run_config_show(format: OutputFormat) -> CliResult<i32> {
    let project_root = current_dir()?;
    let result = config_show(ConfigShowRequest::for_project_root(project_root));
    match format {
        OutputFormat::Text => print!("{}", render_config_show_text(&result)),
        OutputFormat::Json => print!("{}", render_json(&result)),
    }
    Ok(0)
}

fn render_config_show_text(result: &WorkflowResult<ConfigShowData>) -> String {
    let mut output = String::new();
    output.push_str("Agent Configuration\n");
    output.push_str(&format!("Install Level: {}\n", result.data.install_level));
    output.push_str("Config Layers:\n");

    for layer in &result.data.config_layers {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            layer.name,
            config_layer_state_label(layer.state),
            config_layer_path_label(layer.id)
        ));
    }

    output
}

pub(crate) fn render_json<T: Serialize>(result: &WorkflowResult<T>) -> String {
    format!(
        "{}\n",
        serde_json::to_string(result).expect("workflow result serializes")
    )
}

fn config_layer_state_label(state: ConfigLayerState) -> &'static str {
    match state {
        ConfigLayerState::Missing => "missing",
        ConfigLayerState::Empty => "empty",
        ConfigLayerState::Authored => "authored",
    }
}

fn config_layer_path_label(id: ConfigLayerId) -> &'static str {
    match id {
        ConfigLayerId::SharedProject => "agentcfg.toml",
        ConfigLayerId::UserProject => ".agentcfg/agentcfg.toml",
        ConfigLayerId::User => "user config path",
    }
}

fn current_dir() -> CliResult<std::path::PathBuf> {
    std::env::current_dir()
        .map_err(|error| CliError::runtime(format!("cannot determine current directory: {error}")))
}

#[derive(Debug)]
pub(crate) struct CliError {
    message: String,
    exit_code: i32,
}

impl CliError {
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }

    pub(crate) fn runtime(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    pub(crate) fn print(self) -> i32 {
        eprintln!("error: {}", self.message);
        self.exit_code
    }
}
