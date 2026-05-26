use std::process::ExitCode;

#[derive(Debug)]
pub enum CliError {
    Core(agentcfg_core::Error),
    Usage(clap::Error),
    Unexpected(std::io::Error),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::Core(_) | CliError::Unexpected(_) => ExitCode::from(1),
            CliError::Usage(_) => ExitCode::from(2),
        }
    }

    pub fn print(&self) -> std::io::Result<()> {
        match self {
            CliError::Core(error) => {
                eprintln!("error: {error}");
                Ok(())
            }
            CliError::Usage(error) => error.print(),
            CliError::Unexpected(error) => {
                eprintln!("error: {error}");
                Ok(())
            }
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
        Self::Usage(error)
    }
}
