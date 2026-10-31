//! Educational Structural Mechanics Studio Core Module.

pub mod fraction;
pub mod types;
pub mod isostatic_beam;
pub mod truss;
pub mod cross_kani;
pub mod presets;

pub use types::*;
pub use presets::solve_preset;

pub fn solve_teaching_system(config: TeachingInputConfig) -> Result<TeachingSolutionDto, String> {
    solve_preset(&config)
}
