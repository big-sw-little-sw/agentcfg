//! Skill Configuration parsing and persistence for one Config Layer.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::Path;

use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

use crate::config_doc::{
    read_default_clients, ConfigDocError, PersistedClientSelection, SCHEMA_VERSION,
};
use crate::locations::persisted_config_layer_value;
use crate::ConfigLayerId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillConfiguration {
    pub entries: Vec<SkillConfigurationEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SkillConfigurationEntry {
    #[serde(default)]
    pub id: Option<String>,
    pub source: String,
    #[serde(default, rename = "ref")]
    pub git_ref: Option<String>,
    pub include: SkillSelection,
    #[serde(default)]
    pub clients: Option<PersistedClientSelection>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSelection {
    Explicit(Vec<String>),
    All,
}

impl<'de> Deserialize<'de> for SkillSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SkillSelectionVisitor)
    }
}

struct SkillSelectionVisitor;

impl<'de> Visitor<'de> for SkillSelectionVisitor {
    type Value = SkillSelection;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(r#""all" or a list of Source Skill Names"#)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "all" => Ok(SkillSelection::All),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut skills = Vec::new();
        while let Some(skill) = sequence.next_element()? {
            skills.push(skill);
        }
        Ok(SkillSelection::Explicit(skills))
    }
}

impl SkillConfigurationEntry {
    pub fn matches_source(&self, source: &str, git_ref: Option<&str>) -> bool {
        self.source == source && self.git_ref.as_deref() == git_ref
    }

    pub fn matches_entry_id(&self, entry_id: &str) -> bool {
        self.id.as_deref() == Some(entry_id)
    }

    pub fn locator_agrees_with(&self, source: Option<&str>, git_ref: Option<&str>) -> bool {
        if let Some(source) = source {
            if self.source != source {
                return false;
            }
        }
        if let Some(git_ref) = git_ref {
            if self.git_ref.as_deref() != Some(git_ref) {
                return false;
            }
        }
        true
    }

    pub fn is_compatible_with_explicit_selection(
        &self,
        entry_id: Option<&str>,
        source: &str,
        git_ref: Option<&str>,
    ) -> bool {
        match entry_id {
            Some(id) if self.id.as_deref() != Some(id) => return false,
            Some(_) => {}
            None if self.id.is_some() => {}
            None => {
                if !self.matches_source(source, git_ref) {
                    return false;
                }
            }
        }
        if !matches!(self.include, SkillSelection::Explicit(_)) {
            return false;
        }
        if !self.exclude.is_empty() || !self.aliases.is_empty() {
            return false;
        }
        // New selections inherit Default Client Selection; entry-level clients are incompatible.
        self.clients.is_none()
    }

    pub fn final_client_selection(
        &self,
        default_clients: Option<&PersistedClientSelection>,
    ) -> Option<PersistedClientSelection> {
        self.clients.clone().or_else(|| default_clients.cloned())
    }
}

pub fn read_skill_configuration(path: &Path) -> Result<SkillConfiguration, ConfigDocError> {
    if !path.exists() {
        return Ok(SkillConfiguration::default());
    }

    let content = std::fs::read_to_string(path)?;
    let partial: SkillsConfigSection = toml::from_str(&content)
        .map_err(|error: toml::de::Error| ConfigDocError::Invalid(error.to_string()))?;

    let configuration = SkillConfiguration {
        entries: partial.skills,
    };
    validate_entry_ids(&configuration.entries)?;
    Ok(configuration)
}

pub fn write_skill_configuration(
    path: &Path,
    layer: ConfigLayerId,
    configuration: &SkillConfiguration,
) -> Result<(), ConfigDocError> {
    validate_entry_ids(&configuration.entries)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut doc = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        content
            .parse::<DocumentMut>()
            .map_err(|error| ConfigDocError::Invalid(error.to_string()))?
    } else {
        let mut doc = DocumentMut::new();
        doc.insert("version", i64::from(SCHEMA_VERSION).into());
        doc.insert("config-layer", persisted_config_layer_value(layer).into());
        doc
    };

    doc.insert("config-layer", persisted_config_layer_value(layer).into());
    if configuration.entries.is_empty() {
        doc.remove("skills");
    } else {
        doc.insert("skills", skills_toml_value(configuration));
    }

    std::fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn validate_entry_ids(entries: &[SkillConfigurationEntry]) -> Result<(), ConfigDocError> {
    let mut seen = HashSet::new();
    for entry in entries {
        let Some(id) = entry.id.as_deref() else {
            continue;
        };
        if !seen.insert(id) {
            return Err(ConfigDocError::Invalid(format!(
                "duplicate Skill Configuration Entry Id '{id}'"
            )));
        }
    }
    Ok(())
}

pub fn final_client_selection_for_new_entry(
    path: &Path,
) -> Result<Option<PersistedClientSelection>, ConfigDocError> {
    read_default_clients(path)
}

#[derive(Debug, Deserialize)]
struct SkillsConfigSection {
    #[serde(default)]
    skills: Vec<SkillConfigurationEntry>,
}

fn skills_toml_value(configuration: &SkillConfiguration) -> Item {
    let mut skills = toml_edit::ArrayOfTables::new();
    for entry in &configuration.entries {
        skills.push(entry_table(entry));
    }
    Item::ArrayOfTables(skills)
}

fn entry_table(entry: &SkillConfigurationEntry) -> Table {
    let mut table = Table::new();
    if let Some(id) = &entry.id {
        table.insert("id", Value::from(id.as_str()).into());
    }
    table.insert("source", Value::from(entry.source.as_str()).into());
    if let Some(git_ref) = &entry.git_ref {
        table.insert("ref", Value::from(git_ref.as_str()).into());
    }
    match &entry.include {
        SkillSelection::Explicit(skills) => {
            let mut include = Array::new();
            for skill in skills {
                include.push(skill.as_str());
            }
            table.insert("include", Value::Array(include).into());
        }
        SkillSelection::All => {
            table.insert("include", Value::from("all").into());
        }
    }
    if let Some(clients) = &entry.clients {
        table.insert("clients", clients_toml_value(clients));
    }
    if !entry.exclude.is_empty() {
        let mut exclude = Array::new();
        for skill in &entry.exclude {
            exclude.push(skill.as_str());
        }
        table.insert("exclude", Value::Array(exclude).into());
    }
    if !entry.aliases.is_empty() {
        let mut aliases = InlineTable::new();
        for (source_skill, alias) in &entry.aliases {
            aliases.insert(source_skill, Value::from(alias.as_str()));
        }
        table.insert("aliases", Item::Value(Value::InlineTable(aliases)));
    }
    table
}

fn clients_toml_value(clients: &PersistedClientSelection) -> Item {
    match clients {
        PersistedClientSelection::All => Value::from("all").into(),
        PersistedClientSelection::Explicit(clients) => {
            let mut array = Array::new();
            for client in clients {
                array.push(client.to_string());
            }
            Value::Array(array).into()
        }
    }
}
