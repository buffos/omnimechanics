pub mod constant_2d;
pub mod linear_2d;
pub mod quadratic_2d;
pub mod surface_3d;

pub use constant_2d::*;
pub use linear_2d::*;
pub use quadratic_2d::*;
pub use surface_3d::*;

use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::types::{BemElementType, SimulationResult, StressState2D};

pub fn effective_bem_nu(nu0: f64, stress_state: StressState2D) -> f64 {
    match stress_state {
        StressState2D::PlaneStress => nu0 / (1.0 + nu0),
        StressState2D::PlaneStrain => nu0,
    }
}

/// Dispatcher: solves elastostatic problem on homogeneous domain using standalone BEM with specified element type.
pub fn solve_pure_bem(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_with_type(mesh, reference, strategy, load_magnitude, BemElementType::Constant)
}

pub fn solve_pure_bem_with_type(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    elem_type: BemElementType,
) -> SimulationResult {
    solve_pure_bem_with_type_and_stress_state(mesh, reference, strategy, load_magnitude, elem_type, StressState2D::PlaneStress)
}

pub fn solve_pure_bem_with_type_and_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    elem_type: BemElementType,
    stress_state: StressState2D,
) -> SimulationResult {
    if mesh.dimension == 3 {
        return match elem_type {
            BemElementType::Tri3 => solve_pure_bem_tri3_3d(mesh, reference, strategy, load_magnitude),
            _ => {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                let mut min_z = f64::INFINITY;
                let mut max_z = f64::NEG_INFINITY;
                for p in &mesh.nodes {
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
                crate::solver::oraiopoulos_bem_fem::solve_pure_bem_3d(
                    length,
                    height,
                    depth,
                    bnd_nx,
                    bnd_ny,
                    bnd_nz,
                    mesh,
                    reference,
                    load_magnitude,
                )
            }
        };
    }

    match elem_type {
        BemElementType::Constant => solve_pure_bem_constant_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
        BemElementType::Linear => solve_pure_bem_linear_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
        BemElementType::LinearDoubleNodes => solve_pure_bem_linear_dn_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
        BemElementType::QuadraticDiscontinuous => solve_pure_bem_quadratic_discontinuous_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
        BemElementType::Quadratic => solve_pure_bem_quadratic_continuous_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
        BemElementType::Tri3 | BemElementType::Quad4 => solve_pure_bem_constant_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state),
    }
}
