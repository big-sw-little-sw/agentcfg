use std::path::PathBuf;

use crate::scope::ConfigLayer;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Source(#[from] SourceError),

    #[error(transparent)]
    PathEnvironment(#[from] PathEnvironmentError),

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
        "config scope mismatch at {path}: expected `{expected_scope}` for {expected_layer:?}, got `{actual_scope}`"
    )]
    ScopeMismatch {
        path: PathBuf,
        expected_layer: ConfigLayer,
        expected_scope: &'static str,
        actual_scope: String,
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

    #[error("duplicate source id `{source_id}` at {path} for {layer:?}")]
    DuplicateSourceId {
        path: PathBuf,
        layer: ConfigLayer,
        source_id: String,
    },

    #[error(
        "invalid skill alias key `{alias_key}` at {path} for {layer:?}; expected `source_id:skill_name`"
    )]
    InvalidAliasKey {
        path: PathBuf,
        layer: ConfigLayer,
        alias_key: String,
    },

    #[error(
        "skill alias `{alias_key}` references unknown source id `{source_id}` at {path} for {layer:?}"
    )]
    UnknownAliasSource {
        path: PathBuf,
        layer: ConfigLayer,
        alias_key: String,
        source_id: String,
    },

    #[error("unsupported config field `{field}` at {path} for {layer:?}")]
    UnsupportedField {
        path: PathBuf,
        layer: ConfigLayer,
        field: &'static str,
    },

    #[error(
        "unsupported source kind `{kind}` at {path} for {layer:?}; supported source kinds in V1: path; git source support is planned for a later phase"
    )]
    UnsupportedSourceKind {
        path: PathBuf,
        layer: ConfigLayer,
        source_id: Option<String>,
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
pub enum SourceError {
    #[error("source `{source_id}` was not found")]
    NotFound { source_id: String },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PathEnvironmentError {
    #[error("HOME is required to resolve the default for {xdg_var}; set HOME or {xdg_var}")]
    MissingHomeForXdgFallback { xdg_var: &'static str },
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
}
