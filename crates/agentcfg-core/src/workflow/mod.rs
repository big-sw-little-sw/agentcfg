//! Command use cases that compose config, resolution, observation, and planning.

pub mod apply;
pub mod doctor;
pub mod init;
pub mod preview;
pub mod prune;
pub mod status;

/// Whether a workflow reached execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandExecutionOutcome<T> {
    BlockedBeforeExecution,
    Executed(T),
}
