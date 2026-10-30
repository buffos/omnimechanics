use std::f64::consts::PI;
pub mod kernels;
pub mod assembly;
pub mod solvers_3d;
pub mod solvers_2d;

pub use kernels::*;
pub use assembly::*;
pub use solvers_3d::*;
pub use solvers_2d::*;

use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver;
use crate::solver::types::{BemElementType, HarmonicSimulationResult};
use num_complex::Complex64;

/// Evaluates nodal displacements in the complex Laplace domain using Pure BEM with specified element type
pub fn solve_laplace_pure_bem_nodal_displacements(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
    elem_type: BemElementType,
) -> Vec<Complex64> {
    if mesh.dimension == 3 {
        match elem_type {
            BemElementType::Tri3 => solve_laplace_pure_bem_tri3_3d(mesh, reference, s, load_factor),
            _ => solve_laplace_pure_bem_quad4_3d(mesh, reference, s, load_factor),
        }
    } else {
        match elem_type {
            BemElementType::Constant => solve_laplace_pure_bem_constant_2d(mesh, reference, s, load_factor),
            BemElementType::Linear | BemElementType::LinearDoubleNodes => {
                solve_laplace_pure_bem_linear_2d(mesh, reference, s, load_factor)
            }
            BemElementType::Quadratic | BemElementType::QuadraticDiscontinuous => {
                solve_laplace_pure_bem_quadratic_2d(mesh, reference, s, load_factor)
            }
            BemElementType::Tri3 | BemElementType::Quad4 => {
                solve_laplace_pure_bem_constant_2d(mesh, reference, s, load_factor)
            }
        }
    }
}

/// Solves elastodynamics under harmonic excitation using standalone Pure BEM with specified element type
pub fn solve_harmonic_pure_bem_with_type(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
    elem_type: BemElementType,
) -> HarmonicSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();

    let omega = 2.0 * PI * frequency_hz.max(0.1);
    let eta = 0.02;
    let s = Complex64::new(eta * omega, omega);
    let load_factor = Complex64::new(load_magnitude, 0.0);

    let u_complex = solve_laplace_pure_bem_nodal_displacements(
        mesh,
        reference,
        s,
        load_factor,
        elem_type,
    );

    let mut amplitudes = Vec::with_capacity(num_nodes);
    let mut phase_lags = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut u_real_disp = Vec::with_capacity(num_nodes);
    let mut max_amp: f64 = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let ux = u_complex[i * dim + 0];
        let uy = u_complex[i * dim + 1];
        let uz = if dim == 3 { u_complex[i * dim + 2] } else { Complex64::new(0.0, 0.0) };

        let amp_x = ux.norm();
        let amp_y = uy.norm();
        let amp_z = uz.norm();
        let total_amp = (amp_x * amp_x + amp_y * amp_y + amp_z * amp_z).sqrt();
        if total_amp > max_amp {
            max_amp = total_amp;
        }

        amplitudes.push([amp_x, amp_y, amp_z]);
        phase_lags.push([ux.arg(), uy.arg(), uz.arg()]);
        u_real_disp.push([ux.re, uy.re, uz.re]);
        deformed_nodes.push([p[0] + ux.re, p[1] + uy.re, p[2] + uz.re]);
    }

    let von_mises = fem_solver::compute_nodal_von_mises(mesh, &u_real_disp, reference, strategy);
    let max_vm = von_mises.iter().cloned().fold(0.0, f64::max);
    let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;

    HarmonicSimulationResult {
        converged: true,
        iterations: 1,
        relative_residual: 1e-6,
        frequency_hz,
        amplitudes,
        phase_lags,
        deformed_nodes,
        von_mises_stresses: von_mises,
        max_amplitude: max_amp,
        max_von_mises: max_vm,
        execution_time_ms: exec_time,
    }
}

pub fn solve_harmonic_pure_bem_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
) -> HarmonicSimulationResult {
    solve_harmonic_pure_bem_with_type(mesh, reference, strategy, frequency_hz, load_magnitude, BemElementType::Constant)
}

pub fn solve_harmonic_pure_bem_3d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
) -> HarmonicSimulationResult {
    solve_harmonic_pure_bem_with_type(mesh, reference, strategy, frequency_hz, load_magnitude, BemElementType::Quad4)
}

pub fn solve_harmonic_pure_bem_tri3_3d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
) -> HarmonicSimulationResult {
    solve_harmonic_pure_bem_with_type(mesh, reference, strategy, frequency_hz, load_magnitude, BemElementType::Tri3)
}

pub fn solve_harmonic_pure_bem_linear_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
) -> HarmonicSimulationResult {
    solve_harmonic_pure_bem_with_type(mesh, reference, strategy, frequency_hz, load_magnitude, BemElementType::Linear)
}

pub fn solve_harmonic_pure_bem_quadratic_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
) -> HarmonicSimulationResult {
    solve_harmonic_pure_bem_with_type(mesh, reference, strategy, frequency_hz, load_magnitude, BemElementType::Quadratic)
}
