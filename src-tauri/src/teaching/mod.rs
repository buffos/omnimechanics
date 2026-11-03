//! Educational Structural Mechanics Studio Core Module.

pub mod fraction;
pub mod types;
pub mod isostatic_beam;
pub mod truss;
pub mod cross_kani;
pub mod presets;
pub mod classifier;
pub mod handoff;
pub mod persistence;

#[cfg(test)]
pub mod classifier_test;

pub use types::*;
pub use presets::solve_preset;
pub use classifier::{classify_model, ClassifyModelRequest, MethodEligibility, MethodStatusDto};
pub use handoff::{open_step_by_step, preset_to_structural_model, OpenStepByStepRequest, OpenStepByStepResponse, PresetToModelRequest, PresetToModelResponse};
pub use persistence::{load_workspace, save_workspace, LoadWorkspaceRequest, ProjectDocument, SaveWorkspaceRequest};

pub fn solve_teaching_system(config: TeachingInputConfig) -> Result<TeachingSolutionDto, String> {
    solve_preset(&config)
}
