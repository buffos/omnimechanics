use super::dynamic_solver::sp_mat_mul_complex;
use super::fem_solver::{
    assemble_heterogeneous_dynamic_stiffness_and_mass,
    assemble_reference_dynamic_stiffness_and_mass,
    compute_nodal_von_mises,
};
use super::krylov_solver::solve_complex_gmres;
use super::talbot::{generate_talbot_nodes, invert_laplace_talbot};
use super::types::{SimulationFrame, TransientSimulationResult};
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use faer::sparse::SparseColMat;
use num_complex::Complex64;
use rayon::prelude::*;
use std::collections::HashSet;

/// Computes the diagonal inverse preconditioner for a complex dynamic matrix A(s) = K + s^2 * M
pub fn compute_dynamic_diagonal_inverse(
    k_mat: &SparseColMat<usize, f64>,
    m_mat: &SparseColMat<usize, f64>,
    s_sq: Complex64,
) -> Vec<Complex64> {
    let total_dofs = k_mat.nrows();
    let mut diag_inv = vec![Complex64::new(1.0, 0.0); total_dofs];
    for col in 0..total_dofs {
        let mut k_diag = 0.0;
        let mut m_diag = 0.0;
        for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
            if row == col {
                k_diag = val;
                break;
            }
        }
        for (row, &val) in m_mat.row_indices_of_col(col).zip(m_mat.values_of_col(col)) {
            if row == col {
                m_diag = val;
                break;
            }
        }
        let total = Complex64::new(k_diag, 0.0) + s_sq * m_diag;
        if total.norm() > 1e-10 {
            diag_inv[col] = Complex64::new(1.0, 0.0) / total;
        }
    }
    diag_inv
}

/// Solves a single complex Laplace frequency system via Pure FEM:
///   (K_hetero + s^2 * M_hetero) * u(s) = F(s)
/// using diagonally-preconditioned Complex GMRES.
pub fn solve_laplace_pure_fem(
    k_hetero: &SparseColMat<usize, f64>,
    m_hetero: &SparseColMat<usize, f64>,
    fixed_nodes: &HashSet<usize>,
    dim: usize,
    s: Complex64,
    f_s: &[Complex64],
    max_gmres_iter: usize,
    tolerance: f64,
) -> (Vec<Complex64>, usize, f64) {
    let total_dofs = k_hetero.nrows();
    let s_sq = s * s;
    let diag_inv = compute_dynamic_diagonal_inverse(k_hetero, m_hetero, s_sq);

    let mut fixed_dofs = HashSet::new();
    for &node in fixed_nodes {
        for d in 0..dim {
            fixed_dofs.insert(node * dim + d);
        }
    }

    let mut b_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
    for i in 0..total_dofs {
        if !fixed_dofs.contains(&i) {
            b_prec[i] = diag_inv[i] * f_s[i];
        }
    }

    let apply_a = |v: &[Complex64]| {
        let kv = sp_mat_mul_complex(k_hetero, v);
        let mv = sp_mat_mul_complex(m_hetero, v);
        let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..total_dofs {
            let total = kv[i] + s_sq * mv[i];
            out[i] = diag_inv[i] * total;
        }
        out
    };

    let gmres_res = solve_complex_gmres(
        total_dofs,
        &b_prec,
        apply_a,
        max_gmres_iter,
        tolerance,
        max_gmres_iter,
    );

    let mut u = gmres_res.solution;
    for &dof in &fixed_dofs {
        u[dof] = Complex64::new(0.0, 0.0);
    }

    (u, gmres_res.iterations, gmres_res.relative_residual)
}

/// Solves a single complex Laplace frequency system via Dynamic Operator Splitting (BEM Predictor -- FEM Corrector):
///   A_0(s) \Delta u^{(m+1)} = F(s) - A_hetero(s) u^{(m)}
/// where A_0(s) = K_0 + s^2 M_0 is the homogeneous reference operator (BEM),
/// and A_hetero(s) = K_hetero + s^2 M_hetero is the true heterogeneous dynamic operator (FEM).
pub fn solve_laplace_dynamic_operator_splitting(
    k0_mat: &SparseColMat<usize, f64>,
    m0_mat: &SparseColMat<usize, f64>,
    k_hetero: &SparseColMat<usize, f64>,
    m_hetero: &SparseColMat<usize, f64>,
    fixed_nodes: &HashSet<usize>,
    dim: usize,
    s: Complex64,
    f_s: &[Complex64],
    max_iterations: usize,
    tolerance: f64,
    omega: f64,
) -> (Vec<Complex64>, usize, f64) {
    let total_dofs = k0_mat.nrows();
    let s_sq = s * s;
    let diag_a0_inv = compute_dynamic_diagonal_inverse(k0_mat, m0_mat, s_sq);

    let mut fixed_dofs = HashSet::new();
    for &node in fixed_nodes {
        for d in 0..dim {
            fixed_dofs.insert(node * dim + d);
        }
    }

    let apply_a0 = |v: &[Complex64]| {
        let kv = sp_mat_mul_complex(k0_mat, v);
        let mv = sp_mat_mul_complex(m0_mat, v);
        let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..total_dofs {
            let total = kv[i] + s_sq * mv[i];
            out[i] = diag_a0_inv[i] * total;
        }
        out
    };

    let f_norm = f_s.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt().max(1e-12);

    // 1. Initial Predictor Step: Solve on homogeneous reference operator A_0(s) * u^(0) = F(s)
    let mut b_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
    for i in 0..total_dofs {
        if !fixed_dofs.contains(&i) {
            b_prec[i] = diag_a0_inv[i] * f_s[i];
        }
    }

    let pred_res = solve_complex_gmres(total_dofs, &b_prec, apply_a0, 100, 1e-8, 100);
    let mut u = pred_res.solution;
    for &dof in &fixed_dofs {
        u[dof] = Complex64::new(0.0, 0.0);
    }

    let mut final_iter = 1;
    let mut final_rel_err = 1.0;

    // 2. Variational Gauss-Point Dynamic Corrector Loop
    for iter in 1..=max_iterations.max(1) {
        final_iter = iter;

        // Evaluate actual dynamic internal resisting force: F_dyn = K_hetero * u + s^2 * M_hetero * u
        let kv = sp_mat_mul_complex(k_hetero, &u);
        let mv = sp_mat_mul_complex(m_hetero, &u);

        let mut residual = vec![Complex64::new(0.0, 0.0); total_dofs];
        let mut r_norm_sq = 0.0;

        for i in 0..total_dofs {
            if fixed_dofs.contains(&i) {
                residual[i] = Complex64::new(0.0, 0.0);
            } else {
                let f_dyn_i = kv[i] + s_sq * mv[i];
                let r_i = f_s[i] - f_dyn_i;
                residual[i] = r_i;
                r_norm_sq += r_i.norm_sqr();
            }
        }

        let r_norm = r_norm_sq.sqrt();
        let rel_err = r_norm / f_norm;
        final_rel_err = rel_err;

        if rel_err < tolerance {
            break;
        }
        if rel_err > 1e6 || rel_err.is_nan() {
            break;
        }

        // Inner inexact Krylov solve on reference operator: A_0(s) * delta_u = R
        let inner_tol = (0.05 * rel_err).clamp(1e-8, 1e-4);
        let mut r_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                r_prec[i] = diag_a0_inv[i] * residual[i];
            }
        }

        let corr_res = solve_complex_gmres(total_dofs, &r_prec, apply_a0, 80, inner_tol, 80);

        // Relaxed displacement update: u^(m+1) = u^(m) + omega * delta_u
        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                u[i] += omega * corr_res.solution[i];
            }
        }
    }

    (u, final_iter, final_rel_err)
}

/// Solves transient elastodynamics using Dynamic Operator Splitting (BEM Predictor -- FEM Corrector)
/// combined with Talbot contour inversion in the complex Laplace domain.
pub fn solve_transient_dynamic_operator_splitting_talbot(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    total_time: f64,
    time_steps: usize,
    num_talbot_points: usize,
    load_magnitude: f64,
    pulse_duration: f64,
    excitation_type: Option<&str>,
    excitation_frequency: Option<f64>,
    max_splitting_iters: usize,
    tolerance: f64,
    omega: f64,
) -> (TransientSimulationResult, Vec<usize>) {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    // 1. Assemble Reference Homogeneous Operators (BEM Predictor) and Heterogeneous Operators (FEM Corrector)
    let (k0_mat, m0_mat, fixed_nodes) = assemble_reference_dynamic_stiffness_and_mass(mesh, reference);
    let (k_hetero, m_hetero, _) = assemble_heterogeneous_dynamic_stiffness_and_mass(mesh, reference, strategy);

    // 2. Identify loaded boundary nodes
    let mut loaded_nodes_set = HashSet::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                loaded_nodes_set.insert(n);
            }
        }
    }
    let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

    let mut base_f_ext = vec![0.0; total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            base_f_ext[node_idx * dim + 1] = force_per_node;
        }
    }

    // 3. Generate Talbot contour nodes
    let exc_type = excitation_type.unwrap_or("Pulse");
    let exc_freq = excitation_frequency.unwrap_or(3.0);
    let min_pts = if exc_type == "HarmonicSine" { 32 } else { 16 };
    let n_pts = num_talbot_points.max(min_pts);
    let talbot_nodes = generate_talbot_nodes(n_pts, total_time);

    let t_pulse = pulse_duration.max(total_time * 0.05);

    // 4. Parallel solve across Talbot contour points using Dynamic Operator Splitting
    let results: Vec<(Vec<Complex64>, usize)> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;

            let pulse_factor = match exc_type {
                "HarmonicSine" => {
                    let omega_exc = 2.0 * std::f64::consts::PI * exc_freq.max(0.01);
                    Complex64::new(omega_exc, 0.0) / (s * s + Complex64::new(omega_exc * omega_exc, 0.0))
                }
                "Step" => Complex64::new(1.0, 0.0) / s,
                "HalfSine" => {
                    let omega_p = std::f64::consts::PI / t_pulse;
                    let num = Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp());
                    let denom = s * s + Complex64::new(omega_p * omega_p, 0.0);
                    num / denom
                }
                _ => (Complex64::new(1.0, 0.0) - (-s * t_pulse).exp()) / s,
            };

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
                max_splitting_iters,
                tolerance,
                omega,
            );

            (u_s, iters)
        })
        .collect();

    let u_hat_list: Vec<Vec<Complex64>> = results.iter().map(|(u, _)| u.clone()).collect();
    let splitting_iters_per_node: Vec<usize> = results.iter().map(|(_, iters)| *iters).collect();

    // 5. Invert Laplace transform at discrete time frames t_m
    let n_steps = time_steps.max(1);
    let mut time_points = Vec::with_capacity(n_steps);
    let mut frames = Vec::with_capacity(n_steps);
    let mut max_overall_disp = 0.0;

    for m in 1..=n_steps {
        let t = (m as f64 / n_steps as f64) * total_time;
        time_points.push(t);

        let u_dofs = invert_laplace_talbot(&talbot_nodes, &u_hat_list, t);

        let mut displacements = Vec::with_capacity(num_nodes);
        let mut deformed_nodes = Vec::with_capacity(num_nodes);
        let mut frame_max_d = 0.0;

        for i in 0..num_nodes {
            let is_fixed = fixed_nodes.contains(&i);
            let ux = if is_fixed { 0.0 } else { u_dofs[i * dim + 0] };
            let uy = if is_fixed { 0.0 } else { u_dofs[i * dim + 1] };
            let uz = if is_fixed || dim < 3 { 0.0 } else { u_dofs[i * dim + 2] };

            let mag = (ux * ux + uy * uy + uz * uz).sqrt();
            if mag > frame_max_d {
                frame_max_d = mag;
            }

            displacements.push([ux, uy, uz]);
            let orig = mesh.nodes[i];
            deformed_nodes.push([orig[0] + ux, orig[1] + uy, orig[2] + uz]);
        }

        if frame_max_d > max_overall_disp {
            max_overall_disp = frame_max_d;
        }

        let von_mises = compute_nodal_von_mises(mesh, &displacements, reference, strategy);

        frames.push(SimulationFrame {
            time: t,
            displacements,
            deformed_nodes,
            von_mises_stresses: von_mises,
            max_displacement: frame_max_d,
        });
    }

    let sim_res = TransientSimulationResult {
        time_points,
        frames,
        max_overall_displacement: max_overall_disp,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    };

    (sim_res, splitting_iters_per_node)
}

/// Solves transient elastodynamics using Pure FEM combined with Talbot contour inversion in the complex Laplace domain.
pub fn solve_transient_pure_fem_talbot(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    total_time: f64,
    time_steps: usize,
    num_talbot_points: usize,
    load_magnitude: f64,
    pulse_duration: f64,
    excitation_type: Option<&str>,
    excitation_frequency: Option<f64>,
    max_gmres_iter: usize,
    tolerance: f64,
) -> (TransientSimulationResult, Vec<usize>) {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    // 1. Assemble Heterogeneous Dynamic Stiffness & Mass (Pure FEM)
    let (k_hetero, m_hetero, fixed_nodes) = assemble_heterogeneous_dynamic_stiffness_and_mass(mesh, reference, strategy);

    // 2. Identify loaded boundary nodes
    let mut loaded_nodes_set = HashSet::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                loaded_nodes_set.insert(n);
            }
        }
    }
    let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

    let mut base_f_ext = vec![0.0; total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            base_f_ext[node_idx * dim + 1] = force_per_node;
        }
    }

    // 3. Generate Talbot contour nodes
    let exc_type = excitation_type.unwrap_or("Pulse");
    let exc_freq = excitation_frequency.unwrap_or(3.0);
    let min_pts = if exc_type == "HarmonicSine" { 32 } else { 16 };
    let n_pts = num_talbot_points.max(min_pts);
    let talbot_nodes = generate_talbot_nodes(n_pts, total_time);

    let t_pulse = pulse_duration.max(total_time * 0.05);

    // 4. Parallel solve across Talbot contour points using Pure FEM GMRES
    let results: Vec<(Vec<Complex64>, usize)> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;

            let pulse_factor = match exc_type {
                "HarmonicSine" => {
                    let omega_exc = 2.0 * std::f64::consts::PI * exc_freq.max(0.01);
                    Complex64::new(omega_exc, 0.0) / (s * s + Complex64::new(omega_exc * omega_exc, 0.0))
                }
                "Step" => Complex64::new(1.0, 0.0) / s,
                "HalfSine" => {
                    let omega_p = std::f64::consts::PI / t_pulse;
                    let num = Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp());
                    let denom = s * s + Complex64::new(omega_p * omega_p, 0.0);
                    num / denom
                }
                _ => (Complex64::new(1.0, 0.0) - (-s * t_pulse).exp()) / s,
            };

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
                max_gmres_iter,
                tolerance,
            );

            (u_s, iters)
        })
        .collect();

    let u_hat_list: Vec<Vec<Complex64>> = results.iter().map(|(u, _)| u.clone()).collect();
    let gmres_iters_per_node: Vec<usize> = results.iter().map(|(_, iters)| *iters).collect();

    // 5. Invert Laplace transform at discrete time frames t_m
    let n_steps = time_steps.max(1);
    let mut time_points = Vec::with_capacity(n_steps);
    let mut frames = Vec::with_capacity(n_steps);
    let mut max_overall_disp = 0.0;

    for m in 1..=n_steps {
        let t = (m as f64 / n_steps as f64) * total_time;
        time_points.push(t);

        let u_dofs = invert_laplace_talbot(&talbot_nodes, &u_hat_list, t);

        let mut displacements = Vec::with_capacity(num_nodes);
        let mut deformed_nodes = Vec::with_capacity(num_nodes);
        let mut frame_max_d = 0.0;

        for i in 0..num_nodes {
            let is_fixed = fixed_nodes.contains(&i);
            let ux = if is_fixed { 0.0 } else { u_dofs[i * dim + 0] };
            let uy = if is_fixed { 0.0 } else { u_dofs[i * dim + 1] };
            let uz = if is_fixed || dim < 3 { 0.0 } else { u_dofs[i * dim + 2] };

            let mag = (ux * ux + uy * uy + uz * uz).sqrt();
            if mag > frame_max_d {
                frame_max_d = mag;
            }

            displacements.push([ux, uy, uz]);
            let orig = mesh.nodes[i];
            deformed_nodes.push([orig[0] + ux, orig[1] + uy, orig[2] + uz]);
        }

        if frame_max_d > max_overall_disp {
            max_overall_disp = frame_max_d;
        }

        let von_mises = compute_nodal_von_mises(mesh, &displacements, reference, strategy);

        frames.push(SimulationFrame {
            time: t,
            displacements,
            deformed_nodes,
            von_mises_stresses: von_mises,
            max_displacement: frame_max_d,
        });
    }

    let sim_res = TransientSimulationResult {
        time_points,
        frames,
        max_overall_displacement: max_overall_disp,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    };

    (sim_res, gmres_iters_per_node)
}
