//! Command use cases that compose config, locking, inventory, and reconciliation.

pub mod apply;
pub mod doctor;
pub mod init;
pub mod preview;
pub mod prune;
pub mod status;

/// Whether a workflow reached executor mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandExecutionOutcome<T> {
    BlockedBeforeExecution,
    Executed(T),
}
