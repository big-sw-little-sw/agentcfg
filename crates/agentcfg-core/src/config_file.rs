//! Agent Configuration File parsing and editable document updates.

use std::io;
use std::path::Path;

use toml_edit::{Array, DocumentMut, Item, Value};

use crate::clients::ClientId;
use crate::config_layers::ConfigLayerState;
use crate::workflow::Diagnostic;

const DEFAULT_CLIENTS_KEY: &str = "default_clients";
const INVALID_CONFIG_CODE: &str = "invalid-config";
const INVALID_DEFAULT_CLIENTS_CODE: &str = "invalid-default-client-selection";
const UNKNOWN_CLIENT_CODE: &str = "unknown-client";
const UNREADABLE_CONFIG_CODE: &str = "unreadable-config";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ConfigDoc {
    pub(crate) default_clients: Vec<ClientId>,
}

pub(crate) struct ConfigFileRead {
    pub(crate) state: ConfigLayerState,
    pub(crate) doc: DocumentMut,
    pub(crate) config: ConfigDoc,
    pub(crate) blockers: Vec<Diagnostic>,
}

pub(crate) fn read_agent_config(path: &Path) -> ConfigFileRead {
    if !path.exists() {
        return ConfigFileRead {
            state: ConfigLayerState::Missing,
            doc: DocumentMut::new(),
            config: ConfigDoc::default(),
            blockers: Vec::new(),
        };
    }

    let Ok(contents) = std::fs::read_to_string(path) else {
        return ConfigFileRead {
            state: ConfigLayerState::Authored,
            doc: DocumentMut::new(),
            config: ConfigDoc::default(),
            blockers: vec![unreadable_config_diagnostic(path)],
        };
    };

    if contents.trim().is_empty() {
        return ConfigFileRead {
            state: ConfigLayerState::Empty,
            doc: DocumentMut::new(),
            config: ConfigDoc::default(),
            blockers: Vec::new(),
        };
    }

    let Ok(doc) = contents.parse::<DocumentMut>() else {
        return ConfigFileRead {
            state: ConfigLayerState::Authored,
            doc: DocumentMut::new(),
            config: ConfigDoc::default(),
            blockers: vec![invalid_config_diagnostic(path)],
        };
    };

    match parse_config_doc(&doc, path) {
        Ok(config) => ConfigFileRead {
            state: ConfigLayerState::Authored,
            doc,
            config,
            blockers: Vec::new(),
        },
        Err(blocker) => ConfigFileRead {
            state: ConfigLayerState::Authored,
            doc,
            config: ConfigDoc::default(),
            blockers: vec![blocker],
        },
    }
}

pub(crate) fn set_default_clients(doc: &mut DocumentMut, clients: &[ClientId]) {
    doc[DEFAULT_CLIENTS_KEY] = Item::Value(Value::Array(client_array(clients)));
}

pub(crate) fn write_agent_config(path: &Path, doc: DocumentMut) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, doc.to_string())
}

fn parse_config_doc(doc: &DocumentMut, path: &Path) -> Result<ConfigDoc, Diagnostic> {
    Ok(ConfigDoc {
        default_clients: parse_default_clients(doc, path)?,
    })
}

fn parse_default_clients(doc: &DocumentMut, path: &Path) -> Result<Vec<ClientId>, Diagnostic> {
    let Some(item) = doc.get(DEFAULT_CLIENTS_KEY) else {
        return Ok(Vec::new());
    };
    let Some(array) = item.as_array() else {
        return Err(invalid_default_clients_diagnostic(path));
    };

    let mut clients = Vec::new();
    for value in array {
        let Some(client) = value.as_str() else {
            return Err(invalid_default_clients_diagnostic(path));
        };
        let client = client
            .parse::<ClientId>()
            .map_err(|()| unknown_client_diagnostic(client, path))?;
        clients.push(client);
    }

    Ok(clients)
}

fn client_array(clients: &[ClientId]) -> Array {
    let mut array = Array::new();
    for client in clients {
        array.push(client.to_string());
    }
    array
}

fn unreadable_config_diagnostic(path: &Path) -> Diagnostic {
    Diagnostic {
        code: UNREADABLE_CONFIG_CODE.to_string(),
        message: "Cannot read Agent Configuration File.".to_string(),
        context: vec![("path".to_string(), path.to_string_lossy().into_owned())],
        suggested_actions: Vec::new(),
    }
}

fn invalid_config_diagnostic(path: &Path) -> Diagnostic {
    Diagnostic {
        code: INVALID_CONFIG_CODE.to_string(),
        message: "Invalid Agent Configuration File.".to_string(),
        context: vec![("path".to_string(), path.to_string_lossy().into_owned())],
        suggested_actions: Vec::new(),
    }
}

fn invalid_default_clients_diagnostic(path: &Path) -> Diagnostic {
    Diagnostic {
        code: INVALID_DEFAULT_CLIENTS_CODE.to_string(),
        message: "Invalid Default Client Selection.".to_string(),
        context: vec![("path".to_string(), path.to_string_lossy().into_owned())],
        suggested_actions: Vec::new(),
    }
}

fn unknown_client_diagnostic(client: &str, path: &Path) -> Diagnostic {
    Diagnostic {
        code: UNKNOWN_CLIENT_CODE.to_string(),
        message: "Unknown Client in Default Client Selection.".to_string(),
        context: vec![
            ("client".to_string(), client.to_string()),
            ("path".to_string(), path.to_string_lossy().into_owned()),
        ],
        suggested_actions: Vec::new(),
    }
}
