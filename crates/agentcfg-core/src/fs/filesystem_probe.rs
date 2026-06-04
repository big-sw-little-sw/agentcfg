//! Probes path kind, symlink, writability, and directory facts.

use std::path::{Path, PathBuf};

use crate::AgentcfgResult;

/// Filesystem facts gathered without deciding command policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathFacts {
    Missing {
        path: PathBuf,
    },
    File {
        path: PathBuf,
        writable: bool,
    },
    Directory {
        path: PathBuf,
        writable: bool,
        empty: bool,
    },
    Symlink {
        path: PathBuf,
        target: PathBuf,
    },
    Other {
        path: PathBuf,
    },
}

pub fn probe(_path: &Path) -> AgentcfgResult<PathFacts> {
    unimplemented!("filesystem probing is not implemented yet")
}
