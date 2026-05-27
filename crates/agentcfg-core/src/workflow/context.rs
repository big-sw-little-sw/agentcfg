use std::env;
use std::path::{Path, PathBuf};

use crate::config_paths::UserDirs;
use crate::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkflowContext {
    pub(crate) cwd: PathBuf,
    xdg_config_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
}

impl WorkflowContext {
    pub(crate) fn from_process() -> Result<Self> {
        let cwd = env::current_dir().map_err(|source| Error::Io {
            path: PathBuf::from("."),
            source,
        })?;

        Ok(Self::new(
            cwd,
            env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            env::var_os("HOME").map(PathBuf::from),
        ))
    }

    pub(crate) fn new(
        cwd: impl Into<PathBuf>,
        xdg_config_home: Option<impl Into<PathBuf>>,
        home_dir: Option<impl Into<PathBuf>>,
    ) -> Self {
        Self {
            cwd: cwd.into(),
            xdg_config_home: xdg_config_home.map(Into::into),
            home_dir: home_dir.map(Into::into),
        }
    }

    pub(crate) fn user_config_home(&self) -> Result<PathBuf> {
        UserDirs::config_home_from_env_vars(self.xdg_config_home.clone(), self.home_dir.clone())
    }

    pub(crate) fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }
}
