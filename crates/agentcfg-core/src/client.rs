//! Known V1 Client names for ConfigDoc parsing and config mutation validation.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub fn parse_client_name(name: &str) -> Result<Client, String> {
    all_clients()
        .iter()
        .copied()
        .find(|client| client.to_string() == name)
        .ok_or_else(|| format!("unsupported client: {name}"))
}

pub fn all_clients() -> &'static [Client] {
    &[
        Client::Codex,
        Client::Pi,
        Client::OpenCode,
        Client::ClaudeCode,
        Client::Cline,
        Client::Cursor,
    ]
}
