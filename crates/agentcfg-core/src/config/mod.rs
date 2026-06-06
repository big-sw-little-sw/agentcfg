//! Agent Configuration documents and request building.

pub mod skills;

use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::{AgentcfgResult, Client, ClientSelection, ConfigLayerKind, InstallLevel};

pub const SCHEMA_VERSION: u32 = 1;

/// One persisted Agent Configuration document before active-layer request building.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigDoc {
    pub version: u32,
    pub config_layer: ConfigLayerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clients: Option<PersistedClientSelection>,
    #[serde(default)]
    pub skills: skills::SkillConfigDoc,
}

/// Persisted `clients` field value before command-level client selection rules.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// A loaded Config Layer document with its layer identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedConfigDoc {
    pub kind: ConfigLayerKind,
    pub doc: ConfigDoc,
}

/// One active Config Layer after command-level filtering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigLayerRequest {
    pub kind: ConfigLayerKind,
}

/// Active configuration intent after layer, Install Level, and client selection rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigRequest {
    pub install_level: InstallLevel,
    pub clients: ClientSelection,
    pub layers: Vec<ConfigLayerRequest>,
    pub skills: skills::SkillConfigRequest,
}

pub fn build_request(
    _config_docs: &[LoadedConfigDoc],
    _install_level: InstallLevel,
    _clients: ClientSelection,
) -> AgentcfgResult<ConfigRequest> {
    unimplemented!("config request building is not implemented yet")
}

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize,
        de::{
            IntoDeserializer,
            value::{Error as ValueError, SeqDeserializer, StrDeserializer},
        },
    };

    use super::*;

    #[test]
    fn persisted_client_selection_accepts_all_keyword() {
        let deserializer: StrDeserializer<'_, ValueError> = "all".into_deserializer();

        let selection = PersistedClientSelection::deserialize(deserializer).unwrap();

        assert_eq!(selection, PersistedClientSelection::All);
    }

    #[test]
    fn persisted_client_selection_rejects_unknown_keyword() {
        let deserializer: StrDeserializer<'_, ValueError> = "default".into_deserializer();

        let error = PersistedClientSelection::deserialize(deserializer).unwrap_err();

        assert!(error.to_string().contains("\"all\""));
    }

    #[test]
    fn persisted_client_selection_accepts_client_list() {
        let clients: Vec<StrDeserializer<'_, ValueError>> =
            vec!["codex".into_deserializer(), "cursor".into_deserializer()];
        let deserializer: SeqDeserializer<_, ValueError> =
            SeqDeserializer::new(clients.into_iter());

        let selection = PersistedClientSelection::deserialize(deserializer).unwrap();

        assert_eq!(
            selection,
            PersistedClientSelection::Explicit(vec![Client::Codex, Client::Cursor])
        );
    }
}
