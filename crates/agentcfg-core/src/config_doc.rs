//! ConfigDoc parsing and Default Client Selection persistence for one Config Layer.

use std::fmt;
use std::path::Path;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use thiserror::Error;

use crate::client::Client;
use crate::locations::persisted_config_layer_value;
use crate::ConfigLayerId;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedClientSelection {
    All,
    Explicit(Vec<Client>),
}

impl Serialize for PersistedClientSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::All => serializer.serialize_str("all"),
            Self::Explicit(clients) => {
                let mut sequence = serializer.serialize_seq(Some(clients.len()))?;
                for client in clients {
                    sequence.serialize_element(client)?;
                }
                sequence.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for PersistedClientSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PersistedClientSelectionVisitor)
    }
}

struct PersistedClientSelectionVisitor;

impl<'de> Visitor<'de> for PersistedClientSelectionVisitor {
    type Value = PersistedClientSelection;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(r#""all" or a list of supported clients"#)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "all" => Ok(PersistedClientSelection::All),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut clients = Vec::new();
        while let Some(client) = sequence.next_element()? {
            clients.push(client);
        }
        Ok(PersistedClientSelection::Explicit(clients))
    }
}

#[derive(Debug, Error)]
pub enum ConfigDocError {
    #[error("cannot read config: {0}")]
    Read(#[from] std::io::Error),
    #[error("invalid config: {0}")]
    Invalid(String),
}

pub fn read_default_clients(
    path: &Path,
) -> Result<Option<PersistedClientSelection>, ConfigDocError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let table: toml::Table = toml::from_str(&content)
        .map_err(|error: toml::de::Error| ConfigDocError::Invalid(error.to_string()))?;

    let Some(clients_value) = table.get("clients") else {
        return Ok(None);
    };

    let selection: PersistedClientSelection = clients_value
        .clone()
        .try_into()
        .map_err(|error: toml::de::Error| ConfigDocError::Invalid(error.to_string()))?;

    Ok(Some(selection))
}

pub fn write_default_clients(
    path: &Path,
    layer: ConfigLayerId,
    clients: &PersistedClientSelection,
) -> Result<(), ConfigDocError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut table = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        content
            .parse::<toml::Table>()
            .map_err(|error: toml::de::Error| ConfigDocError::Invalid(error.to_string()))?
    } else {
        let mut table = toml::Table::new();
        table.insert(
            "version".to_string(),
            toml::Value::Integer(i64::from(SCHEMA_VERSION)),
        );
        table.insert(
            "config-layer".to_string(),
            toml::Value::String(persisted_config_layer_value(layer).to_string()),
        );
        table
    };

    table.insert(
        "config-layer".to_string(),
        toml::Value::String(persisted_config_layer_value(layer).to_string()),
    );
    table.insert(
        "clients".to_string(),
        toml::Value::try_from(clients)
            .map_err(|error| ConfigDocError::Invalid(error.to_string()))?,
    );

    std::fs::write(path, table.to_string())?;
    Ok(())
}
