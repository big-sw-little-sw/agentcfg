use std::process::ExitCode;

use clap::Parser;
use clap::error::ErrorKind;

mod args;
mod commands;
mod error;
mod render;

use error::CliError;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = error.print();
            error.exit_code()
        }
    }
}

fn run() -> Result<(), CliError> {
    let Some(cli) = parse_cli()? else {
        return Ok(());
    };

    commands::handle(cli)
}

fn parse_cli() -> Result<Option<args::Cli>, CliError> {
    match args::Cli::try_parse() {
        Ok(cli) => Ok(Some(cli)),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error.print().map_err(CliError::Unexpected)?;
            Ok(None)
        }
        Err(error) => Err(CliError::from(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn core_errors_exit_one() {
        let error = CliError::from(agentcfg_core::Error::from(
            agentcfg_core::ConfigError::MissingRequiredField {
                path: "agentcfg.toml".into(),
                layer: agentcfg_core::workflow::ConfigLayer::SharedProject,
                field: "scope",
            },
        ));

        assert_eq!(error.exit_code(), ExitCode::from(1));
    }

    #[test]
    fn usage_errors_exit_two() {
        let error = CliError::Usage(clap::Error::raw(
            clap::error::ErrorKind::UnknownArgument,
            "unknown option `--bogus`",
        ));

        assert_eq!(error.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn clap_errors_exit_two_after_mapping() {
        let error = args::Cli::try_parse_from(["agentcfg", "doctor", "--user"]).unwrap_err();
        let cli_error = CliError::from(error);

        assert!(matches!(cli_error, CliError::Usage(_)));
        assert_eq!(cli_error.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn unexpected_errors_exit_one() {
        let error = CliError::Unexpected(std::io::Error::other("failed to write output"));

        assert_eq!(error.exit_code(), ExitCode::from(1));
    }
}
