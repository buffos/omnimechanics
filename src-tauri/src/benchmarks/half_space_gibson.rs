use crate::material::strategies::DirectionalGradientField;
use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
use crate::mesh::generators_2d::generate_beam_2d;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::{
    assemble_custom_heterogeneous_dynamic_stiffness_and_mass,
    assemble_custom_reference_dynamic_stiffness_and_mass,
};
use crate::solver::hybrid_solver::{
    solve_laplace_dynamic_operator_splitting, solve_laplace_pure_fem,
};
use crate::solver::talbot::{generate_talbot_nodes, invert_laplace_talbot};
use num_complex::Complex64;
use rayon::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HalfSpaceTrajectoryRecord {
    pub time_s: f64,
    pub baseline_disp_mm: f64,
    pub fem_coarse_disp_mm: f64,
    pub fem_fine_disp_mm: f64,
    pub splitting_coarse_disp_mm: f64,
    pub splitting_fine_disp_mm: f64,
}

#[derive(Debug, Clone)]
pub struct DiscretizationAccuracyRecord {
    pub resolution: String,
    pub num_nodes: usize,
    pub total_dofs: usize,
    pub fem_error_pct: f64,
    pub fem_cpu_ms: f64,
    pub splitting_error_pct: f64,
    pub splitting_cpu_ms: f64,
    pub avg_fem_iters: f64,
    pub avg_splitting_iters: f64,
}

#[derive(Debug, Clone)]
pub struct ContrastSensitivityRecord {
    pub alpha: f64,
    pub fem_error_pct: f64,
    pub splitting_error_pct: f64,
    pub splitting_iters: usize,
    pub fem_cpu_ms: f64,
    pub splitting_cpu_ms: f64,
}

/// Solves half-space transient response with surface footing load using Pure FEM + Talbot
pub fn solve_half_space_pure_fem(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    alpha: f64,
    total_time: f64,
    time_steps: usize,
    num_talbot_nodes: usize,
    footing_load: f64,
    footing_width: f64,
    pulse_duration: f64,
) -> (Vec<f64>, Vec<f64>, f64, Vec<usize>) {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    let h = mesh.nodes.iter().map(|p| p[1]).fold(0.0, f64::max).max(1.0);
    let w = mesh.nodes.iter().map(|p| p[0]).fold(0.0, f64::max).max(1.0);

    // Gibson soil strategy: stiffness increases linearly with depth (H - y)
    let strategy = DirectionalGradientField::new(
        Axis::Y,
        alpha,
        h,
        1.0,
        GradientForm::Linear,
    );

    // Fixed bottom bedrock and lateral boundaries
    let fixed_tags = ["Bottom", "Left", "Right"];
    let (k_hetero, m_hetero, fixed_nodes) = assemble_custom_heterogeneous_dynamic_stiffness_and_mass(
        mesh,
        ref_mat,
        &strategy,
        &fixed_tags,
    );

    // Identify surface footing nodes (center of Top boundary, y = H)
    let center_x = w * 0.5;
    let half_b = footing_width * 0.5;
    let mut footing_nodes = Vec::new();
    for bnd in &mesh.boundary_elements {
        if bnd.tag == "Top" {
            for &n in &bnd.nodes {
                let x = mesh.nodes[n][0];
                if (x - center_x).abs() <= half_b && !footing_nodes.contains(&n) {
                    footing_nodes.push(n);
                }
            }
        }
    }
    if footing_nodes.is_empty() {
        // Fallback to top center node
        let mut best_node = 0;
        let mut min_dist = f64::MAX;
        for (i, p) in mesh.nodes.iter().enumerate() {
            if (p[1] - h).abs() < 1e-4 {
                let d = (p[0] - center_x).abs();
                if d < min_dist {
                    min_dist = d;
                    best_node = i;
                }
            }
        }
        footing_nodes.push(best_node);
    }

    let mut base_f_ext = vec![0.0; total_dofs];
    let force_per_node = -footing_load / footing_nodes.len() as f64;
    for &n in &footing_nodes {
        base_f_ext[n * dim + 1] = force_per_node;
    }

    let talbot_nodes = generate_talbot_nodes(num_talbot_nodes.max(16), total_time);
    let t_pulse = pulse_duration.max(total_time * 0.05);

    let results: Vec<(Vec<Complex64>, usize)> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;
            let omega_p = std::f64::consts::PI / t_pulse;
            let pulse_factor = (Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp()))
                / (s * s + Complex64::new(omega_p * omega_p, 0.0));

            let mut f_s = vec![Complex64::new(0.0, 0.0); total_dofs];
            for i in 0..total_dofs {
                f_s[i] = Complex64::new(base_f_ext[i], 0.0) * pulse_factor;
            }

            let (u_s, iters, _) = solve_laplace_pure_fem(
                &k_hetero,
                &m_hetero,
                &fixed_nodes,
                dim,
                s,
                &f_s,
                100,
                1e-7,
            );
            (u_s, iters)
        })
        .collect();

    let u_hat_list: Vec<Vec<Complex64>> = results.iter().map(|(u, _)| u.clone()).collect();
    let iters_list: Vec<usize> = results.iter().map(|(_, iters)| *iters).collect();

    // Time inversion at center footing node
    let target_node = footing_nodes[footing_nodes.len() / 2];
    let n_steps = time_steps.max(1);
    let mut times = Vec::with_capacity(n_steps);
    let mut center_disps = Vec::with_capacity(n_steps);

    for m in 1..=n_steps {
        let t = (m as f64 / n_steps as f64) * total_time;
        times.push(t);
        let u_dofs = invert_laplace_talbot(&talbot_nodes, &u_hat_list, t);
        let uy = u_dofs[target_node * dim + 1].abs() * 1000.0; // mm
        center_disps.push(uy);
    }

    let cpu_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    (times, center_disps, cpu_ms, iters_list)
}

/// Solves half-space transient response using Variational BEM-FEM Operator Splitting + Talbot
pub fn solve_half_space_operator_splitting(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    alpha: f64,
    total_time: f64,
    time_steps: usize,
    num_talbot_nodes: usize,
    footing_load: f64,
    footing_width: f64,
    pulse_duration: f64,
    omega: f64,
) -> (Vec<f64>, Vec<f64>, f64, Vec<usize>) {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    let h = mesh.nodes.iter().map(|p| p[1]).fold(0.0, f64::max).max(1.0);
    let w = mesh.nodes.iter().map(|p| p[0]).fold(0.0, f64::max).max(1.0);

    let strategy = DirectionalGradientField::new(
        Axis::Y,
        alpha,
        h,
        1.0,
        GradientForm::Linear,
    );

    let fixed_tags = ["Bottom", "Left", "Right"];
    let (k0_mat, m0_mat, fixed_nodes) = assemble_custom_reference_dynamic_stiffness_and_mass(
        mesh,
        ref_mat,
        &fixed_tags,
    );
    let (k_hetero, m_hetero, _) = assemble_custom_heterogeneous_dynamic_stiffness_and_mass(
        mesh,
        ref_mat,
        &strategy,
        &fixed_tags,
    );

    let center_x = w * 0.5;
    let half_b = footing_width * 0.5;
    let mut footing_nodes = Vec::new();
    for bnd in &mesh.boundary_elements {
        if bnd.tag == "Top" {
            for &n in &bnd.nodes {
                let x = mesh.nodes[n][0];
                if (x - center_x).abs() <= half_b && !footing_nodes.contains(&n) {
                    footing_nodes.push(n);
                }
            }
        }
    }
    if footing_nodes.is_empty() {
        let mut best_node = 0;
        let mut min_dist = f64::MAX;
        for (i, p) in mesh.nodes.iter().enumerate() {
            if (p[1] - h).abs() < 1e-4 {
                let d = (p[0] - center_x).abs();
                if d < min_dist {
                    min_dist = d;
                    best_node = i;
                }
            }
        }
        footing_nodes.push(best_node);
    }

    let mut base_f_ext = vec![0.0; total_dofs];
    let force_per_node = -footing_load / footing_nodes.len() as f64;
    for &n in &footing_nodes {
        base_f_ext[n * dim + 1] = force_per_node;
    }

    let talbot_nodes = generate_talbot_nodes(num_talbot_nodes.max(16), total_time);
    let t_pulse = pulse_duration.max(total_time * 0.05);

    let results: Vec<(Vec<Complex64>, usize)> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;
            let omega_p = std::f64::consts::PI / t_pulse;
            let pulse_factor = (Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp()))
                / (s * s + Complex64::new(omega_p * omega_p, 0.0));

            let mut f_s = vec![Complex64::new(0.0, 0.0); total_dofs];
            for i in 0..total_dofs {
                f_s[i] = Complex64::new(base_f_ext[i], 0.0) * pulse_factor;
            }

            let (u_s, iters, _) = solve_laplace_dynamic_operator_splitting(
                &k0_mat,
                &m0_mat,
                &k_hetero,
                &m_hetero,
                &fixed_nodes,
                dim,
                s,
                &f_s,
                30,
                1e-5,
                omega,
            );
            (u_s, iters)
        })
        .collect();

    let u_hat_list: Vec<Vec<Complex64>> = results.iter().map(|(u, _)| u.clone()).collect();
    let iters_list: Vec<usize> = results.iter().map(|(_, iters)| *iters).collect();

    let target_node = footing_nodes[footing_nodes.len() / 2];
    let n_steps = time_steps.max(1);
    let mut times = Vec::with_capacity(n_steps);
    let mut center_disps = Vec::with_capacity(n_steps);

    for m in 1..=n_steps {
        let t = (m as f64 / n_steps as f64) * total_time;
        times.push(t);
        let u_dofs = invert_laplace_talbot(&talbot_nodes, &u_hat_list, t);
        let uy = u_dofs[target_node * dim + 1].abs() * 1000.0; // mm
        center_disps.push(uy);
    }

    let cpu_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    (times, center_disps, cpu_ms, iters_list)
}

/// Computes L2 relative error norm between test trajectory and baseline
fn compute_l2_relative_error(test: &[f64], baseline: &[f64]) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for (t, b) in test.iter().zip(baseline.iter()) {
        num += (t - b).powi(2);
        den += b.powi(2);
    }
    (num / den.max(1e-12)).sqrt() * 100.0 // in percent
}

#[cfg(test)]
#[path = "half_space_gibson_test.rs"]
mod tests;
