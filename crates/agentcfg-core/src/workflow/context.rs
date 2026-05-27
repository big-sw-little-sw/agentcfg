use std::env;
use std::path::{Path, PathBuf};

use crate::config_paths::UserDirs;
use crate::{Error, Result};

/// Process and test context for workflow entrypoints.
///
/// Path resolution for config and state homes uses [`UserDirs`]; `HOME` is read only
/// for user-level Client Discovery Location scans when no explicit home is injected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkflowContext {
    pub(crate) cwd: PathBuf,
    user_dirs: UserDirs,
    home_dir: Option<PathBuf>,
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
            home_dir: env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| !path.as_os_str().is_empty()),
        })
    }

    pub(crate) fn new(
        cwd: impl Into<PathBuf>,
        user_dirs: UserDirs,
        home_dir: Option<impl Into<PathBuf>>,
    ) -> Self {
        Self {
            cwd: cwd.into(),
            user_dirs,
            home_dir: home_dir.map(Into::into),
        }
    }

    pub(crate) fn user_config_home(&self) -> PathBuf {
        self.user_dirs.config_home().to_path_buf()
    }

    pub(crate) fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }
}
