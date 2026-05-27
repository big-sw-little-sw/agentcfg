//! Path derivation for config files, lockfiles, and **Managed State**.
//!
//! Lockfiles sit beside Config Layer config files. **Managed State** paths include
//! the Manifest and the Managed Skill Content root used when applying Locked Desired State.
//!
//! This module derives paths only. It does not create directories, read config
//! contents, or mutate the filesystem.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::layer_level::{ConfigLayer, InstallLevel};
use crate::{PathEnvironmentError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigFilePaths {
    layer: ConfigLayer,
    config_file: PathBuf,
    lockfile: PathBuf,
}

impl ConfigFilePaths {
    pub fn for_shared_project(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref();

        Self {
            layer: ConfigLayer::SharedProject,
            config_file: project_root.join("agentcfg.toml"),
            lockfile: project_root.join("agentcfg.lock"),
        }
    }

    pub fn for_user_project(project_root: impl AsRef<Path>) -> Self {
        let agentcfg_dir = project_root.as_ref().join(".agentcfg");

        Self {
            layer: ConfigLayer::UserProject,
            config_file: agentcfg_dir.join("config.toml"),
            lockfile: agentcfg_dir.join("lock.toml"),
        }
    }

    pub fn for_user_config_home(config_home: impl AsRef<Path>) -> Self {
        let agentcfg_dir = config_home.as_ref().join("agentcfg");

        Self {
            layer: ConfigLayer::User,
            config_file: agentcfg_dir.join("config.toml"),
            lockfile: agentcfg_dir.join("lock.toml"),
        }
    }

    pub fn layer(&self) -> ConfigLayer {
        self.layer
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn lockfile(&self) -> &Path {
        &self.lockfile
    }
}

/// Paths under **Managed State** for the Manifest and **Managed Skill Content** derived from Skill Sources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedStatePaths {
    install_level: InstallLevel,
    manifest: PathBuf,
    managed_skill_content_root: PathBuf,
}

impl ManagedStatePaths {
    pub fn for_project(project_root: impl AsRef<Path>) -> Self {
        let agentcfg_dir = project_root.as_ref().join(".agentcfg");

        Self {
            install_level: InstallLevel::Project,
            manifest: agentcfg_dir.join("manifest.json"),
            managed_skill_content_root: agentcfg_dir.join("sources"),
        }
    }

    pub fn for_user_state_home(state_home: impl AsRef<Path>) -> Self {
        let agentcfg_dir = state_home.as_ref().join("agentcfg");

        Self {
            install_level: InstallLevel::User,
            manifest: agentcfg_dir.join("manifest.json"),
            managed_skill_content_root: agentcfg_dir.join("sources"),
        }
    }

    pub fn install_level(&self) -> InstallLevel {
        self.install_level
    }

    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    pub fn managed_skill_content_root(&self) -> &Path {
        &self.managed_skill_content_root
    }
}

/// Resolved user environment paths for agentcfg and Client Discovery Locations.
///
/// `config_home` and `state_home` follow XDG conventions (with `HOME` fallbacks).
/// `home_dir` is the user home directory (`HOME`) used for user-level
/// **Client Discovery Locations** (for example `~/.agents/skills`). It is not
/// derived from `config_home` when `XDG_CONFIG_HOME` points elsewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserDirs {
    config_home: PathBuf,
    state_home: PathBuf,
    home_dir: Option<PathBuf>,
}

impl UserDirs {
    pub fn new(
        config_home: impl Into<PathBuf>,
        state_home: impl Into<PathBuf>,
        home_dir: Option<impl Into<PathBuf>>,
    ) -> Self {
        Self {
            config_home: config_home.into(),
            state_home: state_home.into(),
            home_dir: home_dir.map(Into::into),
        }
    }

    pub fn from_env() -> Result<Self> {
        Self::from_env_vars(
            env::var_os("XDG_CONFIG_HOME"),
            env::var_os("XDG_STATE_HOME"),
            env::var_os("HOME"),
        )
    }

    pub fn from_env_vars(
        xdg_config_home: Option<impl Into<PathBuf>>,
        xdg_state_home: Option<impl Into<PathBuf>>,
        home: Option<impl Into<PathBuf>>,
    ) -> Result<Self> {
        let xdg_config_home = absolute_xdg_path(xdg_config_home);
        let xdg_state_home = absolute_xdg_path(xdg_state_home);
        let home = non_empty_path(home);

        Ok(Self::new(
            xdg_dir_or_home_fallback(
                xdg_config_home,
                home.as_deref(),
                "XDG_CONFIG_HOME",
                ".config",
            )?,
            xdg_dir_or_home_fallback(
                xdg_state_home,
                home.as_deref(),
                "XDG_STATE_HOME",
                ".local/state",
            )?,
            home,
        ))
    }

    pub fn config_home(&self) -> &Path {
        &self.config_home
    }

    pub fn state_home(&self) -> &Path {
        &self.state_home
    }

    /// User home for user-level **Client Discovery Locations**, when `HOME` was set or injected.
    pub fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }
}

fn xdg_dir_or_home_fallback(
    xdg_dir: Option<PathBuf>,
    home: Option<&Path>,
    xdg_var: &'static str,
    fallback_suffix: &str,
) -> Result<PathBuf> {
    if let Some(xdg_dir) = xdg_dir {
        return Ok(xdg_dir);
    }

    home.map(|home| xdg_default_dir(home, fallback_suffix))
        .ok_or_else(|| missing_home_for_xdg_default(xdg_var))
}

fn non_empty_path(path: Option<impl Into<PathBuf>>) -> Option<PathBuf> {
    path.map(Into::into)
        .filter(|path| !path.as_os_str().is_empty())
}

fn absolute_xdg_path(path: Option<impl Into<PathBuf>>) -> Option<PathBuf> {
    non_empty_path(path).filter(|path| path.is_absolute())
}

fn xdg_default_dir(home: &Path, fallback_suffix: &str) -> PathBuf {
    home.join(fallback_suffix)
}

fn missing_home_for_xdg_default(xdg_var: &'static str) -> crate::Error {
    PathEnvironmentError::MissingHomeForXdgFallback { xdg_var }.into()
}

pub fn discover_project_root(start_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let start_dir = start_dir.as_ref();

    let root = ancestors(start_dir)
        .find(|ancestor| has_git_marker(ancestor))
        .or_else(|| ancestors(start_dir).find(|ancestor| has_agentcfg_marker(ancestor)))
        .unwrap_or(start_dir);

    Ok(root.to_path_buf())
}

fn ancestors(path: &Path) -> impl Iterator<Item = &Path> {
    std::iter::successors(Some(path), |path| path.parent())
}

fn has_git_marker(path: &Path) -> bool {
    fs::metadata(path.join(".git")).is_ok()
}

fn has_agentcfg_marker(path: &Path) -> bool {
    fs::metadata(path.join("agentcfg.toml")).is_ok()
        || fs::metadata(path.join(".agentcfg").join("config.toml")).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn shared_project_config_uses_root_file_and_adjacent_lockfile() {
        let paths = ConfigFilePaths::for_shared_project("/repo");

        assert_eq!(paths.layer(), ConfigLayer::SharedProject);
        assert_eq!(paths.config_file(), Path::new("/repo/agentcfg.toml"));
        assert_eq!(paths.lockfile(), Path::new("/repo/agentcfg.lock"));
    }

    #[test]
    fn user_project_config_uses_agentcfg_config_and_lockfile() {
        let paths = ConfigFilePaths::for_user_project("/repo");

        assert_eq!(paths.layer(), ConfigLayer::UserProject);
        assert_eq!(
            paths.config_file(),
            Path::new("/repo/.agentcfg/config.toml")
        );
        assert_eq!(paths.lockfile(), Path::new("/repo/.agentcfg/lock.toml"));
    }

    #[test]
    fn user_config_uses_config_home_without_requiring_state_home() {
        let paths = ConfigFilePaths::for_user_config_home("/home/me/.config");

        assert_eq!(paths.layer(), ConfigLayer::User);
        assert_eq!(
            paths.config_file(),
            Path::new("/home/me/.config/agentcfg/config.toml")
        );
        assert_eq!(
            paths.lockfile(),
            Path::new("/home/me/.config/agentcfg/lock.toml")
        );
    }

    #[test]
    fn managed_state_paths_resolve_manifest_and_managed_skill_content() {
        let project_paths = ManagedStatePaths::for_project("/repo");

        assert_eq!(project_paths.install_level(), InstallLevel::Project);
        assert_eq!(
            project_paths.manifest(),
            Path::new("/repo/.agentcfg/manifest.json")
        );
        assert_eq!(
            project_paths.managed_skill_content_root(),
            Path::new("/repo/.agentcfg/sources")
        );

        let user_paths = ManagedStatePaths::for_user_state_home("/home/me/.local/state");

        assert_eq!(user_paths.install_level(), InstallLevel::User);
        assert_eq!(
            user_paths.manifest(),
            Path::new("/home/me/.local/state/agentcfg/manifest.json")
        );
        assert_eq!(
            user_paths.managed_skill_content_root(),
            Path::new("/home/me/.local/state/agentcfg/sources")
        );
    }

    #[test]
    fn user_dirs_use_xdg_overrides() {
        let user_dirs =
            UserDirs::from_env_vars(Some("/xdg/config"), Some("/xdg/state"), Some("/home/me"))
                .expect("user dirs");

        assert_eq!(user_dirs.config_home(), Path::new("/xdg/config"));
        assert_eq!(user_dirs.state_home(), Path::new("/xdg/state"));
        assert_eq!(user_dirs.home_dir(), Some(Path::new("/home/me")));
    }

    #[test]
    fn user_dirs_fall_back_to_home_for_empty_or_missing_xdg_vars() {
        let user_dirs =
            UserDirs::from_env_vars(Some(""), None::<&str>, Some("/home/me")).expect("user dirs");

        assert_eq!(user_dirs.config_home(), Path::new("/home/me/.config"));
        assert_eq!(user_dirs.state_home(), Path::new("/home/me/.local/state"));
        assert_eq!(user_dirs.home_dir(), Some(Path::new("/home/me")));
    }

    #[test]
    fn user_dirs_fall_back_to_home_for_relative_xdg_config_home() {
        let user_dirs = UserDirs::from_env_vars(
            Some("relative/config"),
            Some("/xdg/state"),
            Some("/home/me"),
        )
        .expect("user dirs");

        assert_eq!(user_dirs.config_home(), Path::new("/home/me/.config"));
        assert_eq!(user_dirs.state_home(), Path::new("/xdg/state"));
    }

    #[test]
    fn user_dirs_fall_back_to_home_for_relative_xdg_state_home() {
        let user_dirs = UserDirs::from_env_vars(
            Some("/xdg/config"),
            Some("relative/state"),
            Some("/home/me"),
        )
        .expect("user dirs");

        assert_eq!(user_dirs.config_home(), Path::new("/xdg/config"));
        assert_eq!(user_dirs.state_home(), Path::new("/home/me/.local/state"));
    }

    #[test]
    fn user_dirs_allow_missing_home_when_xdg_overrides_are_complete() {
        let user_dirs =
            UserDirs::from_env_vars(Some("/xdg/config"), Some("/xdg/state"), None::<&str>)
                .expect("user dirs");

        assert_eq!(user_dirs.config_home(), Path::new("/xdg/config"));
        assert_eq!(user_dirs.state_home(), Path::new("/xdg/state"));
        assert_eq!(user_dirs.home_dir(), None);
    }

    #[test]
    fn user_dirs_report_missing_home_for_config_home_fallback() {
        let error = UserDirs::from_env_vars(None::<&str>, Some("/xdg/state"), None::<&str>)
            .expect_err("missing home should fail");

        assert!(matches!(
            error,
            Error::PathEnvironment(PathEnvironmentError::MissingHomeForXdgFallback {
                xdg_var: "XDG_CONFIG_HOME"
            })
        ));
    }

    #[test]
    fn user_dirs_report_missing_home_for_state_home_fallback() {
        let error = UserDirs::from_env_vars(Some("/xdg/config"), None::<&str>, None::<&str>)
            .expect_err("missing home should fail");

        assert!(matches!(
            error,
            Error::PathEnvironment(PathEnvironmentError::MissingHomeForXdgFallback {
                xdg_var: "XDG_STATE_HOME"
            })
        ));
    }

    #[test]
    fn project_root_discovery_returns_directory_with_git_dir_marker() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir(temp.path().join(".git")).expect("git marker");

        assert_eq!(
            discover_project_root(temp.path()).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_walks_up_to_nearest_git_marker() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("a").join("b");
        fs::create_dir(temp.path().join(".git")).expect("git marker");
        fs::create_dir_all(&nested).expect("nested");

        assert_eq!(
            discover_project_root(&nested).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_accepts_git_file_marker() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join(".git"), "gitdir: ../actual.git").expect("git marker");

        assert_eq!(
            discover_project_root(temp.path()).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_uses_agentcfg_toml_marker_without_git() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(temp.path().join("agentcfg.toml"), "").expect("agentcfg marker");

        assert_eq!(
            discover_project_root(temp.path()).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_uses_user_project_config_marker_without_git() {
        let temp = tempfile::tempdir().expect("tempdir");
        let agentcfg_dir = temp.path().join(".agentcfg");
        fs::create_dir(&agentcfg_dir).expect("agentcfg dir");
        fs::write(agentcfg_dir.join("config.toml"), "").expect("agentcfg marker");

        assert_eq!(
            discover_project_root(temp.path()).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_prefers_git_marker_over_nested_agentcfg_marker() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("a").join("b");
        let nested_agentcfg = nested.join(".agentcfg");

        fs::create_dir(temp.path().join(".git")).expect("git marker");
        fs::create_dir_all(&nested_agentcfg).expect("nested agentcfg");
        fs::write(nested_agentcfg.join("config.toml"), "").expect("agentcfg marker");

        assert_eq!(
            discover_project_root(&nested).expect("project root"),
            temp.path()
        );
    }

    #[test]
    fn project_root_discovery_uses_nearest_agentcfg_marker_when_no_git_marker_exists() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("a").join("b");
        let parent_agentcfg = temp.path().join("a").join(".agentcfg");
        let nested_agentcfg = nested.join(".agentcfg");

        fs::create_dir_all(&parent_agentcfg).expect("parent agentcfg");
        fs::write(parent_agentcfg.join("config.toml"), "").expect("parent marker");
        fs::create_dir_all(&nested_agentcfg).expect("nested agentcfg");
        fs::write(nested_agentcfg.join("config.toml"), "").expect("nested marker");

        assert_eq!(
            discover_project_root(&nested).expect("project root"),
            nested
        );
    }

    #[test]
    fn project_root_discovery_returns_start_directory_when_no_marker_exists() {
        let temp = tempfile::tempdir().expect("tempdir");
        let nested = temp.path().join("a").join("b");
        fs::create_dir_all(&nested).expect("nested");

        assert_eq!(
            discover_project_root(&nested).expect("project root"),
            nested
        );
    }
}
