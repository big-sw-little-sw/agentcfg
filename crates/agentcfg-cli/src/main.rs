use agentcfg_core::{
    config_show, ConfigLayerId, ConfigLayerState, ConfigShowData, ConfigShowRequest, InstallLevel,
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
            ConfigCommand::Show(args) => run_config_show(args.format),
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
enum OutputFormat {
    Text,
    Json,
}

fn run_config_show(format: OutputFormat) -> i32 {
    let project_root = match std::env::current_dir() {
        Ok(project_root) => project_root,
        Err(error) => {
            eprintln!("error: cannot determine current directory: {error}");
            return 1;
        }
    };
    let result = config_show(ConfigShowRequest::project(project_root));
    match format {
        OutputFormat::Text => print!("{}", render_text(&result)),
        OutputFormat::Json => print!("{}", render_json(&result)),
    }
    0
}

fn render_text(result: &WorkflowResult<ConfigShowData>) -> String {
    let mut output = String::new();
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
            config_layer_path_label(layer.id)
        ));
    }

    output
}

fn render_json(result: &WorkflowResult<ConfigShowData>) -> String {
    format!(
        "{}\n",
        serde_json::to_string(result).expect("config show result serializes")
    )
}

fn install_level_label(install_level: InstallLevel) -> &'static str {
    match install_level {
        InstallLevel::Project => "project",
    }
}

fn config_layer_state_label(state: ConfigLayerState) -> &'static str {
    match state {
        ConfigLayerState::Missing => "missing",
        ConfigLayerState::Empty => "empty",
    }
}

fn config_layer_path_label(id: ConfigLayerId) -> &'static str {
    match id {
        ConfigLayerId::SharedProject => "agentcfg.toml",
        ConfigLayerId::UserProject => ".agentcfg/agentcfg.toml",
    }
}
