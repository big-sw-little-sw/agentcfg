use std::path::PathBuf;

use crate::layer_level::ConfigLayer;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    SkillSource(#[from] SkillSourceError),

    #[error(transparent)]
    PathEnvironment(#[from] PathEnvironmentError),

    #[error(transparent)]
    Init(#[from] InitError),

    #[error("filesystem error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Unsupported(#[from] UnsupportedError),

    // This is for agentcfg bugs, not user-input validation or recoverable
    // operational failures.
    #[error("internal invariant violated: {message}")]
    Internal { message: String },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("failed to parse config at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error(
        "config Persisted Scope Value mismatch at {path}: expected `{expected_persisted_scope_value}` for {expected_layer:?}, got `{actual_persisted_scope_value}`"
    )]
    PersistedScopeValueMismatch {
        path: PathBuf,
        expected_layer: ConfigLayer,
        expected_persisted_scope_value: &'static str,
        actual_persisted_scope_value: String,
    },

    #[error("missing required config field `{field}` at {path} for {layer:?}")]
    MissingRequiredField {
        path: PathBuf,
        layer: ConfigLayer,
        field: &'static str,
    },

    #[error("empty required config field `{field}` at {path} for {layer:?}")]
    EmptyRequiredField {
        path: PathBuf,
        layer: ConfigLayer,
        field: &'static str,
    },

    #[error("duplicate Skill Source id `{skill_source_id}` at {path} for {layer:?}")]
    DuplicateSkillSourceId {
        path: PathBuf,
        layer: ConfigLayer,
        skill_source_id: String,
    },

    #[error(
        "invalid Skill Alias key `{skill_alias_key}` at {path} for {layer:?}; expected `skill_source_id:source_skill_name` (Source Skill Name)"
    )]
    InvalidSkillAliasKey {
        path: PathBuf,
        layer: ConfigLayer,
        skill_alias_key: String,
    },

    #[error(
        "Skill Alias `{skill_alias_key}` references unknown Skill Source id `{skill_source_id}` at {path} for {layer:?}"
    )]
    UnknownSkillAliasSkillSource {
        path: PathBuf,
        layer: ConfigLayer,
        skill_alias_key: String,
        skill_source_id: String,
    },

    #[error("unsupported config field `{field}` at {path} for {layer:?}")]
    UnsupportedField {
        path: PathBuf,
        layer: ConfigLayer,
        field: &'static str,
    },

    #[error(
        "unsupported Skill Source kind `{kind}` at {path} for {layer:?}; supported Skill Source kinds in V1: path; git Skill Source support is planned for a later phase"
    )]
    UnsupportedSkillSourceKind {
        path: PathBuf,
        layer: ConfigLayer,
        skill_source_id: Option<String>,
        kind: String,
    },

    #[error("invalid config value `{value}` for `{field}` at {path} for {layer:?}")]
    InvalidFieldValue {
        path: PathBuf,
        layer: ConfigLayer,
        field: &'static str,
        value: String,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SkillSourceError {
    #[error(
        "Skill Source `{skill_source_id}` was not found at `{configured_path}` (resolved to `{resolved_path}`)"
    )]
    NotFound {
        skill_source_id: String,
        configured_path: PathBuf,
        resolved_path: PathBuf,
    },

    #[error(
        "Skill Source `{skill_source_id}` is not a directory at `{configured_path}` (resolved to `{resolved_path}`)"
    )]
    NotDirectory {
        skill_source_id: String,
        configured_path: PathBuf,
        resolved_path: PathBuf,
    },

    #[error(
        "duplicate Source Skill Name `{source_skill_name}` in Skill Source `{skill_source_id}` at: {skill_dirs:?}"
    )]
    DuplicateSourceSkillName {
        skill_source_id: String,
        source_skill_name: String,
        skill_dirs: Vec<PathBuf>,
    },

    #[error(
        "non-UTF-8 Source Skill Name in directory `{skill_dir}` for Skill Source `{skill_source_id}`"
    )]
    NonUtf8SourceSkillName {
        skill_source_id: String,
        skill_dir: PathBuf,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PathEnvironmentError {
    #[error("HOME is required to resolve the default for {xdg_var}; set HOME or {xdg_var}")]
    MissingHomeForXdgFallback { xdg_var: &'static str },

    #[error(
        "HOME must be an absolute path for Client Discovery Location resolution; set HOME to an absolute path"
    )]
    HomeNotAbsolute,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InitError {
    #[error("config already exists at {path} for {layer:?}")]
    ConfigAlreadyExists { path: PathBuf, layer: ConfigLayer },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UnsupportedError {
    #[error("unsupported feature `{feature}`")]
    Feature { feature: &'static str },
}

// Keep variants broad only while callers can handle the failures the same way.
// Add a new variant when a failure needs distinct remediation, tests, or
// structured fields.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_structured_io_error() {
        let error = Error::Io {
            path: PathBuf::from("agentcfg.toml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing file"),
        };

        assert_eq!(
            error.to_string(),
            "filesystem error at agentcfg.toml: missing file"
        );
    }

    #[test]
    fn converts_and_displays_wrapped_config_error() {
        let error: Error = ConfigError::MissingRequiredField {
            path: PathBuf::from("agentcfg.toml"),
            layer: ConfigLayer::SharedProject,
            field: "scope",
        }
        .into();

        assert!(matches!(error, Error::Config(_)));
        assert_eq!(
            error.to_string(),
            "missing required config field `scope` at agentcfg.toml for SharedProject"
        );
    }

    #[test]
    fn converts_and_displays_wrapped_skill_source_not_directory_error() {
        let error: Error = SkillSourceError::NotDirectory {
            skill_source_id: "personal".to_string(),
            configured_path: PathBuf::from("../skills"),
            resolved_path: PathBuf::from("/workspace/skills"),
        }
        .into();

        assert!(matches!(error, Error::SkillSource(_)));
        assert_eq!(
            error.to_string(),
            "Skill Source `personal` is not a directory at `../skills` (resolved to `/workspace/skills`)"
        );
    }

    #[test]
    fn converts_and_displays_wrapped_path_environment_error() {
        let error: Error = PathEnvironmentError::MissingHomeForXdgFallback {
            xdg_var: "XDG_CONFIG_HOME",
        }
        .into();

        assert!(matches!(error, Error::PathEnvironment(_)));
        assert_eq!(
            error.to_string(),
            "HOME is required to resolve the default for XDG_CONFIG_HOME; set HOME or XDG_CONFIG_HOME"
        );
    }

    #[test]
    fn converts_and_displays_wrapped_init_error() {
        let error: Error = InitError::ConfigAlreadyExists {
            path: PathBuf::from(".agentcfg/config.toml"),
            layer: ConfigLayer::UserProject,
        }
        .into();

        assert!(matches!(error, Error::Init(_)));
        assert_eq!(
            error.to_string(),
            "config already exists at .agentcfg/config.toml for UserProject"
        );
    }
}
