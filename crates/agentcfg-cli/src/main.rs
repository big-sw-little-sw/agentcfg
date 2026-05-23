use std::process::ExitCode;

const HELP: &str = "\
agentcfg

Usage: agentcfg [OPTIONS]

Options:
  -h, --help    Print help
";

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
    let show_help = std::env::args_os()
        .nth(1)
        .is_none_or(|arg| arg == "--help" || arg == "-h");

    if show_help {
        print!("{HELP}");
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn unexpected_errors_exit_one() {
        let error = CliError::Unexpected(anyhow::anyhow!("failed to write output"));

        assert_eq!(error.exit_code(), ExitCode::from(1));
    }
}
