//! Project Anchor checks for Project Level workflows.

use crate::{Diagnostic, SuggestedAction, WorkflowContext};

pub fn project_anchor_blocker(context: &WorkflowContext) -> Option<Diagnostic> {
    if context.is_anchored() {
        return None;
    }

    Some(Diagnostic {
        code: "project-unanchored".to_string(),
        message:
            "Cannot run Project Level mutation: working directory is not anchored to a Project."
                .to_string(),
        context: vec![(
            "working-directory".to_string(),
            context.project_root.display().to_string(),
        )],
        suggested_actions: vec![
            SuggestedAction {
                command: "agentcfg init".to_string(),
                reason: "Establish project markers at the current directory.".to_string(),
            },
            SuggestedAction {
                command: "agentcfg <command> --project-root <path>".to_string(),
                reason: "Override Project Root discovery with an explicit path.".to_string(),
            },
        ],
    })
}

pub fn project_unanchored_diagnostic(context: &WorkflowContext) -> Option<Diagnostic> {
    if context.is_anchored() {
        return None;
    }

    Some(Diagnostic {
        code: "project-unanchored".to_string(),
        message: "Working directory is not anchored to a Project; reporting configuration relative to the current directory.".to_string(),
        context: vec![(
            "working-directory".to_string(),
            context.project_root.display().to_string(),
        )],
        suggested_actions: vec![
            SuggestedAction {
                command: "agentcfg init".to_string(),
                reason: "Establish project markers at the current directory.".to_string(),
            },
            SuggestedAction {
                command: "agentcfg <command> --project-root <path>".to_string(),
                reason: "Override Project Root discovery with an explicit path.".to_string(),
            },
        ],
    })
}
