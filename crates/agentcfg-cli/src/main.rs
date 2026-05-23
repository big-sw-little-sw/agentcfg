use std::process::ExitCode;

use clap::Parser;
use clap::error::ErrorKind;

mod args;
mod commands;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
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
            error
                .print()
                .map_err(|error| CliError::Unexpected(anyhow::Error::from(error)))?;
            Ok(None)
        }
        Err(error) => Err(CliError::from(error)),
    }
}

#[derive(Debug)]
pub enum CliError {
    Core(agentcfg_core::Error),
    Usage { message: String },
    Unexpected(anyhow::Error),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Core(_) | CliError::Unexpected(_) => ExitCode::from(1),
            CliError::Usage { .. } => ExitCode::from(2),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Core(error) => write!(f, "error: {error}"),
            CliError::Usage { message } => write!(f, "usage error: {message}"),
            CliError::Unexpected(error) => write!(f, "error: {error}"),
        }
    }
}

impl From<agentcfg_core::Error> for CliError {
    fn from(error: agentcfg_core::Error) -> Self {
        Self::Core(error)
    }
}

impl From<clap::Error> for CliError {
    fn from(error: clap::Error) -> Self {
        Self::Usage {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn core_errors_exit_one() {
        let error = CliError::from(agentcfg_core::Error::from(
            agentcfg_core::ConfigError::MissingRequiredField { field: "scope" },
        ));

        assert_eq!(error.exit_code(), ExitCode::from(1));
    }

    #[test]
    fn usage_errors_exit_two() {
        let error = CliError::Usage {
            message: "unknown option `--bogus`".to_string(),
        };

        assert_eq!(error.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn clap_errors_exit_two_after_mapping() {
        let error = args::Cli::try_parse_from(["agentcfg", "doctor", "--user"]).unwrap_err();
        let cli_error = CliError::from(error);

        assert!(matches!(cli_error, CliError::Usage { .. }));
        assert_eq!(cli_error.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn unexpected_errors_exit_one() {
        let error = CliError::Unexpected(anyhow::anyhow!("failed to write output"));

        assert_eq!(error.exit_code(), ExitCode::from(1));
    }
}
