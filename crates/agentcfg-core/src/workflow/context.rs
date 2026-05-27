use std::env;
use std::path::{Path, PathBuf};

use crate::config_paths::UserDirs;
use crate::{Error, Result};

/// Process and test context for workflow entrypoints.
///
/// Path resolution uses [`UserDirs`] for config home, state home, and user home
/// (for Client Discovery Location scans).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkflowContext {
    pub(crate) cwd: PathBuf,
    user_dirs: UserDirs,
}

impl WorkflowContext {
    pub(crate) fn from_process() -> Result<Self> {
        let cwd = env::current_dir().map_err(|source| Error::Io {
            path: PathBuf::from("."),
            source,
        })?;

        Ok(Self {
            cwd,
            user_dirs: UserDirs::from_env()?,
        })
    }

    pub(crate) fn new(cwd: impl Into<PathBuf>, user_dirs: UserDirs) -> Self {
        Self {
            cwd: cwd.into(),
            user_dirs,
        }
    }

    pub(crate) fn user_config_home(&self) -> PathBuf {
        self.user_dirs.config_home().to_path_buf()
    }

    pub(crate) fn home_dir(&self) -> Option<&Path> {
        self.user_dirs.home_dir()
    }
}
