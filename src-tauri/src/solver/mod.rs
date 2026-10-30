pub mod anderson;
pub mod bem_solver;
pub mod bessel;
pub mod boundary_shape;
pub mod oraiopoulos_bem_fem;
pub use oraiopoulos_bem_fem as decoupled_hybrid;
pub mod dynamic_bem;
pub mod dynamic_solver;
pub mod fem_solver;
pub mod hybrid_solver;
pub mod krylov_solver;
pub mod robin_transmission;
pub mod talbot;
pub mod tests;
pub mod time_marching;
pub mod true_bem_fem;
pub mod types;

pub use boundary_shape::BoundaryShapeMatrix;
pub use types::*;

use crate::material::strategies::{
    DirectionalGradientField, InclusionField, J2PlasticityField, MathExpressionField,
    VecchioCollinsField,
};
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::MaterialStrategyConfig;

pub fn create_strategy(strategy: &MaterialStrategyConfig) -> Box<dyn MaterialFieldStrategy> {
    match strategy {
        MaterialStrategyConfig::DirectionalGradient {
            axis,
            alpha,
            length_scale,
            power,
            form,
        } => Box::new(DirectionalGradientField::new(
            *axis,
            *alpha,
            *length_scale,
            *power,
            *form,
        )),

        MaterialStrategyConfig::AnalyticalExpression { expression } => {
            Box::new(MathExpressionField::new(expression.clone()))
        }

        MaterialStrategyConfig::Inclusion {
            shape,
            center,
            radius,
            box_half_extents,
            e_inclusion,
            nu_inclusion,
            rho_inclusion,
            smooth_transition,
        } => Box::new(InclusionField::new(
            *shape,
            *center,
            *radius,
            *box_half_extents,
            *e_inclusion,
            *nu_inclusion,
            *rho_inclusion,
            *smooth_transition,
        )),

        MaterialStrategyConfig::VecchioCollinsConcrete {
            fc_prime,
            epsilon_0,
            ft,
            rebar_ratio,
            simulated_strain,
        } => Box::new(VecchioCollinsField::new(
            *fc_prime,
            *epsilon_0,
            *ft,
            *rebar_ratio,
            *simulated_strain,
        )),

        MaterialStrategyConfig::J2Plasticity {
            sigma_y0,
            hardening_modulus,
            plastic_strain,
        } => Box::new(J2PlasticityField::new(
            *sigma_y0,
            *hardening_modulus,
            *plastic_strain,
        )),
    }
}

pub fn run_simulation(req: SimulationRequest) -> SimulationResult {
    let strategy = create_strategy(&req.strategy);
    let stress_state = req.stress_state.unwrap_or_default();

    match req.solver_mode {
        SolverMode::PureFEM => fem_solver::solve_pure_fem_with_stress_state(
            &req.mesh,
            &req.reference_material,
            &*strategy,
            req.load_magnitude,
            stress_state,
        ),
        SolverMode::PureBEM => {
            if req.mesh.dimension == 3 {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                let mut min_z = f64::INFINITY;
                let mut max_z = f64::NEG_INFINITY;
                for p in &req.mesh.nodes {
                    if p[0] < min_x { min_x = p[0]; }
                    if p[0] > max_x { max_x = p[0]; }
                    if p[1] < min_y { min_y = p[1]; }
                    if p[1] > max_y { max_y = p[1]; }
                    if p[2] < min_z { min_z = p[2]; }
                    if p[2] > max_z { max_z = p[2]; }
                }
                let length = (max_x - min_x).max(1e-3);
                let height = (max_y - min_y).max(1e-3);
                let depth = (max_z - min_z).max(1e-3);
                let bnd_nx = (length / 0.5).round().clamp(2.0, 10.0) as usize;
                let bnd_ny = (height / 0.5).round().clamp(1.0, 4.0) as usize;
                let bnd_nz = (depth / 0.5).round().clamp(1.0, 4.0) as usize;
                oraiopoulos_bem_fem::solve_pure_bem_3d(
                    length,
                    height,
                    depth,
                    bnd_nx,
                    bnd_ny,
                    bnd_nz,
                    &req.mesh,
                    &req.reference_material,
                    req.load_magnitude,
                )
            } else {
                bem_solver::solve_pure_bem_with_type_and_stress_state(
                    &req.mesh,
                    &req.reference_material,
                    &*strategy,
                    req.load_magnitude,
                    req.bem_element_type.unwrap_or_default(),
                    stress_state,
                )
            }
        }
        SolverMode::BEMFEM => {
            if req.mesh.dimension == 2 {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                for p in &req.mesh.nodes {
                    if p[0] < min_x { min_x = p[0]; }
                    if p[0] > max_x { max_x = p[0]; }
                    if p[1] < min_y { min_y = p[1]; }
                    if p[1] > max_y { max_y = p[1]; }
                }
                let length = (max_x - min_x).max(1e-3);
                let height = (max_y - min_y).max(1e-3);
                let bnd_nx = (length / 0.5).round().clamp(4.0, 16.0) as usize;
                let bnd_ny = (height / 0.5).round().clamp(2.0, 8.0) as usize;
                let accel = req.hybrid_acceleration.unwrap_or_default();
                let (res, _, _) = decoupled_hybrid::solve_decoupled_bem_fem_hybrid(
                    length,
                    height,
                    bnd_nx,
                    bnd_ny,
                    &req.mesh,
                    &req.reference_material,
                    &*strategy,
                    req.load_magnitude,
                    req.max_iterations,
                    req.tolerance,
                    accel,
                    stress_state,
                );
                res
            } else {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                let mut min_z = f64::INFINITY;
                let mut max_z = f64::NEG_INFINITY;
                for p in &req.mesh.nodes {
                    if p[0] < min_x { min_x = p[0]; }
                    if p[0] > max_x { max_x = p[0]; }
                    if p[1] < min_y { min_y = p[1]; }
                    if p[1] > max_y { max_y = p[1]; }
                    if p[2] < min_z { min_z = p[2]; }
                    if p[2] > max_z { max_z = p[2]; }
                }
                let length = (max_x - min_x).max(1e-3);
                let height = (max_y - min_y).max(1e-3);
                let depth = (max_z - min_z).max(1e-3);
                let bnd_nx = (length / 0.5).round().clamp(2.0, 10.0) as usize;
                let bnd_ny = (height / 0.5).round().clamp(1.0, 4.0) as usize;
                let bnd_nz = (depth / 0.5).round().clamp(1.0, 4.0) as usize;
                let accel = req.hybrid_acceleration.unwrap_or_default();
                let (res, _, _) = decoupled_hybrid::solve_oraiopoulos_bem_fem_3d(
                    length,
                    height,
                    depth,
                    bnd_nx,
                    bnd_ny,
                    bnd_nz,
                    &req.mesh,
                    &req.reference_material,
                    &*strategy,
                    req.load_magnitude,
                    req.max_iterations,
                    req.tolerance,
                    accel,
                );
                res
            }
        }
    }
}

pub fn run_harmonic_simulation(req: HarmonicSimulationRequest) -> HarmonicSimulationResult {
    let strategy = create_strategy(&req.strategy);
    let mode = req.solver_mode.unwrap_or_else(|| {
        if req.use_robin_coupling.unwrap_or(false) {
            HarmonicSolverMode::Robin
        } else {
            HarmonicSolverMode::PureFEM
        }
    });

    match mode {
        HarmonicSolverMode::PureFEM => {
            dynamic_solver::solve_harmonic(
                &req.mesh,
                &req.reference_material,
                &*strategy,
                req.frequency_hz,
                req.load_magnitude,
                req.max_gmres_iter,
                req.tolerance,
            )
        }
        HarmonicSolverMode::PureBEM => {
            let elem_type = req.bem_element_type.unwrap_or(if req.mesh.dimension == 3 {
                types::BemElementType::Quad4
            } else {
                types::BemElementType::Constant
            });
            dynamic_bem::solve_harmonic_pure_bem_with_type(
                &req.mesh,
                &req.reference_material,
                &*strategy,
                req.frequency_hz,
                req.load_magnitude,
                elem_type,
            )
        }
        HarmonicSolverMode::Robin => {
            robin_transmission::solve_dynamic_robin_hybrid(
                &req.mesh,
                &req.reference_material,
                &*strategy,
                req.frequency_hz,
                req.load_magnitude,
                req.max_gmres_iter,
                req.tolerance.max(0.05),
            )
        }
    }
}

pub fn run_transient_simulation(req: TransientSimulationRequest) -> TransientSimulationResult {
    let strategy = create_strategy(&req.strategy);
    if req.integration_method.as_deref() == Some("Newmark") {
        time_marching::solve_transient_newmark(
            &req.mesh,
            &req.reference_material,
            &*strategy,
            req.total_time,
            req.time_steps,
            req.load_magnitude,
            req.pulse_duration,
            req.excitation_type.as_deref(),
            req.excitation_frequency,
            req.hht_alpha,
            req.rayleigh_alpha,
            req.rayleigh_beta,
        )
    } else if req.solver_mode.as_deref() == Some("PureBEM") {
        let elem_type = req.bem_element_type.unwrap_or(if req.mesh.dimension == 3 {
            types::BemElementType::Quad4
        } else {
            types::BemElementType::Constant
        });
        dynamic_solver::solve_transient_talbot_bem(
            &req.mesh,
            &req.reference_material,
            &*strategy,
            req.total_time,
            req.time_steps,
            req.num_talbot_points,
            req.load_magnitude,
            req.pulse_duration,
            req.excitation_type.as_deref(),
            req.excitation_frequency,
            elem_type,
        )
    } else {
        dynamic_solver::solve_transient_talbot(
            &req.mesh,
            &req.reference_material,
            &*strategy,
            req.total_time,
            req.time_steps,
            req.num_talbot_points,
            req.load_magnitude,
            req.pulse_duration,
            req.excitation_type.as_deref(),
            req.excitation_frequency,
        )
    }
}
