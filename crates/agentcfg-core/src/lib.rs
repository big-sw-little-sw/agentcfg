mod discovery_registry;
pub mod config;
pub mod config_paths;
pub mod desired_state;
mod error;
pub mod scope;
pub mod workflow;

pub use desired_state::ConfiguredItemKind;
pub use error::{
    ConfigError, Error, InitError, PathEnvironmentError, Result, SourceError, UnsupportedError,
};
