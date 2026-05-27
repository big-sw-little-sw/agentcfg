use agentcfg_core::workflow::{
    self, ConfigLayer, DoctorRequest, InitRequest, InitWarning, InstallLevel, PreviewRequest,
    ApplyRequest, PruneRequest, SourceResolutionPolicy, StatusRequest,
};

use crate::CliError;
use crate::args::{Cli, CliCommand, InitArgs};

pub(crate) fn handle(cli: Cli) -> Result<(), CliError> {
    match workflow_invocation_for(cli.command) {
        WorkflowInvocation::Init(request) => render_init_result(&workflow::init(request)?)?,
        WorkflowInvocation::Preview(request) => workflow::preview(request).map(|_| ())?,
        WorkflowInvocation::Apply(request) => workflow::apply(request).map(|_| ())?,
        WorkflowInvocation::Prune(request) => workflow::prune(request).map(|_| ())?,
        WorkflowInvocation::Status(request) => workflow::status(request).map(|_| ())?,
        WorkflowInvocation::Doctor(request) => workflow::doctor(request).map(|_| ())?,
    }

    Ok(())
}

fn render_init_result(result: &workflow::InitResult) -> Result<(), CliError> {
    println!("Created config: {}", result.config_file.display());

    for warning in &result.warnings {
        render_discovery_warning(warning);
    }

    Ok(())
}

fn render_discovery_warning(warning: &InitWarning) {
    match warning {
        InitWarning::DiscoveryLocationReadFailure(read_failure) => {
            eprintln!(
                "warning: could not scan client discovery location at {} for {}: {}",
                read_failure.path.display(),
                read_failure.clients.join(", "),
                read_failure.error
            );
        }
        InitWarning::UnmanagedInstalledArtifact(artifact) => {
            eprintln!(
                "warning: unmanaged installed artifact exists at {} ({})",
                artifact.path.display(),
                artifact.clients.join(", ")
            );
        }
        _ => {}
    }
}

#[derive(Debug, Eq, PartialEq)]
enum WorkflowInvocation {
    Init(InitRequest),
    Preview(PreviewRequest),
    Apply(ApplyRequest),
    Prune(PruneRequest),
    Status(StatusRequest),
    Doctor(DoctorRequest),
}

fn workflow_invocation_for(command: CliCommand) -> WorkflowInvocation {
    match command {
        CliCommand::Init(args) => {
            WorkflowInvocation::Init(InitRequest::new(init_config_layer(args)))
        }
        CliCommand::Preview(args) => WorkflowInvocation::Preview(PreviewRequest::new(
            install_level(args.user),
            source_resolution_policy(args.upgrade),
        )),
        CliCommand::Apply(args) => WorkflowInvocation::Apply(ApplyRequest::new(
            install_level(args.user),
            source_resolution_policy(args.upgrade),
        )),
        CliCommand::Prune(args) => {
            WorkflowInvocation::Prune(PruneRequest::new(install_level(args.user)))
        }
        CliCommand::Status(args) => {
            WorkflowInvocation::Status(StatusRequest::new(install_level(args.user)))
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

fn install_level(user: bool) -> InstallLevel {
    if user {
        InstallLevel::User
    } else {
        InstallLevel::Project
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
    fn maps_preview_forms_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "preview"]),
            WorkflowInvocation::Preview(PreviewRequest::new(
                InstallLevel::Project,
                SourceResolutionPolicy::UseLocked,
            ))
        );
        assert_eq!(
            invocation_for(["agentcfg", "preview", "--user", "--upgrade"]),
            WorkflowInvocation::Preview(PreviewRequest::new(
                InstallLevel::User,
                SourceResolutionPolicy::RefreshSources,
            ))
        );
    }

    #[test]
    fn maps_apply_forms_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "apply"]),
            WorkflowInvocation::Apply(ApplyRequest::new(
                InstallLevel::Project,
                SourceResolutionPolicy::UseLocked,
            ))
        );
        assert_eq!(
            invocation_for(["agentcfg", "apply", "--user", "--upgrade"]),
            WorkflowInvocation::Apply(ApplyRequest::new(
                InstallLevel::User,
                SourceResolutionPolicy::RefreshSources,
            ))
        );
    }

    #[test]
    fn maps_install_level_commands_to_workflow_request() {
        assert_eq!(
            invocation_for(["agentcfg", "prune"]),
            WorkflowInvocation::Prune(PruneRequest::new(InstallLevel::Project))
        );
        assert_eq!(
            invocation_for(["agentcfg", "prune", "--user"]),
            WorkflowInvocation::Prune(PruneRequest::new(InstallLevel::User))
        );
        assert_eq!(
            invocation_for(["agentcfg", "status"]),
            WorkflowInvocation::Status(StatusRequest::new(InstallLevel::Project))
        );
        assert_eq!(
            invocation_for(["agentcfg", "status", "--user"]),
            WorkflowInvocation::Status(StatusRequest::new(InstallLevel::User))
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
