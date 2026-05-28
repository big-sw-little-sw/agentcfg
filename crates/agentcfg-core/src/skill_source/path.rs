//! Path **Skill Source** root resolution and bounded skill directory discovery.
//!
//! **Walk keys** (internal only): `canonicalize`, `canonical_root`, and
//! `is_under_canonical_root` use canonical paths for bounds checks. On case-insensitive
//! volumes (e.g. macOS APFS), `canonicalize` may fold path segment casing; those paths
//! must never be written into persisted or user-facing outputs.
//!
//! **Stored paths** (case-sensitive strings): [`DiscoveredSkill::skill_dir`],
//! [`DiscoveredSkill::source_skill_name`], and [`SkillSourceError`] path fields keep the
//! literal spelling from config and from `read_dir` / `entry.path()` during the walk.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{Error, Result, SkillSourceError};

const SKILL_FILE: &str = "SKILL.md";
const DEFAULT_DISCOVERY_DEPTH: u8 = 4;
const MAX_DISCOVERY_DEPTH: u8 = 8;

/// Validated `[[skill_sources]].discovery_depth`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryDepth(u8);

impl DiscoveryDepth {
    pub const DEFAULT: Self = Self(DEFAULT_DISCOVERY_DEPTH);
    pub const MAX: u8 = MAX_DISCOVERY_DEPTH;

    pub const fn as_u8(self) -> u8 {
        self.0
    }

    pub fn try_from_u8(value: u8) -> Option<Self> {
        if value == 0 || value > Self::MAX {
            None
        } else {
            Some(Self(value))
        }
    }
}

/// Path discovery result: resolved configured root plus discovered skill inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredSkillsInPathSource {
    /// Resolved configured Skill Source path (not `canonicalize` output).
    pub resolved_root: PathBuf,
    pub discovered_skills: Vec<DiscoveredSkill>,
}

/// A **Skill** directory discovered under a path **Skill Source** root.
///
/// `skill_dir` and `source_skill_name` use walk-time path spelling (`read_dir` /
/// `entry.path()`), not canonicalized forms, so segment casing matches what was listed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredSkill {
    /// Leaf directory name from the walk path (`skill_dir.file_name()`).
    pub source_skill_name: String,
    /// Skill directory path as encountered during discovery (not `canonicalize`d).
    pub skill_dir: PathBuf,
}

/// Resolve a configured Skill Source `path` relative to the config file parent.
fn resolve_skill_source_root(config_file: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        configured_path.to_path_buf()
    } else {
        config_file
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
            .join(configured_path)
    }
}

/// Discover **Skills** under a path **Skill Source** root.
///
/// The configured path is resolved relative to the config file parent before discovery.
/// Traversal uses canonical paths only for internal walk decisions (root bounds).
/// Returned [`DiscoveredSkill`] paths and names preserve case-sensitive spelling from
/// the directory walk, not from `canonicalize`.
///
/// Symlink directory entries below the Skill Source root are skipped in M2.1.
pub fn discover_skills_in_source(
    skill_source_id: &str,
    config_file: &Path,
    configured_path: &Path,
    discovery_depth: DiscoveryDepth,
) -> Result<DiscoveredSkillsInPathSource> {
    let resolved_root = resolve_skill_source_root(config_file, configured_path);
    validate_skill_source_root(skill_source_id, configured_path, &resolved_root)?;

    let canonical_root = fs::canonicalize(&resolved_root).map_err(|source| Error::Io {
        path: resolved_root.clone(),
        source,
    })?;

    let mut discovered_skills = Vec::new();
    // Walk keys only — canonical paths may fold case on case-insensitive volumes
    // (macOS APFS). Never store these in `DiscoveredSkill` or error path fields.
    scan_directory(
        skill_source_id,
        &canonical_root,
        &resolved_root,
        0,
        discovery_depth,
        &mut discovered_skills,
    )?;

    check_duplicate_source_skill_names(skill_source_id, &discovered_skills)?;

    discovered_skills.sort_by(|left, right| left.source_skill_name.cmp(&right.source_skill_name));
    Ok(DiscoveredSkillsInPathSource {
        resolved_root,
        discovered_skills,
    })
}

fn validate_skill_source_root(
    skill_source_id: &str,
    configured_path: &Path,
    resolved_root: &Path,
) -> Result<()> {
    let metadata = match fs::metadata(resolved_root) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(not_found(skill_source_id, configured_path, resolved_root));
        }
        Err(source) => {
            return Err(Error::Io {
                path: resolved_root.to_path_buf(),
                source,
            });
        }
    };

    if !metadata.is_dir() {
        return Err(SkillSourceError::NotDirectory {
            skill_source_id: skill_source_id.to_string(),
            configured_path: configured_path.to_path_buf(),
            resolved_path: resolved_root.to_path_buf(),
        }
        .into());
    }

    Ok(())
}

fn not_found(skill_source_id: &str, configured_path: &Path, resolved_path: &Path) -> Error {
    SkillSourceError::NotFound {
        skill_source_id: skill_source_id.to_string(),
        configured_path: configured_path.to_path_buf(),
        resolved_path: resolved_path.to_path_buf(),
    }
    .into()
}

fn scan_directory(
    skill_source_id: &str,
    canonical_root: &Path,
    current_dir: &Path,
    depth: u8,
    discovery_depth: DiscoveryDepth,
    discovered: &mut Vec<DiscoveredSkill>,
) -> Result<()> {
    if depth > 0 && is_hidden_directory(current_dir) {
        return Ok(());
    }

    if is_skill_directory(current_dir) {
        let canonical_skill = fs::canonicalize(current_dir).map_err(|source| Error::Io {
            path: current_dir.to_path_buf(),
            source,
        })?;
        if !is_under_canonical_root(&canonical_skill, canonical_root) {
            return Ok(());
        }
        let source_skill_name = source_skill_name_from_dir(skill_source_id, current_dir)?;
        discovered.push(DiscoveredSkill {
            source_skill_name,
            skill_dir: current_dir.to_path_buf(),
        });
        return Ok(());
    }

    if depth >= discovery_depth.as_u8() {
        return Ok(());
    }

    let canonical_dir = fs::canonicalize(current_dir).map_err(|source| Error::Io {
        path: current_dir.to_path_buf(),
        source,
    })?;
    if !is_under_canonical_root(&canonical_dir, canonical_root) {
        return Ok(());
    }

    let entries = fs::read_dir(current_dir).map_err(|source| Error::Io {
        path: current_dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: current_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();

        if !path.is_dir() || is_symlink_directory(&path) {
            continue;
        }

        scan_directory(
            skill_source_id,
            canonical_root,
            &path,
            depth + 1,
            discovery_depth,
            discovered,
        )?;
    }

    Ok(())
}

fn is_under_canonical_root(canonical: &Path, canonical_root: &Path) -> bool {
    canonical == canonical_root || canonical.starts_with(canonical_root)
}

fn is_symlink_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn source_skill_name_from_dir(skill_source_id: &str, skill_dir: &Path) -> Result<String> {
    if let Some(name) = skill_dir.file_name().and_then(OsStr::to_str) {
        return Ok(name.to_string());
    }

    for component in skill_dir.components().rev() {
        let Component::Normal(name) = component else {
            continue;
        };
        return match name.to_str() {
            Some(utf8) => Ok(utf8.to_string()),
            None => Err(SkillSourceError::NonUtf8SourceSkillName {
                skill_source_id: skill_source_id.to_string(),
                skill_dir: skill_dir.to_path_buf(),
            }
            .into()),
        };
    }

    let canonical = fs::canonicalize(skill_dir).map_err(|source| Error::Io {
        path: skill_dir.to_path_buf(),
        source,
    })?;
    match canonical.file_name().and_then(OsStr::to_str) {
        Some(name) => Ok(name.to_string()),
        None => Err(SkillSourceError::NonUtf8SourceSkillName {
            skill_source_id: skill_source_id.to_string(),
            skill_dir: skill_dir.to_path_buf(),
        }
        .into()),
    }
}

fn is_hidden_directory(dir: &Path) -> bool {
    dir.file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

fn is_skill_directory(dir: &Path) -> bool {
    dir.join(SKILL_FILE).is_file()
}

fn check_duplicate_source_skill_names(
    skill_source_id: &str,
    discovered: &[DiscoveredSkill],
) -> Result<()> {
    let mut by_name: BTreeMap<&str, Vec<PathBuf>> = BTreeMap::new();

    for skill in discovered {
        by_name
            .entry(skill.source_skill_name.as_str())
            .or_default()
            .push(skill.skill_dir.clone());
    }

    for (source_skill_name, mut skill_dirs) in by_name {
        if skill_dirs.len() <= 1 {
            continue;
        }

        skill_dirs.sort();
        return Err(SkillSourceError::DuplicateSourceSkillName {
            skill_source_id: skill_source_id.to_string(),
            source_skill_name: source_skill_name.to_string(),
            skill_dirs,
        }
        .into());
    }

    Ok(())
}

#[cfg(test)]
/// Segment count from `resolved_root` to `skill_dir` (skill directory depth).
fn skill_directory_depth(resolved_root: &Path, skill_dir: &Path) -> Option<u8> {
    let relative = skill_dir.strip_prefix(resolved_root).ok()?;
    let segments = relative
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .count();
    u8::try_from(segments).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn temp_skill_source() -> (TempDir, PathBuf) {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path().join("skills");
        fs::create_dir_all(&root).unwrap();
        (tempdir, root)
    }

    fn write_skill(skill_dir: &Path) {
        fs::create_dir_all(skill_dir).unwrap();
        fs::write(skill_dir.join(SKILL_FILE), "skill").unwrap();
    }

    fn discover(
        skill_source_id: &str,
        configured_path: &Path,
        discovery_depth: u8,
    ) -> Result<DiscoveredSkillsInPathSource> {
        let config_dir = tempfile::tempdir().unwrap();
        let config_file = config_dir.path().join("agentcfg.toml");

        discover_skills_in_source(
            skill_source_id,
            &config_file,
            configured_path,
            DiscoveryDepth::try_from_u8(discovery_depth).unwrap(),
        )
    }

    fn discovered_skills(result: DiscoveredSkillsInPathSource) -> Vec<DiscoveredSkill> {
        result.discovered_skills
    }

    #[test]
    fn path_skill_source_discovery_depth_validates_bounds() {
        assert_eq!(DiscoveryDepth::try_from_u8(1).unwrap().as_u8(), 1);
        assert_eq!(DiscoveryDepth::try_from_u8(8).unwrap().as_u8(), 8);
        assert!(DiscoveryDepth::try_from_u8(0).is_none());
        assert!(DiscoveryDepth::try_from_u8(9).is_none());
    }

    #[test]
    fn path_skill_source_discovery_flat_layout() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("foo"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "foo");
        assert_eq!(skills[0].skill_dir, root.join("foo"));
    }

    #[test]
    fn path_skill_source_discovery_skill_at_source_root() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root);

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(
            skills[0].source_skill_name,
            root.file_name().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn path_skill_source_discovery_hidden_dot_root_is_scanned() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path().join(".skills");
        fs::create_dir_all(&root).unwrap();
        write_skill(&root.join("from-dot-root"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "from-dot-root");
    }

    #[test]
    fn path_skill_source_discovery_hidden_agent_skills_root_is_scanned() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path().join(".agent-skills");
        fs::create_dir_all(&root).unwrap();
        write_skill(&root.join("agent-skill"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "agent-skill");
    }

    #[test]
    fn path_skill_source_discovery_config_parent_dot_path() {
        let tempdir = tempfile::tempdir().unwrap();
        let config_dir = tempdir.path().join("project");
        fs::create_dir_all(&config_dir).unwrap();
        write_skill(&config_dir);

        let config_file = config_dir.join("agentcfg.toml");
        let configured = Path::new(".");
        let result = discover_skills_in_source(
            "personal",
            &config_file,
            configured,
            DiscoveryDepth::try_from_u8(4).unwrap(),
        )
        .unwrap();

        assert_eq!(result.resolved_root, config_dir);
        assert_eq!(result.discovered_skills.len(), 1);
        assert_eq!(result.discovered_skills[0].source_skill_name, "project");
        assert_eq!(result.discovered_skills[0].skill_dir, config_dir);
    }

    #[test]
    fn path_skill_source_discovery_stored_skill_dir_preserves_walk_path_spelling() {
        let (_tempdir, root) = temp_skill_source();
        let skill_dir = root.join("DesignTeam").join("my-skill");
        write_skill(&skill_dir);

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "my-skill");
        let skill_dir_str = skills[0].skill_dir.to_string_lossy();
        assert!(
            skill_dir_str.ends_with("DesignTeam/my-skill"),
            "expected walk-path spelling in skill_dir, got {skill_dir_str}"
        );
    }

    #[test]
    fn path_skill_source_discovery_nested_layout_within_depth() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("design").join("foo"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "foo");
        assert_eq!(skills[0].skill_dir, root.join("design").join("foo"));
        assert_eq!(skill_directory_depth(&root, &skills[0].skill_dir), Some(2));
    }

    #[test]
    fn path_skill_source_discovery_beyond_discovery_depth() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(
            &root
                .join("a")
                .join("b")
                .join("c")
                .join("d")
                .join("e")
                .join("deep"),
        );

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    fn path_skill_source_discovery_within_default_depth_four() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("a").join("b").join("c").join("within"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "within");
    }

    #[test]
    fn path_skill_source_discovery_nested_skill_exclusion() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("foo"));
        write_skill(&root.join("foo").join("bar"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "foo");
    }

    #[test]
    fn path_skill_source_discovery_duplicate_leaf_names() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("a").join("dup"));
        write_skill(&root.join("b").join("dup"));

        let error = discover("personal", &root, 4).unwrap_err();

        assert!(matches!(
            error,
            Error::SkillSource(SkillSourceError::DuplicateSourceSkillName {
                skill_source_id,
                source_skill_name,
                ref skill_dirs,
            }) if skill_source_id == "personal"
                && source_skill_name == "dup"
                && skill_dirs == &[root.join("a").join("dup"), root.join("b").join("dup")]
        ));
    }

    #[test]
    fn path_skill_source_discovery_skips_hidden_directories() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join(".hidden").join("secret"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    fn path_skill_source_discovery_empty_source_directory() {
        let (_tempdir, root) = temp_skill_source();

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    fn path_skill_source_discovery_missing_source_directory() {
        let tempdir = tempfile::tempdir().unwrap();
        let config_dir = tempdir.path().join("config");
        fs::create_dir(&config_dir).unwrap();
        let missing = config_dir.join("../missing");
        let configured = PathBuf::from("../missing");
        let config_file = config_dir.join("agentcfg.toml");
        let error = discover_skills_in_source(
            "personal",
            &config_file,
            &configured,
            DiscoveryDepth::try_from_u8(4).unwrap(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            Error::SkillSource(SkillSourceError::NotFound {
                skill_source_id,
                ref configured_path,
                ref resolved_path,
            }) if skill_source_id == "personal"
                && configured_path == &configured
                && resolved_path == &missing
        ));
    }

    #[test]
    fn path_skill_source_discovery_file_path_is_not_directory() {
        let tempdir = tempfile::tempdir().unwrap();
        let file_path = tempdir.path().join("not-a-dir");
        fs::write(&file_path, "not a directory").unwrap();

        let error = discover("personal", &file_path, 4).unwrap_err();

        assert!(matches!(
            error,
            Error::SkillSource(SkillSourceError::NotDirectory {
                skill_source_id,
                ref configured_path,
                ref resolved_path,
            }) if skill_source_id == "personal"
                && configured_path == &file_path
                && resolved_path == &file_path
        ));
    }

    #[test]
    fn path_skill_source_discovery_relative_path_resolution() {
        let tempdir = tempfile::tempdir().unwrap();
        let config_dir = tempdir.path().join("config");
        let skills_dir = tempdir.path().join("skills");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir.join("rel-skill"));

        let config_file = config_dir.join("agentcfg.toml");
        let configured = Path::new("../skills");
        let result = discover_skills_in_source(
            "personal",
            &config_file,
            configured,
            DiscoveryDepth::try_from_u8(4).unwrap(),
        )
        .unwrap();

        let expected_resolved = config_dir.join("../skills");
        assert_eq!(result.resolved_root, expected_resolved);
        assert_eq!(result.discovered_skills.len(), 1);
        assert_eq!(result.discovered_skills[0].source_skill_name, "rel-skill");
    }

    #[test]
    fn path_skill_source_discovery_absolute_path_resolution() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("abs-skill"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "abs-skill");
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_read_dir_io_error() {
        let (_tempdir, root) = temp_skill_source();
        fs::create_dir(root.join("locked")).unwrap();
        let mut permissions = fs::metadata(root.join("locked")).unwrap().permissions();
        permissions.set_mode(0o000);
        fs::set_permissions(root.join("locked"), permissions).unwrap();

        let error = discover("personal", &root, 4).unwrap_err();

        assert!(matches!(error, Error::Io { .. }));

        let mut permissions = fs::metadata(root.join("locked")).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(root.join("locked"), permissions).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_non_utf8_skill_dir_name() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let bad_name_dir = PathBuf::from(OsString::from_vec(vec![0xff, 0xfe]));

        let error = source_skill_name_from_dir("personal", &bad_name_dir).unwrap_err();

        assert!(matches!(
            error,
            Error::SkillSource(SkillSourceError::NonUtf8SourceSkillName {
                skill_source_id,
                ref skill_dir,
            }) if skill_source_id == "personal" && skill_dir == &bad_name_dir
        ));

        // APFS rejects non-UTF-8 path components; on Linux, exercise full discovery.
        #[cfg(target_os = "linux")]
        {
            let (tempdir, root) = temp_skill_source();
            let bad_skill_dir = root.join(&bad_name_dir);
            fs::create_dir_all(&bad_skill_dir).unwrap();
            fs::write(bad_skill_dir.join(SKILL_FILE), "skill").unwrap();

            let error = discover("personal", &root, 4).unwrap_err();

            assert!(matches!(
                error,
                Error::SkillSource(SkillSourceError::NonUtf8SourceSkillName {
                    skill_source_id,
                    ref skill_dir,
                }) if skill_source_id == "personal" && skill_dir == bad_skill_dir
            ));
            drop(tempdir);
        }
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_symlink_cycle() {
        use std::os::unix::fs::symlink;

        let (_tempdir, root) = temp_skill_source();
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();
        symlink(&dir_b, dir_a.join("to_b")).unwrap();
        symlink(&dir_a, dir_b.join("to_a")).unwrap();

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_symlink_escape_outside_root() {
        use std::os::unix::fs::symlink;

        let tempdir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = tempdir.path().join("skills");
        fs::create_dir_all(&root).unwrap();
        write_skill(&outside.path().join("outside-skill"));
        symlink(outside.path(), root.join("escape")).unwrap();

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_skips_symlink_skill_alias() {
        use std::os::unix::fs::symlink;

        let (_tempdir, root) = temp_skill_source();
        let real = root.join("real");
        write_skill(&real);
        symlink(&real, root.join("alias")).unwrap();

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].source_skill_name, "real");
        assert_eq!(skills[0].skill_dir, real);
    }

    #[test]
    #[cfg(unix)]
    fn path_skill_source_discovery_skips_symlink_directory_without_traversal() {
        use std::os::unix::fs::symlink;

        let (_tempdir, root) = temp_skill_source();
        let outside = tempfile::tempdir().unwrap();
        let shared = outside.path().join("shared");
        fs::create_dir(&shared).unwrap();
        write_skill(&shared.join("deep-only"));
        symlink(&shared, root.join("shallow-link")).unwrap();

        let skills =
            discovered_skills(discover("personal", &root.join("shallow-link"), 2).unwrap());

        assert_eq!(skills.len(), 1);

        let skills = discovered_skills(discover("personal", &root, 2).unwrap());

        assert!(skills.is_empty());
    }

    #[test]
    fn path_skill_source_discovery_sorts_by_source_skill_name() {
        let (_tempdir, root) = temp_skill_source();
        write_skill(&root.join("zebra"));
        write_skill(&root.join("alpha"));

        let skills = discovered_skills(discover("personal", &root, 4).unwrap());

        assert_eq!(
            skills
                .iter()
                .map(|skill| skill.source_skill_name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zebra"]
        );
    }
}
