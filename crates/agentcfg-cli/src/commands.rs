use std::collections::BTreeMap;
use std::path::PathBuf;

use agentcfg_core::workflow::{
    self, ConfigLayer, DoctorRequest, InitRequest, InitWarning, InstallScope, PlanRequest,
    PruneRequest, SourceResolutionPolicy, StatusRequest, SyncRequest, UnmanagedClientArtifact,
};

use crate::CliError;
use crate::args::{Cli, CliCommand, InitArgs};

pub(crate) fn handle(cli: Cli) -> Result<(), CliError> {
    match workflow_invocation_for(cli.command) {
        WorkflowInvocation::Init(request) => render_init_result(&workflow::init(request)?)?,
        WorkflowInvocation::Plan(request) => workflow::plan(request).map(|_| ())?,
        WorkflowInvocation::Sync(request) => workflow::sync(request).map(|_| ())?,
        WorkflowInvocation::Prune(request) => workflow::prune(request).map(|_| ())?,
        WorkflowInvocation::Status(request) => workflow::status(request).map(|_| ())?,
        WorkflowInvocation::Doctor(request) => workflow::doctor(request).map(|_| ())?,
    }

    Ok(())
}

fn render_init_result(result: &workflow::InitResult) -> Result<(), CliError> {
    println!("Created config: {}", result.config_file.display());

    for (path, error, clients) in scan_failure_warnings(&result.warnings) {
        eprintln!(
            "warning: could not scan client target at {} for {}: {}",
            path.display(),
            clients.join(", "),
            error
        );
    }

    for (path, clients) in unmanaged_artifact_warnings(&result.warnings) {
        eprintln!(
            "warning: unmanaged skill artifact exists at {} ({})",
            path.display(),
            clients.join(", ")
        );
    }

    Ok(())
}

fn unmanaged_artifact_warnings(warnings: &[InitWarning]) -> Vec<(PathBuf, Vec<&'static str>)> {
    let mut by_path = BTreeMap::<PathBuf, Vec<&'static str>>::new();
    for artifact in warnings.iter().filter_map(unmanaged_artifact_warning) {
        by_path
            .entry(artifact.path.clone())
            .or_default()
            .push(artifact.client);
    }

    by_path
        .into_iter()
        .map(|(path, mut clients)| {
            clients.sort_unstable();
            clients.dedup();
            (path, clients)
        })
        .collect()
}

fn scan_failure_warnings(warnings: &[InitWarning]) -> Vec<(PathBuf, String, Vec<&'static str>)> {
    let mut by_path_and_error = BTreeMap::<(PathBuf, String), Vec<&'static str>>::new();
    for scan_failure in warnings.iter().filter_map(scan_failure_warning) {
        by_path_and_error
            .entry((scan_failure.path.clone(), scan_failure.error.clone()))
            .or_default()
            .push(scan_failure.client);
    }

    by_path_and_error
        .into_iter()
        .map(|((path, error), mut clients)| {
            clients.sort_unstable();
            clients.dedup();
            (path, error, clients)
        })
        .collect()
}

fn unmanaged_artifact_warning(warning: &InitWarning) -> Option<&UnmanagedClientArtifact> {
    match warning {
        InitWarning::UnmanagedClientArtifact(artifact) => Some(artifact),
        _ => None,
    }
}

fn scan_failure_warning(
    warning: &InitWarning,
) -> Option<&agentcfg_core::workflow::ClientTargetScanFailure> {
    match warning {
        InitWarning::ClientTargetScanFailed(scan_failure) => Some(scan_failure),
        _ => None,
    }
}

#[derive(Debug, Eq, PartialEq)]
enum WorkflowInvocation {
    Init(InitRequest),
    Plan(PlanRequest),
    Sync(SyncRequest),
    Prune(PruneRequest),
    Status(StatusRequest),
    Doctor(DoctorRequest),
}

fn workflow_invocation_for(command: CliCommand) -> WorkflowInvocation {
    match command {
        CliCommand::Init(args) => {
            WorkflowInvocation::Init(InitRequest::new(init_config_layer(args)))
        }
        CliCommand::Plan(args) => WorkflowInvocation::Plan(PlanRequest::new(
            install_scope(args.user),
            source_resolution_policy(args.upgrade),
        )),
        CliCommand::Sync(args) => WorkflowInvocation::Sync(SyncRequest::new(
            install_scope(args.user),
            source_resolution_policy(args.upgrade),
        )),
        CliCommand::Prune(args) => {
            WorkflowInvocation::Prune(PruneRequest::new(install_scope(args.user)))
        }
        CliCommand::Status(args) => {
            WorkflowInvocation::Status(StatusRequest::new(install_scope(args.user)))
        }
        CliCommand::Doctor => WorkflowInvocation::Doctor(DoctorRequest::new()),
    }
}

fn init_config_layer(args: InitArgs) -> ConfigLayer {
    if args.project {
        ConfigLayer::SharedProject
    } else if args.user {
        ConfigLayer::User
    } else {
        ConfigLayer::UserProject
    }
}

fn install_scope(user: bool) -> InstallScope {
    if user {
        InstallScope::User
    } else {
        InstallScope::Project
    }
}

fn source_resolution_policy(upgrade: bool) -> SourceResolutionPolicy {
    if upgrade {
        SourceResolutionPolicy::RefreshSources
    } else {
        SourceResolutionPolicy::UseLocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn maps_init_forms_to_config_layer() {
        assert_eq!(
            invocation_for(["agentcfg", "init"]),
            WorkflowInvocation::Init(InitRequest::new(ConfigLayer::UserProject))
        );
        assert_eq!(
            invocation_for(["agentcfg", "init", "--project"]),
            WorkflowInvocation::Init(InitRequest::new(ConfigLayer::SharedProject))
        );
        assert_eq!(
            invocation_for(["agentcfg", "init", "--user"]),
            WorkflowInvocation::Init(InitRequest::new(ConfigLayer::User))
        );
    }

    #[test]
    fn maps_plan_forms_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "plan"]),
            WorkflowInvocation::Plan(PlanRequest::new(
                InstallScope::Project,
                SourceResolutionPolicy::UseLocked,
            ))
        );
        assert_eq!(
            invocation_for(["agentcfg", "plan", "--user", "--upgrade"]),
            WorkflowInvocation::Plan(PlanRequest::new(
                InstallScope::User,
                SourceResolutionPolicy::RefreshSources,
            ))
        );
    }

    #[test]
    fn maps_sync_forms_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "sync"]),
            WorkflowInvocation::Sync(SyncRequest::new(
                InstallScope::Project,
                SourceResolutionPolicy::UseLocked,
            ))
        );
        assert_eq!(
            invocation_for(["agentcfg", "sync", "--user", "--upgrade"]),
            WorkflowInvocation::Sync(SyncRequest::new(
                InstallScope::User,
                SourceResolutionPolicy::RefreshSources,
            ))
        );
    }

    #[test]
    fn maps_install_scoped_commands_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "prune"]),
            WorkflowInvocation::Prune(PruneRequest::new(InstallScope::Project))
        );
        assert_eq!(
            invocation_for(["agentcfg", "prune", "--user"]),
            WorkflowInvocation::Prune(PruneRequest::new(InstallScope::User))
        );
        assert_eq!(
            invocation_for(["agentcfg", "status"]),
            WorkflowInvocation::Status(StatusRequest::new(InstallScope::Project))
        );
        assert_eq!(
            invocation_for(["agentcfg", "status", "--user"]),
            WorkflowInvocation::Status(StatusRequest::new(InstallScope::User))
        );
    }

    #[test]
    fn maps_doctor_without_scope() {
        assert_eq!(
            invocation_for(["agentcfg", "doctor"]),
            WorkflowInvocation::Doctor(DoctorRequest::new())
        );
    }

    fn invocation_for<const N: usize>(args: [&str; N]) -> WorkflowInvocation {
        workflow_invocation_for(Cli::parse_from(args).command)
    }
}
