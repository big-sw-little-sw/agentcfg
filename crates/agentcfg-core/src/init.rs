//! Init workflow for establishing Project Markers in non-git Projects.

use std::path::{Path, PathBuf};

use crate::locations::{
    discover_project_root, has_project_markers, project_local_config_dir, ProjectRootError,
};
use crate::{Diagnostic, WorkflowResult, WorkflowStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitRequest {
    pub cwd: PathBuf,
    pub explicit_project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InitData {
    pub project_root: PathBuf,
    pub created_markers: bool,
}

pub fn init(request: InitRequest) -> WorkflowResult<InitData> {
    let discovered = discover_project_root(&request.cwd);
    let project_root = match resolve_init_target(
        &request.cwd,
        request.explicit_project_root.as_deref(),
        &discovered,
    ) {
        Ok(root) => root,
        Err(blocker) => {
            return WorkflowResult {
                workflow: "init",
                status: WorkflowStatus::Success,
                diagnostics: Vec::new(),
                blockers: vec![blocker],
                suggested_actions: Vec::new(),
                progress_events: Vec::new(),
                data: InitData {
                    project_root: request.cwd,
                    created_markers: false,
                },
            };
        }
    };

    let marker_dir = project_local_config_dir(&project_root);
    let had_markers = has_project_markers(&project_root);
    let created_markers = !marker_dir.exists();

    if let Err(error) = std::fs::create_dir_all(&marker_dir) {
        return WorkflowResult {
            workflow: "init",
            status: WorkflowStatus::Success,
            diagnostics: Vec::new(),
            blockers: vec![init_write_blocker(&project_root, error)],
            suggested_actions: Vec::new(),
            progress_events: Vec::new(),
            data: InitData {
                project_root,
                created_markers: false,
            },
        };
    }

    WorkflowResult {
        workflow: "init",
        status: WorkflowStatus::Success,
        diagnostics: Vec::new(),
        blockers: Vec::new(),
        suggested_actions: Vec::new(),
        progress_events: Vec::new(),
        data: InitData {
            project_root,
            created_markers: created_markers && !had_markers,
        },
    }
}

fn resolve_init_target(
    cwd: &Path,
    explicit_project_root: Option<&Path>,
    discovered: &crate::locations::DiscoveredProjectRoot,
) -> Result<PathBuf, Diagnostic> {
    if let Some(root) = explicit_project_root {
        if !root.exists() {
            return Err(project_root_blocker(ProjectRootError::NotFound(
                root.to_path_buf(),
            )));
        }
        if !root.is_dir() {
            return Err(project_root_blocker(ProjectRootError::NotDirectory(
                root.to_path_buf(),
            )));
        }
        return Ok(root.to_path_buf());
    }

    if discovered.anchor.is_none() {
        return Ok(cwd.to_path_buf());
    }

    Ok(discovered.root.clone())
}

fn project_root_blocker(error: ProjectRootError) -> Diagnostic {
    Diagnostic {
        code: "project-root-invalid".to_string(),
        message: format!("Cannot resolve Project Root: {error}"),
        context: Vec::new(),
        suggested_actions: Vec::new(),
    }
}

fn init_write_blocker(project_root: &Path, error: std::io::Error) -> Diagnostic {
    Diagnostic {
        code: "init-write-failed".to_string(),
        message: format!(
            "Cannot create project markers at {}: {error}",
            project_root.display()
        ),
        context: vec![(
            "project-root".to_string(),
            project_root.display().to_string(),
        )],
        suggested_actions: Vec::new(),
    }
}
