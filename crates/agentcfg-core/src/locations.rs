//! Config Layer and Install Level path resolution.

use std::path::{Path, PathBuf};

use crate::{ConfigLayerId, InstallLevel};

pub fn resolve_project_root(start: &Path) -> PathBuf {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return current;
        }
        if !current.pop() {
            return start.to_path_buf();
        }
    }
}

pub fn config_layer_path(project_root: &Path, layer: ConfigLayerId) -> PathBuf {
    match layer {
        ConfigLayerId::SharedProject => project_root.join("agentcfg.toml"),
        ConfigLayerId::UserProject => project_root.join(".agentcfg").join("agentcfg.toml"),
        ConfigLayerId::User => user_config_path(),
    }
}

pub fn user_config_path() -> PathBuf {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home)
            .join("agentcfg")
            .join("agentcfg.toml");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("agentcfg")
            .join("agentcfg.toml");
    }

    PathBuf::from(".config/agentcfg/agentcfg.toml")
}

pub fn active_config_layers(install_level: InstallLevel) -> Vec<ConfigLayerId> {
    match install_level {
        InstallLevel::Project => vec![ConfigLayerId::SharedProject, ConfigLayerId::UserProject],
        InstallLevel::User => vec![ConfigLayerId::User],
    }
}

pub fn layer_label(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "Shared Project Config",
        ConfigLayerId::UserProject => "User Project Config",
        ConfigLayerId::User => "User Config",
    }
}

pub fn layer_relative_path_label(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "agentcfg.toml",
        ConfigLayerId::UserProject => ".agentcfg/agentcfg.toml",
        ConfigLayerId::User => "agentcfg.toml",
    }
}

pub fn persisted_config_layer_value(layer: ConfigLayerId) -> &'static str {
    match layer {
        ConfigLayerId::SharedProject => "shared-project",
        ConfigLayerId::UserProject => "user-project",
        ConfigLayerId::User => "user",
    }
}
