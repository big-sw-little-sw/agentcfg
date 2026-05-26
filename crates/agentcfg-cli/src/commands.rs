use agentcfg_core::workflow::{
    self, ConfigLayer, DoctorRequest, InitRequest, InstallScope, PlanRequest, PruneRequest,
    SourceResolutionPolicy, StatusRequest, SyncRequest,
};

use crate::args::{Cli, CliCommand, ClientScopeArgs, InitArgs, WorkflowScopeArgs};
use crate::render;
use crate::CliError;

pub(crate) fn handle(cli: Cli) -> Result<(), CliError> {
    match workflow_invocation_for(cli.command) {
        WorkflowInvocation::Init(request) => {
            render::render_init_result(&workflow::init(request)?)?
        }
        WorkflowInvocation::Plan(request) => workflow::plan(request).map(|_| ())?,
        WorkflowInvocation::Sync(request) => workflow::sync(request).map(|_| ())?,
        WorkflowInvocation::Prune(request) => workflow::prune(request).map(|_| ())?,
        WorkflowInvocation::Status(request) => workflow::status(request).map(|_| ())?,
        WorkflowInvocation::Doctor(request) => workflow::doctor(request).map(|_| ())?,
    }

    Ok(())
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
        CliCommand::Plan(args) => WorkflowInvocation::Plan(plan_request(args)),
        CliCommand::Sync(args) => WorkflowInvocation::Sync(sync_request(args)),
        CliCommand::Prune(args) => WorkflowInvocation::Prune(prune_request(args)),
        CliCommand::Status(args) => WorkflowInvocation::Status(status_request(args)),
        CliCommand::Doctor => WorkflowInvocation::Doctor(DoctorRequest::new()),
    }
}

fn plan_request(args: WorkflowScopeArgs) -> PlanRequest {
    let mut request = PlanRequest::new(
        install_scope(args.user),
        source_resolution_policy(args.upgrade),
    );
    request.clients = args.clients;
    request
}

fn sync_request(args: WorkflowScopeArgs) -> SyncRequest {
    let mut request = SyncRequest::new(
        install_scope(args.user),
        source_resolution_policy(args.upgrade),
    );
    request.clients = args.clients;
    request
}

fn prune_request(args: ClientScopeArgs) -> PruneRequest {
    let mut request = PruneRequest::new(install_scope(args.user));
    request.clients = args.clients;
    request
}

fn status_request(args: ClientScopeArgs) -> StatusRequest {
    let mut request = StatusRequest::new(install_scope(args.user));
    request.clients = args.clients;
    request
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
        assert_eq!(
            invocation_for(["agentcfg", "plan", "--client", "codex", "--client", "claude"]),
            WorkflowInvocation::Plan({
                let mut request = PlanRequest::new(
                    InstallScope::Project,
                    SourceResolutionPolicy::UseLocked,
                );
                request.clients = vec!["codex".to_string(), "claude".to_string()];
                request
            })
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
