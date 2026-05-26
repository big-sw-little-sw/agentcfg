use agentcfg_core::workflow::{
    InitResult, InitWarning, ProjectRootDiscoveryFailed, TargetReadFailure,
};

use crate::CliError;

pub fn render_init_result(result: &InitResult) -> Result<(), CliError> {
    println!("Created config: {}", result.config_file.display());

    for warning in &result.warnings {
        render_init_warning(warning);
    }

    Ok(())
}

fn render_init_warning(warning: &InitWarning) {
    match warning {
        InitWarning::TargetReadFailure(read_failure) => {
            render_target_read_failure(read_failure);
        }
        InitWarning::ExistingTargetArtifact(artifact) => {
            eprintln!(
                "warning: unmanaged skill artifact exists at {} ({})",
                artifact.path.display(),
                artifact.clients.join(", ")
            );
        }
        InitWarning::ProjectRootDiscoveryFailed(failure) => {
            render_project_root_discovery_failed(failure);
        }
        #[allow(unreachable_patterns)]
        _ => eprintln!("warning: {warning:?}"),
    }
}

fn render_target_read_failure(read_failure: &TargetReadFailure) {
    eprintln!(
        "warning: could not scan client target at {} for {}: {}",
        read_failure.path.display(),
        read_failure.clients.join(", "),
        read_failure.error.message
    );
}

fn render_project_root_discovery_failed(failure: &ProjectRootDiscoveryFailed) {
    eprintln!(
        "warning: could not discover project root from {}: {}",
        failure.start_dir.display(),
        failure.error.message
    );
}
