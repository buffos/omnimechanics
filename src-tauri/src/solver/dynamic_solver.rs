use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::{
    assemble_dynamic_stiffness_and_mass, compute_nodal_von_mises,
};
use crate::solver::krylov_solver::solve_complex_gmres;
use crate::solver::talbot::{generate_talbot_nodes, invert_laplace_talbot};
use crate::solver::types::{
    BemElementType, HarmonicSimulationResult, SimulationFrame, TransientSimulationResult,
};
use faer::sparse::SparseColMat;
use num_complex::Complex64;
use rayon::prelude::*;
use std::f64::consts::PI;

/// Multiplies a real SparseColMat by a complex vector: out = M * v
pub fn sp_mat_mul_complex(mat: &SparseColMat<usize, f64>, v: &[Complex64]) -> Vec<Complex64> {
    let n = mat.nrows();
    let mut out = vec![Complex64::new(0.0, 0.0); n];

    for col in 0..mat.ncols() {
        let v_col = v[col];
        if v_col.norm_sqr() < 1e-30 {
            continue;
        }
        for (row, &val) in mat.row_indices_of_col(col).zip(mat.values_of_col(col)) {
            out[row] += v_col * val;
        }
    }

    out
}

/// Solves harmonic steady-state dynamic problem for a given excitation frequency f (Hz)
pub fn solve_harmonic(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    frequency_hz: f64,
    load_magnitude: f64,
    max_gmres_iter: usize,
    tolerance: f64,
) -> HarmonicSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    // 1. Assemble K_0 and M_FEM with exact Dirichlet DOF filtering (no condition number blowup)
    let (k0_mat, m_mat, fixed_nodes) = assemble_dynamic_stiffness_and_mass(mesh, reference, strategy);

    // 2. Identify loaded nodes (e.g. Right boundary) deduplicated
    let mut loaded_nodes_set = std::collections::HashSet::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                loaded_nodes_set.insert(n);
            }
        }
    }
    let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

    // 3. Formulate external complex load vector F(w)
    let mut f_ext = vec![Complex64::new(0.0, 0.0); total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            let dof_y = node_idx * dim + 1;
            f_ext[dof_y] = Complex64::new(force_per_node, 0.0);
        }
    }

    // 4. Set complex Laplace parameter s = eta * w + i * w (with structural damping eta = 0.02)
    let omega = 2.0 * PI * frequency_hz.max(0.1);
    let eta = 0.02;
    let s = Complex64::new(eta * omega, omega);
    let s_sq = s * s;

    // Extract diagonal preconditioner for GMRES: D_ii = K_ii + s^2 * M_ii
    let mut diag_inv = vec![Complex64::new(1.0, 0.0); total_dofs];
    for col in 0..total_dofs {
        let mut k_diag = 0.0;
        let mut m_diag = 0.0;
        for (row, &val) in k0_mat.row_indices_of_col(col).zip(k0_mat.values_of_col(col)) {
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
        if total.norm() > 1e-8 {
            diag_inv[col] = Complex64::new(1.0, 0.0) / total;
        }
    }

    // Right-hand side preconditioned: b_prec = D^{-1} * F_ext
    let mut b_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
    for i in 0..total_dofs {
        b_prec[i] = diag_inv[i] * f_ext[i];
    }

    // Matrix-free operator: A_prec * v = D^{-1} * (K0 * v + s^2 * M * v)
    let apply_a = |v: &[Complex64]| {
        let kv = sp_mat_mul_complex(&k0_mat, v);
        let mv = sp_mat_mul_complex(&m_mat, v);
        let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
        for i in 0..total_dofs {
            let total = kv[i] + s_sq * mv[i];
            out[i] = diag_inv[i] * total;
        }
        out
    };

    // 5. Solve via Complex GMRES (unrestarted for robust Krylov subspace convergence)
    let gmres_res = solve_complex_gmres(
        total_dofs,
        &b_prec,
        apply_a,
        max_gmres_iter.max(120),
        tolerance.max(1e-7),
        max_gmres_iter.max(120),
    );

    // 6. Extract nodal amplitudes, phases, and peak deformed coordinates
    let u_complex = gmres_res.solution;
    let mut amplitudes = Vec::with_capacity(num_nodes);
    let mut phase_lags = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut u_real_disp: Vec<[f64; 3]> = Vec::with_capacity(num_nodes);
    let mut max_amp = 0.0;

    for i in 0..num_nodes {
        let ux = u_complex[i * dim + 0];
        let uy = u_complex[i * dim + 1];
        let uz = if dim == 3 { u_complex[i * dim + 2] } else { Complex64::new(0.0, 0.0) };

        let amp_x = ux.norm();
        let amp_y = uy.norm();
        let amp_z = uz.norm();
        let node_amp = (amp_x * amp_x + amp_y * amp_y + amp_z * amp_z).sqrt();
        if node_amp > max_amp {
            max_amp = node_amp;
        }

        amplitudes.push([amp_x, amp_y, amp_z]);
        phase_lags.push([ux.arg(), uy.arg(), uz.arg()]);

        // Peak instantaneous displacement: Re(u)
        let disp_re = [ux.re, uy.re, if dim == 3 { uz.re } else { 0.0 }];
        u_real_disp.push(disp_re);

        let orig = mesh.nodes[i];
        deformed_nodes.push([orig[0] + ux.re, orig[1] + uy.re, orig[2] + uz.re]);
    }

    // Zero out fixed Dirichlet displacements
    for &node_idx in &fixed_nodes {
        amplitudes[node_idx] = [0.0, 0.0, 0.0];
        phase_lags[node_idx] = [0.0, 0.0, 0.0];
        u_real_disp[node_idx] = [0.0, 0.0, 0.0];
        deformed_nodes[node_idx] = mesh.nodes[node_idx];
    }

    // 7. Compute von Mises stresses corresponding to peak amplitudes
    let von_mises = compute_nodal_von_mises(mesh, &u_real_disp, reference, strategy);
    let max_vm = von_mises.iter().cloned().fold(0.0_f64, f64::max);

    HarmonicSimulationResult {
        converged: gmres_res.converged,
        iterations: gmres_res.iterations,
        relative_residual: gmres_res.relative_residual,
        frequency_hz,
        amplitudes,
        phase_lags,
        deformed_nodes,
        von_mises_stresses: von_mises,
        max_amplitude: max_amp,
        max_von_mises: max_vm,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    }
}

/// Solves transient elastodynamic problem using modified Talbot contour quadrature in the complex Laplace domain
pub fn solve_transient_talbot(
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
) -> TransientSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    // 1. Assemble K_0 and M_FEM with exact Dirichlet DOF filtering
    let (k0_mat, m_mat, fixed_nodes) = assemble_dynamic_stiffness_and_mass(mesh, reference, strategy);

    // 2. Identify loaded nodes deduplicated
    let mut loaded_nodes_set = std::collections::HashSet::new();
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

    // 3. Generate Talbot contour nodes in the complex s-plane
    // For oscillatory sinusoidal loading, at least 32 nodes are required to accurately resolve the poles
    let exc_type = excitation_type.unwrap_or("Pulse");
    let exc_freq = excitation_frequency.unwrap_or(3.0);
    let min_pts = if exc_type == "HarmonicSine" { 32 } else { 16 };
    let n_pts = num_talbot_points.max(min_pts);
    let talbot_nodes = generate_talbot_nodes(n_pts, total_time);

    // 4. Parallel solve across all Talbot points using Rayon
    let t_pulse = pulse_duration.max(total_time * 0.05);

    let u_hat_list: Vec<Vec<Complex64>> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;
            let s_sq = s * s;

            // Load transform F(s) depending on excitation type:
            let pulse_factor = match exc_type {
                "HarmonicSine" => {
                    let omega = 2.0 * std::f64::consts::PI * exc_freq.max(0.01);
                    Complex64::new(omega, 0.0) / (s * s + Complex64::new(omega * omega, 0.0))
                }
                "Step" => {
                    Complex64::new(1.0, 0.0) / s
                }
                "HalfSine" => {
                    let omega_p = std::f64::consts::PI / t_pulse;
                    let num = Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp());
                    let denom = s * s + Complex64::new(omega_p * omega_p, 0.0);
                    num / denom
                }
                _ => { // "Pulse"
                    (Complex64::new(1.0, 0.0) - (-s * t_pulse).exp()) / s
                }
            };

            // Extract diagonal preconditioner for this s_k
            let mut diag_inv = vec![Complex64::new(1.0, 0.0); total_dofs];
            for col in 0..total_dofs {
                let mut k_diag = 0.0;
                let mut m_diag = 0.0;
                for (row, &val) in k0_mat.row_indices_of_col(col).zip(k0_mat.values_of_col(col)) {
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
                if total.norm() > 1e-8 {
                    diag_inv[col] = Complex64::new(1.0, 0.0) / total;
                }
            }

            let mut b_prec = vec![Complex64::new(0.0, 0.0); total_dofs];
            for i in 0..total_dofs {
                let f_s = Complex64::new(base_f_ext[i], 0.0) * pulse_factor;
                b_prec[i] = diag_inv[i] * f_s;
            }

            let apply_a = |v: &[Complex64]| {
                let kv = sp_mat_mul_complex(&k0_mat, v);
                let mv = sp_mat_mul_complex(&m_mat, v);
                let mut out = vec![Complex64::new(0.0, 0.0); total_dofs];
                for i in 0..total_dofs {
                    let total = kv[i] + s_sq * mv[i];
                    out[i] = diag_inv[i] * total;
                }
                out
            };

            let res = solve_complex_gmres(total_dofs, &b_prec, apply_a, 100, 1e-7, 100);
            res.solution
        })
        .collect();

    // 5. Invert Laplace transform at discrete time frames t_m in (0, total_time]
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

    TransientSimulationResult {
        time_points,
        frames,
        max_overall_displacement: max_overall_disp,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    }
}

/// Solves transient elastodynamic problem using standalone Pure BEM via modified Talbot contour quadrature
pub fn solve_transient_talbot_bem(
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
    elem_type: BemElementType,
) -> TransientSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();

    let exc_type = excitation_type.unwrap_or("Pulse");
    let exc_freq = excitation_frequency.unwrap_or(3.0);
    let min_pts = if exc_type == "HarmonicSine" { 32 } else { 16 };
    let n_pts = num_talbot_points.max(min_pts);
    let talbot_nodes = generate_talbot_nodes(n_pts, total_time);

    let t_pulse = pulse_duration.max(total_time * 0.05);

    // Parallel solve across all Talbot points using Rayon
    let u_hat_list: Vec<Vec<Complex64>> = talbot_nodes
        .par_iter()
        .map(|pt| {
            let s = pt.s;

            // Load transform F(s) depending on excitation type:
            let pulse_factor = match exc_type {
                "HarmonicSine" => {
                    let omega = 2.0 * std::f64::consts::PI * exc_freq.max(0.01);
                    Complex64::new(omega, 0.0) / (s * s + Complex64::new(omega * omega, 0.0))
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

            let load_factor = Complex64::new(load_magnitude, 0.0) * pulse_factor;
            crate::solver::dynamic_bem::solve_laplace_pure_bem_nodal_displacements(
                mesh,
                reference,
                s,
                load_factor,
                elem_type,
            )
        })
        .collect();

    // Invert Laplace transform at discrete time frames t_m in (0, total_time]
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
            let ux = u_dofs[i * dim + 0];
            let uy = u_dofs[i * dim + 1];
            let uz = if dim == 3 { u_dofs[i * dim + 2] } else { 0.0 };

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

    TransientSimulationResult {
        time_points,
        frames,
        max_overall_displacement: max_overall_disp,
        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
    }
}
