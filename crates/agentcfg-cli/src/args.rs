use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "agentcfg")]
#[command(about = "Manage agent configuration as repeatable desired state")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliCommand {
    #[command(about = "Create an agentcfg config file")]
    Init(InitArgs),

    #[command(name = "preview", about = "Preview configured changes without writing them")]
    Preview(PreviewArgs),

    #[command(about = "Apply configured changes")]
    Sync(SyncArgs),

    #[command(about = "Remove stale managed artifacts")]
    Prune(InstallScopeArgs),

    #[command(about = "Show managed install state")]
    Status(InstallScopeArgs),

    #[command(about = "Check local configuration and environment")]
    Doctor,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub(crate) struct InitArgs {
    #[arg(long)]
    pub(crate) project: bool,

    #[arg(long)]
    pub(crate) user: bool,
}

#[derive(Args, Debug)]
pub(crate) struct PreviewArgs {
    #[arg(long)]
    pub(crate) user: bool,

    #[arg(long)]
    pub(crate) upgrade: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SyncArgs {
    #[arg(long)]
    pub(crate) user: bool,

    #[arg(long)]
    pub(crate) upgrade: bool,
}

#[derive(Args, Debug)]
pub(crate) struct InstallScopeArgs {
    #[arg(long)]
    pub(crate) user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_supported_command_forms() {
        for args in [
            ["agentcfg", "init"].as_slice(),
            ["agentcfg", "init", "--project"].as_slice(),
            ["agentcfg", "init", "--user"].as_slice(),
            ["agentcfg", "preview"].as_slice(),
            ["agentcfg", "preview", "--upgrade"].as_slice(),
            ["agentcfg", "preview", "--user"].as_slice(),
            ["agentcfg", "preview", "--user", "--upgrade"].as_slice(),
            ["agentcfg", "sync"].as_slice(),
            ["agentcfg", "sync", "--upgrade"].as_slice(),
            ["agentcfg", "sync", "--user"].as_slice(),
            ["agentcfg", "sync", "--user", "--upgrade"].as_slice(),
            ["agentcfg", "prune"].as_slice(),
            ["agentcfg", "prune", "--user"].as_slice(),
            ["agentcfg", "status"].as_slice(),
            ["agentcfg", "status", "--user"].as_slice(),
            ["agentcfg", "doctor"].as_slice(),
        ] {
            Cli::try_parse_from(args).unwrap_or_else(|error| {
                panic!("expected {args:?} to parse, got {error}");
            });
        }
    }

    #[test]
    fn rejects_unsupported_command_forms() {
        for args in [
            ["agentcfg", "init", "--project", "--user"].as_slice(),
            ["agentcfg", "init", "--upgrade"].as_slice(),
            ["agentcfg", "prune", "--upgrade"].as_slice(),
            ["agentcfg", "status", "--upgrade"].as_slice(),
            ["agentcfg", "doctor", "--user"].as_slice(),
            ["agentcfg", "doctor", "--upgrade"].as_slice(),
            ["agentcfg", "plan"].as_slice(),
            ["agentcfg", "preview", "--project"].as_slice(),
            ["agentcfg", "sync", "--project"].as_slice(),
            ["agentcfg", "prune", "--project"].as_slice(),
            ["agentcfg", "status", "--project"].as_slice(),
        ] {
            assert!(
                Cli::try_parse_from(args).is_err(),
                "expected {args:?} to be rejected"
            );
        }
    }
}
