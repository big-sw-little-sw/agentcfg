//! User workflow entrypoints for the CLI and future frontends.
//!
//! Preview and apply orchestrate resolution of **Locked Desired State** into
//! **Managed State** and **Client Discovery Locations**. V1 stubs return
//! placeholder results until preview operation generation and apply are implemented.
//!
//! **Status** reports managed install-state consistency for an Install Level.
//! **Doctor** reports environment and configuration readiness; it does not
//! replace **Status** for install-state reporting.
//!
//! **Prune** removes **Stale Discovery Requirements** and **Stale Installed
//! Artifacts** from Managed State when removal is safe.
//!
//! These functions are orchestration boundaries, not the lower-level
//! config, preview operation, apply, status, or diagnostic APIs. Those focused APIs
//! should be added when they are needed by implemented behavior.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::parse_config_str;
use crate::config_paths::{ConfigFilePaths, UserDirs, discover_project_root};
use crate::discovery_registry::{
    ClientDiscoveryLocation, project_client_discovery_locations, user_client_discovery_locations,
};
pub use crate::layer_level::{ConfigLayer, InstallLevel};
use crate::{Error, InitError, Result};

/// How preview/apply move from **Desired State** to **Locked Desired State** via lockfiles.
///
/// Active Config Layers express **Desired State**; lockfiles record **Locked Desired State**
/// for Configured Items that need repeatable Skill Source resolution.
///
/// - [`UseLocked`]: use **Locked Desired State** from the active lockfile without Source Refresh.
/// - [`RefreshSources`]: perform **Source Refresh** to refresh Skill Source resolutions before
///   producing updated **Locked Desired State** and materializing **Managed Skill Content**.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillSourceResolutionPolicy {
    UseLocked,
    RefreshSources,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitRequest {
    pub config_layer: ConfigLayer,
}

impl InitRequest {
    pub fn new(config_layer: ConfigLayer) -> Self {
        Self { config_layer }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InitResult {
    pub config_file: PathBuf,
    pub warnings: Vec<InitWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InitWarning {
    UnmanagedArtifact(UnmanagedArtifact),
    ClientDiscoveryLocationReadFailure(ClientDiscoveryLocationReadFailure),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UnmanagedArtifact {
    pub clients: Vec<&'static str>,
    pub install_level: InstallLevel,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ClientDiscoveryLocationReadFailure {
    pub clients: Vec<&'static str>,
    pub install_level: InstallLevel,
    pub path: PathBuf,
    pub error: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DiscoveryLocationInspection {
    unmanaged_artifacts: Vec<UnmanagedArtifact>,
    read_failures: Vec<ClientDiscoveryLocationReadFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PreviewRequest {
    pub install_level: InstallLevel,
    pub skill_source_resolution: SkillSourceResolutionPolicy,
}

impl PreviewRequest {
    pub fn new(
        install_level: InstallLevel,
        skill_source_resolution: SkillSourceResolutionPolicy,
    ) -> Self {
        Self {
            install_level,
            skill_source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PreviewResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApplyRequest {
    pub install_level: InstallLevel,
    pub skill_source_resolution: SkillSourceResolutionPolicy,
}

impl ApplyRequest {
    pub fn new(
        install_level: InstallLevel,
        skill_source_resolution: SkillSourceResolutionPolicy,
    ) -> Self {
        Self {
            install_level,
            skill_source_resolution,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApplyResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneRequest {
    pub install_level: InstallLevel,
}

impl PruneRequest {
    pub fn new(install_level: InstallLevel) -> Self {
        Self { install_level }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PruneResult {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusRequest {
    pub install_level: InstallLevel,
}

impl StatusRequest {
    pub fn new(install_level: InstallLevel) -> Self {
        Self { install_level }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusResult {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct DoctorRequest {}

impl DoctorRequest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct DoctorResult {}

pub fn init(request: InitRequest) -> Result<InitResult> {
    let context = WorkflowContext::from_process()?;
    init_with_context(request, &context)
}

pub fn preview(_request: PreviewRequest) -> Result<PreviewResult> {
    Ok(PreviewResult {})
}

pub fn apply(_request: ApplyRequest) -> Result<ApplyResult> {
    Ok(ApplyResult {})
}

pub fn prune(_request: PruneRequest) -> Result<PruneResult> {
    Ok(PruneResult {})
}

pub fn status(_request: StatusRequest) -> Result<StatusResult> {
    Ok(StatusResult {})
}

pub fn doctor(_request: DoctorRequest) -> Result<DoctorResult> {
    Ok(DoctorResult {})
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkflowContext {
    cwd: PathBuf,
    xdg_config_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
}

impl WorkflowContext {
    fn from_process() -> Result<Self> {
        let cwd = env::current_dir().map_err(|source| Error::Io {
            path: PathBuf::from("."),
            source,
        })?;

        Ok(Self::new(
            cwd,
            env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            env::var_os("HOME").map(PathBuf::from),
        ))
    }

    fn new(
        cwd: impl Into<PathBuf>,
        xdg_config_home: Option<impl Into<PathBuf>>,
        home_dir: Option<impl Into<PathBuf>>,
    ) -> Self {
        Self {
            cwd: cwd.into(),
            xdg_config_home: xdg_config_home.map(Into::into),
            home_dir: home_dir.map(Into::into),
        }
    }

    fn user_config_home(&self) -> Result<PathBuf> {
        UserDirs::config_home_from_env_vars(self.xdg_config_home.clone(), self.home_dir.clone())
    }

    fn home_dir(&self) -> Option<&Path> {
        self.home_dir.as_deref()
    }
}

fn init_with_context(request: InitRequest, context: &WorkflowContext) -> Result<InitResult> {
    let config_paths = init_config_paths(request.config_layer, context)?;
    let contents = starter_config_contents(request.config_layer);

    parse_config_str(
        request.config_layer,
        config_paths.config_file().to_path_buf(),
        &contents,
    )?;
    create_config_file(config_paths.config_file(), request.config_layer, &contents)?;

    Ok(InitResult {
        config_file: config_paths.config_file().to_path_buf(),
        warnings: init_warnings_from(inspect_existing_discovery_locations(
            request.config_layer,
            context,
        )),
    })
}

fn init_config_paths(layer: ConfigLayer, context: &WorkflowContext) -> Result<ConfigFilePaths> {
    match layer {
        ConfigLayer::SharedProject => {
            let project_root = discover_project_root(&context.cwd)?;
            Ok(ConfigFilePaths::for_shared_project(project_root))
        }
        ConfigLayer::UserProject => {
            let project_root = discover_project_root(&context.cwd)?;
            Ok(ConfigFilePaths::for_user_project(project_root))
        }
        ConfigLayer::User => Ok(ConfigFilePaths::for_user_config_home(
            context.user_config_home()?,
        )),
    }
}

fn starter_config_contents(layer: ConfigLayer) -> String {
    format!(
        "scope = \"{}\"\n\n[skills]\nclients = \"all\"\n",
        layer.persisted_scope_value()
    )
}

fn create_config_file(path: &Path, layer: ConfigLayer, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::AlreadyExists {
                InitError::ConfigAlreadyExists {
                    path: path.to_path_buf(),
                    layer,
                }
                .into()
            } else {
                Error::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;

    file.write_all(contents.as_bytes())
        .and_then(|_| file.flush())
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn inspect_existing_discovery_locations(
    layer: ConfigLayer,
    context: &WorkflowContext,
) -> DiscoveryLocationInspection {
    let locations = match layer {
        ConfigLayer::SharedProject | ConfigLayer::UserProject => {
            let Ok(project_root) = discover_project_root(&context.cwd) else {
                return DiscoveryLocationInspection::default();
            };
            project_client_discovery_locations(&project_root)
        }
        ConfigLayer::User => {
            let Some(home_dir) = context.home_dir() else {
                return DiscoveryLocationInspection::default();
            };
            user_client_discovery_locations(home_dir)
        }
    };

    let mut inspection = DiscoveryLocationInspection::default();
    for location in locations {
        match scan_discovery_location(&location) {
            Ok(artifacts) => inspection.unmanaged_artifacts.extend(artifacts),
            Err(read_failure) => inspection.read_failures.push(read_failure),
        }
    }

    inspection
}

fn init_warnings_from(inspection: DiscoveryLocationInspection) -> Vec<InitWarning> {
    inspection
        .read_failures
        .into_iter()
        .map(InitWarning::ClientDiscoveryLocationReadFailure)
        .chain(
            inspection
                .unmanaged_artifacts
                .into_iter()
                .map(InitWarning::UnmanagedArtifact),
        )
        .collect()
}

fn scan_discovery_location(
    location: &ClientDiscoveryLocation,
) -> std::result::Result<Vec<UnmanagedArtifact>, ClientDiscoveryLocationReadFailure> {
    match location.path.try_exists() {
        Ok(false) => return Ok(Vec::new()),
        Ok(true) => {}
        Err(source) => return Err(scan_failure(location, source)),
    }

    let entries = fs::read_dir(&location.path).map_err(|source| scan_failure(location, source))?;

    let mut artifacts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| scan_failure(location, source))?;

        artifacts.push(UnmanagedArtifact {
            clients: location.clients.clone(),
            install_level: location.install_level,
            path: entry.path(),
        });
    }

    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn scan_failure(
    location: &ClientDiscoveryLocation,
    source: std::io::Error,
) -> ClientDiscoveryLocationReadFailure {
    ClientDiscoveryLocationReadFailure {
        clients: location.clients.clone(),
        install_level: location.install_level,
        path: location.path.clone(),
        error: source.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config_str;

    #[test]
    fn starter_configs_parse_for_all_layers() {
        for layer in [
            ConfigLayer::SharedProject,
            ConfigLayer::UserProject,
            ConfigLayer::User,
        ] {
            let config =
                parse_config_str(layer, "config.toml", &starter_config_contents(layer)).unwrap();

            assert_eq!(config.layer(), layer);
        }
    }

    #[test]
    fn default_init_creates_user_project_config() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::UserProject), &context).unwrap();

        let config_file = temp.path().join(".agentcfg").join("config.toml");
        assert_eq!(result.config_file, config_file);
        assert!(temp.path().join(".agentcfg").is_dir());
        assert_config_layer(&config_file, ConfigLayer::UserProject);
    }

    #[test]
    fn project_init_creates_shared_project_config_without_agentcfg_dir() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        let config_file = temp.path().join("agentcfg.toml");
        assert_eq!(result.config_file, config_file);
        assert!(!temp.path().join(".agentcfg").exists());
        assert_config_layer(&config_file, ConfigLayer::SharedProject);
    }

    #[test]
    fn user_init_creates_user_config_without_project_agentcfg_dir() {
        let temp = tempfile::tempdir().unwrap();
        let config_home = temp.path().join("xdg-config");
        let context = WorkflowContext::new(
            temp.path(),
            Some(config_home.clone()),
            Some(temp.path().join("home")),
        );

        let result = init_with_context(InitRequest::new(ConfigLayer::User), &context).unwrap();

        let config_file = config_home.join("agentcfg").join("config.toml");
        assert_eq!(result.config_file, config_file);
        assert!(!temp.path().join(".agentcfg").exists());
        assert_config_layer(&config_file, ConfigLayer::User);
    }

    #[test]
    fn existing_config_is_not_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let agentcfg_dir = temp.path().join(".agentcfg");
        let config_file = agentcfg_dir.join("config.toml");
        fs::create_dir(&agentcfg_dir).unwrap();
        fs::write(&config_file, "existing").unwrap();
        let context = context_for_project(temp.path());

        let error =
            init_with_context(InitRequest::new(ConfigLayer::UserProject), &context).unwrap_err();

        assert!(matches!(
            error,
            Error::Init(InitError::ConfigAlreadyExists {
                ref path,
                layer: ConfigLayer::UserProject,
            }) if path == &config_file
        ));
        assert_eq!(fs::read_to_string(config_file).unwrap(), "existing");
    }

    #[test]
    fn unmanaged_project_artifacts_are_reported_and_not_modified() {
        let temp = tempfile::tempdir().unwrap();
        let agents_skill = temp.path().join(".agents").join("skills").join("review");
        let claude_skill = temp.path().join(".claude").join("skills").join("docs");
        fs::create_dir_all(&agents_skill).unwrap();
        fs::create_dir_all(&claude_skill).unwrap();
        fs::write(agents_skill.join("SKILL.md"), "review").unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        let artifacts = unmanaged_artifacts(&result);
        assert_eq!(
            artifacts.len(),
            2,
            "artifacts were not grouped: {artifacts:?}"
        );
        assert_artifact(
            &artifacts,
            &["codex", "cursor", "opencode", "pi"],
            &agents_skill,
        );
        assert_artifact(&artifacts, &["claude"], &claude_skill);
        assert_eq!(
            fs::read_to_string(agents_skill.join("SKILL.md")).unwrap(),
            "review"
        );
        assert!(!temp.path().join(".cline").exists());
    }

    #[test]
    fn missing_discovery_location_roots_are_not_created() {
        let temp = tempfile::tempdir().unwrap();
        let context = context_for_project(temp.path());

        let result =
            init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context).unwrap();

        assert!(result.warnings.is_empty());
        assert!(!temp.path().join(".agents").exists());
        assert!(!temp.path().join(".claude").exists());
        assert!(!temp.path().join(".cline").exists());
    }

    #[cfg(unix)]
    #[test]
    fn discovery_location_scan_failure_is_reported_as_warning_after_config_creation() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let agents_skills = temp.path().join(".agents").join("skills");
        fs::create_dir_all(&agents_skills).unwrap();
        let original_permissions = fs::metadata(&agents_skills).unwrap().permissions();
        fs::set_permissions(&agents_skills, fs::Permissions::from_mode(0o000)).unwrap();
        let context = context_for_project(temp.path());

        let result = init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context);

        fs::set_permissions(&agents_skills, original_permissions).unwrap();
        let result = result.unwrap();
        let config_file = temp.path().join("agentcfg.toml");
        assert!(config_file.is_file());
        assert!(
            result.warnings.iter().any(|warning| matches!(
                warning,
                InitWarning::ClientDiscoveryLocationReadFailure(read_failure)
                    if read_failure.path == agents_skills
                        && read_failure.clients == ["codex", "cursor", "opencode", "pi"]
            )),
            "missing scan warning in {:?}",
            result.warnings
        );
    }

    #[cfg(unix)]
    #[test]
    fn discovery_location_scan_metadata_failure_is_reported_as_warning_after_config_creation() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let agents_dir = temp.path().join(".agents");
        let agents_skills = agents_dir.join("skills");
        fs::create_dir_all(&agents_skills).unwrap();
        let original_permissions = fs::metadata(&agents_dir).unwrap().permissions();
        fs::set_permissions(&agents_dir, fs::Permissions::from_mode(0o000)).unwrap();
        let context = context_for_project(temp.path());

        let result = init_with_context(InitRequest::new(ConfigLayer::SharedProject), &context);

        fs::set_permissions(&agents_dir, original_permissions).unwrap();
        let result = result.unwrap();
        let config_file = temp.path().join("agentcfg.toml");
        assert!(config_file.is_file());
        assert!(
            result.warnings.iter().any(|warning| matches!(
                warning,
                InitWarning::ClientDiscoveryLocationReadFailure(read_failure)
                    if read_failure.path == agents_skills
                        && read_failure.clients == ["codex", "cursor", "opencode", "pi"]
            )),
            "missing scan warning in {:?}",
            result.warnings
        );
    }

    fn context_for_project(project_root: &Path) -> WorkflowContext {
        WorkflowContext::new(
            project_root,
            None::<PathBuf>,
            Some(project_root.join("home")),
        )
    }

    fn assert_config_layer(path: &Path, layer: ConfigLayer) {
        let contents = fs::read_to_string(path).unwrap();
        assert_eq!(
            parse_config_str(layer, path, &contents).unwrap().layer(),
            layer
        );
    }

    fn unmanaged_artifacts(result: &InitResult) -> Vec<UnmanagedArtifact> {
        result
            .warnings
            .iter()
            .filter_map(|warning| match warning {
                InitWarning::UnmanagedArtifact(artifact) => Some(artifact.clone()),
                InitWarning::ClientDiscoveryLocationReadFailure(_) => None,
            })
            .collect()
    }

    fn assert_artifact(artifacts: &[UnmanagedArtifact], clients: &[&str], path: &Path) {
        assert!(
            artifacts.iter().any(|artifact| artifact.clients == clients
                && artifact.install_level == InstallLevel::Project
                && artifact.path == path),
            "missing artifact for {clients:?} at {} in {artifacts:?}",
            path.display()
        );
    }
}
