//! Core contracts and workflow modules for managing repeatable Agent Configuration.

pub mod client_registry;
pub mod config;
pub mod content_digest;
pub mod execution;
pub mod fs;
pub mod installation;
pub mod lockfile;
pub mod manifest;
pub mod planning;
pub mod resolution;
pub mod stores;
pub mod workflow;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type AgentcfgResult<T> = Result<T, AgentcfgError>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgentcfgError {}

// Keep plain string domain wrappers consistent without repeating constructor/accessor glue.
macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Client {
    Codex,
    Pi,
    OpenCode,
    ClaudeCode,
    Cline,
    Cursor,
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Codex => "codex",
            Self::Pi => "pi",
            Self::OpenCode => "open-code",
            Self::ClaudeCode => "claude-code",
            Self::Cline => "cline",
            Self::Cursor => "cursor",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientSelection {
    AllSupported,
    Explicit(Vec<Client>),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigLayerKind {
    SharedProject,
    UserProject,
    User,
}

impl fmt::Display for ConfigLayerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SharedProject => "shared-project",
            Self::UserProject => "user-project",
            Self::User => "user",
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLevel {
    Project,
    User,
}

impl fmt::Display for InstallLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Project => "project",
            Self::User => "user",
        })
    }
}

string_newtype!(SourceSkillName);
string_newtype!(DiscoveryName);
string_newtype!(ConfigSourceId);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ClientDiscoveryLocation(PathBuf);

impl ClientDiscoveryLocation {
    pub fn new(value: impl Into<PathBuf>) -> Self {
        Self(value.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for ClientDiscoveryLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path().display())
    }
}

string_newtype!(TreeDigest);
