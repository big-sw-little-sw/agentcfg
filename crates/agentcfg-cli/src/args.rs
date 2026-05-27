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

    #[command(
        name = "preview",
        about = "Preview configured changes without writing them",
        long_about = "Read-only preview of changes Apply would make from Locked Desired State. \
                      Never writes config, lockfiles, the Manifest, Managed State, Skill Sources, \
                      or Client Discovery Locations."
    )]
    Preview(PreviewArgs),

    #[command(
        name = "apply",
        about = "Apply Locked Desired State into Managed State and Client Discovery Locations",
        long_about = "Apply Locked Desired State from active Config Layers into Managed State \
                      (including Managed Skill Content) and Client Discovery Locations. \
                      Creates missing lockfiles when needed. Never writes back to Skill Sources."
    )]
    Apply(ApplyArgs),

    #[command(
        about = "Remove Stale Installed Artifacts and Stale Discovery Requirements from managed state"
    )]
    Prune(InstallLevelArgs),

    #[command(about = "Report managed install-state consistency for an Install Level")]
    Status(InstallLevelArgs),

    #[command(about = "Check environment and configuration readiness")]
    Doctor,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub(crate) struct InitArgs {
    #[arg(long)]
    pub(crate) project: bool,

    #[arg(
        long,
        help = "Select User Config (User Config Layer) instead of the default User Project Config"
    )]
    pub(crate) user: bool,
}

#[derive(Args, Debug)]
pub(crate) struct PreviewArgs {
    #[arg(
        long,
        help = "Run at User Level (user Install Level) instead of the default Project Level"
    )]
    pub(crate) user: bool,

    #[arg(
        long = "refresh-sources",
        long_help = "Perform Source Refresh: refresh Skill Source resolutions before previewing Locked Desired State"
    )]
    pub(crate) refresh_sources: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ApplyArgs {
    #[arg(
        long,
        help = "Run at User Level (user Install Level) instead of the default Project Level"
    )]
    pub(crate) user: bool,

    #[arg(
        long = "refresh-sources",
        long_help = "Perform Source Refresh: refresh Skill Source resolutions before applying Locked Desired State"
    )]
    pub(crate) refresh_sources: bool,
}

#[derive(Args, Debug)]
pub(crate) struct InstallLevelArgs {
    #[arg(
        long,
        help = "Run at User Level (user Install Level) instead of the default Project Level"
    )]
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
            ["agentcfg", "preview", "--refresh-sources"].as_slice(),
            ["agentcfg", "preview", "--user"].as_slice(),
            ["agentcfg", "preview", "--user", "--refresh-sources"].as_slice(),
            ["agentcfg", "apply"].as_slice(),
            ["agentcfg", "apply", "--refresh-sources"].as_slice(),
            ["agentcfg", "apply", "--user"].as_slice(),
            ["agentcfg", "apply", "--user", "--refresh-sources"].as_slice(),
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
            ["agentcfg", "init", "--refresh-sources"].as_slice(),
            ["agentcfg", "preview", "--upgrade"].as_slice(),
            ["agentcfg", "apply", "--upgrade"].as_slice(),
            ["agentcfg", "prune", "--upgrade"].as_slice(),
            ["agentcfg", "prune", "--refresh-sources"].as_slice(),
            ["agentcfg", "status", "--upgrade"].as_slice(),
            ["agentcfg", "status", "--refresh-sources"].as_slice(),
            ["agentcfg", "doctor", "--user"].as_slice(),
            ["agentcfg", "doctor", "--upgrade"].as_slice(),
            ["agentcfg", "doctor", "--refresh-sources"].as_slice(),
            ["agentcfg", "plan"].as_slice(),
            ["agentcfg", "sync"].as_slice(),
            ["agentcfg", "preview", "--project"].as_slice(),
            ["agentcfg", "apply", "--project"].as_slice(),
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
