use agentcfg_core::workflow::{
    self, ConfigScope, DoctorRequest, InitRequest, PlanRequest, PruneRequest, SourceResolutionMode,
    StatusRequest, SyncRequest, TargetScope,
};

use crate::CliError;
use crate::args::{Cli, Command, InitArgs};

pub(crate) fn handle(cli: Cli) -> Result<(), CliError> {
    match command_action(cli.command) {
        CommandAction::Init(request) => workflow::init(request).map(|_| ())?,
        CommandAction::Plan(request) => workflow::plan(request).map(|_| ())?,
        CommandAction::Sync(request) => workflow::sync(request).map(|_| ())?,
        CommandAction::Prune(request) => workflow::prune(request).map(|_| ())?,
        CommandAction::Status(request) => workflow::status(request).map(|_| ())?,
        CommandAction::Doctor(request) => workflow::doctor(request).map(|_| ())?,
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum CommandAction {
    Init(InitRequest),
    Plan(PlanRequest),
    Sync(SyncRequest),
    Prune(PruneRequest),
    Status(StatusRequest),
    Doctor(DoctorRequest),
}

fn command_action(command: Command) -> CommandAction {
    match command {
        Command::Init(args) => CommandAction::Init(InitRequest::new(init_scope(args))),
        Command::Plan(args) => CommandAction::Plan(PlanRequest::new(
            target_scope(args.user),
            resolution_mode(args.upgrade),
        )),
        Command::Sync(args) => CommandAction::Sync(SyncRequest::new(
            target_scope(args.user),
            resolution_mode(args.upgrade),
        )),
        Command::Prune(args) => CommandAction::Prune(PruneRequest::new(target_scope(args.user))),
        Command::Status(args) => CommandAction::Status(StatusRequest::new(target_scope(args.user))),
        Command::Doctor => CommandAction::Doctor(DoctorRequest::new()),
    }
}

fn init_scope(args: InitArgs) -> ConfigScope {
    if args.project {
        ConfigScope::Project
    } else if args.user {
        ConfigScope::User
    } else {
        ConfigScope::UserProject
    }
}

fn target_scope(user: bool) -> TargetScope {
    if user {
        TargetScope::User
    } else {
        TargetScope::Project
    }
}

fn resolution_mode(upgrade: bool) -> SourceResolutionMode {
    if upgrade {
        SourceResolutionMode::Upgrade
    } else {
        SourceResolutionMode::Locked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn maps_init_forms_to_config_scope() {
        assert_eq!(
            action_for(["agentcfg", "init"]),
            CommandAction::Init(InitRequest::new(ConfigScope::UserProject))
        );
        assert_eq!(
            action_for(["agentcfg", "init", "--project"]),
            CommandAction::Init(InitRequest::new(ConfigScope::Project))
        );
        assert_eq!(
            action_for(["agentcfg", "init", "--user"]),
            CommandAction::Init(InitRequest::new(ConfigScope::User))
        );
    }

    #[test]
    fn maps_plan_forms_to_workflow_request() {
        assert_eq!(
            action_for(["agentcfg", "plan"]),
            CommandAction::Plan(PlanRequest::new(
                TargetScope::Project,
                SourceResolutionMode::Locked,
            ))
        );
        assert_eq!(
            action_for(["agentcfg", "plan", "--user", "--upgrade"]),
            CommandAction::Plan(PlanRequest::new(
                TargetScope::User,
                SourceResolutionMode::Upgrade,
            ))
        );
    }

    #[test]
    fn maps_sync_forms_to_workflow_request() {
        assert_eq!(
            action_for(["agentcfg", "sync"]),
            CommandAction::Sync(SyncRequest::new(
                TargetScope::Project,
                SourceResolutionMode::Locked,
            ))
        );
        assert_eq!(
            action_for(["agentcfg", "sync", "--user", "--upgrade"]),
            CommandAction::Sync(SyncRequest::new(
                TargetScope::User,
                SourceResolutionMode::Upgrade,
            ))
        );
    }

    #[test]
    fn maps_target_scoped_commands_to_workflow_request() {
        assert_eq!(
            action_for(["agentcfg", "prune"]),
            CommandAction::Prune(PruneRequest::new(TargetScope::Project))
        );
        assert_eq!(
            action_for(["agentcfg", "prune", "--user"]),
            CommandAction::Prune(PruneRequest::new(TargetScope::User))
        );
        assert_eq!(
            action_for(["agentcfg", "status"]),
            CommandAction::Status(StatusRequest::new(TargetScope::Project))
        );
        assert_eq!(
            action_for(["agentcfg", "status", "--user"]),
            CommandAction::Status(StatusRequest::new(TargetScope::User))
        );
    }

    #[test]
    fn maps_doctor_without_scope() {
        assert_eq!(
            action_for(["agentcfg", "doctor"]),
            CommandAction::Doctor(DoctorRequest::new())
        );
    }

    fn action_for<const N: usize>(args: [&str; N]) -> CommandAction {
        command_action(Cli::parse_from(args).command)
    }
}
