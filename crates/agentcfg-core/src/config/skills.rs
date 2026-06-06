//! Skill Configuration contracts for Config Documents and Config Requests.

use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{ConfigSourceId, DiscoveryName, SourceSkillName};

/// Persisted Skill Configuration from one Config Document.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SkillConfigDoc {
    #[serde(default)]
    pub sources: Vec<SkillSourceConfigDoc>,
}

/// Persisted Skill Source declaration before source contents are inspected.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SkillSourceConfigDoc {
    pub id: ConfigSourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default)]
    pub include: Vec<SourceSkillName>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub aliases: BTreeMap<SourceSkillName, DiscoveryName>,
}

/// Active Skill Configuration request after Config Layer composition.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillConfigRequest {}
