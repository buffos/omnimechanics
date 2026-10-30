use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::{assemble_dynamic_stiffness_and_mass, compute_nodal_von_mises};
use crate::solver::types::{SimulationFrame, TransientSimulationResult};
use faer::sparse::SparseColMat;
use std::collections::HashSet;

/// Multiplies a real SparseColMat by a vector: out = mat * v
pub fn sp_mat_mul(mat: &SparseColMat<usize, f64>, v: &[f64]) -> Vec<f64> {
    let n = mat.nrows();
    let mut out = vec![0.0; n];
    for col in 0..mat.ncols() {
        let v_col = v[col];
        if v_col.abs() < 1e-30 {
            continue;
        }
        for (row, &val) in mat.row_indices_of_col(col).zip(mat.values_of_col(col)) {
            out[row] += v_col * val;
        }
    }
    out
}

/// Solves K_eff * x = b using warm-started Preconditioned Conjugate Gradient (PCG)
pub fn solve_pcg(
    k_mat: &SparseColMat<usize, f64>,
    b: &[f64],
    x_init: &[f64],
    max_iter: usize,
    tol: f64,
    fixed_dofs: &HashSet<usize>,
) -> Vec<f64> {
    let n = b.len();
    let mut x = x_init.to_vec();
    for &dof in fixed_dofs {
        x[dof] = 0.0;
    }

    // Precompute diagonal inverse preconditioner
    let mut diag_inv = vec![1.0; n];
    for col in 0..n {
        for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
            if row == col && val.abs() > 1e-12 {
                diag_inv[row] = 1.0 / val;
            }
        }
    }

    // Initial residual: r = b - K * x
    let ax = sp_mat_mul(k_mat, &x);
    let mut r = vec![0.0; n];
    for i in 0..n {
        if fixed_dofs.contains(&i) {
            r[i] = 0.0;
        } else {
            r[i] = b[i] - ax[i];
        }
    }

    let norm_b = b.iter().map(|&v| v * v).sum::<f64>().sqrt().max(1e-12);
    let norm_r0 = r.iter().map(|&v| v * v).sum::<f64>().sqrt();
    if norm_r0 / norm_b < tol {
        return x;
    }

    // z = M^{-1} r
    let mut z = vec![0.0; n];
    for i in 0..n {
        z[i] = r[i] * diag_inv[i];
    }
    let mut p = z.clone();
    let mut rz_old: f64 = r.iter().zip(&z).map(|(&ri, &zi)| ri * zi).sum();

    for _ in 0..max_iter {
        let ap = sp_mat_mul(k_mat, &p);
        let mut pap: f64 = 0.0;
        for i in 0..n {
            if !fixed_dofs.contains(&i) {
                pap += p[i] * ap[i];
            }
        }

        if pap.abs() < 1e-30 {
            break;
        }

        let alpha = rz_old / pap;

        for i in 0..n {
            if !fixed_dofs.contains(&i) {
                x[i] += alpha * p[i];
                r[i] -= alpha * ap[i];
            }
        }

        let norm_r = r.iter().map(|&v| v * v).sum::<f64>().sqrt();
        if norm_r / norm_b < tol {
            break;
        }

        for i in 0..n {
            z[i] = r[i] * diag_inv[i];
        }
        let rz_new: f64 = r.iter().zip(&z).map(|(&ri, &zi)| ri * zi).sum();
        let beta_val = rz_new / rz_old;
        rz_old = rz_new;

        for i in 0..n {
            if fixed_dofs.contains(&i) {
                p[i] = 0.0;
            } else {
                p[i] = z[i] + beta_val * p[i];
            }
        }
    }

    x
}

/// Solves transient elastodynamic problem using classical Direct Time-Marching
/// (Newmark-β or Hilber-Hughes-Taylor α method)
pub fn solve_transient_newmark(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    total_time: f64,
    time_steps: usize,
    load_magnitude: f64,
    pulse_duration: f64,
    excitation_type: Option<&str>,
    excitation_frequency: Option<f64>,
    hht_alpha: Option<f64>,
    rayleigh_alpha: Option<f64>,
    rayleigh_beta: Option<f64>,
) -> TransientSimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;

    // 1. Assemble dynamic stiffness K and mass matrix M with exact Dirichlet DOF filtering
    let (k_mat, m_mat, fixed_nodes) = assemble_dynamic_stiffness_and_mass(mesh, reference, strategy);

    let mut fixed_dofs = HashSet::new();
    for &node in &fixed_nodes {
        for d in 0..dim {
            fixed_dofs.insert(node * dim + d);
        }
    }

    // 2. Identify loaded boundary nodes (e.g. tagged "Right") deduplicated
    let mut loaded_nodes_set = HashSet::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                loaded_nodes_set.insert(n);
            }
        }
    }
    let loaded_nodes: Vec<usize> = loaded_nodes_set.into_iter().collect();

    let mut base_f_spatial = vec![0.0; total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            base_f_spatial[node_idx * dim + 1] = force_per_node;
        }
    }

    // 3. Time stepping parameters
    let n_steps = time_steps.max(2);
    let dt = total_time / n_steps as f64;

    // Integration scheme: Newmark-β or HHT-α
    // α ∈ [-1/3, 0]. α = 0 corresponds to classical Newmark average acceleration
    let alpha_hht = hht_alpha.unwrap_or(0.0).clamp(-1.0 / 3.0, 0.0);
    let gamma = (1.0 - 2.0 * alpha_hht) / 2.0;
    let beta = (1.0 - alpha_hht) * (1.0 - alpha_hht) / 4.0;

    // Rayleigh damping parameters: C = α_R * M + β_R * K
    let alpha_r = rayleigh_alpha.unwrap_or(0.05);
    let beta_r = rayleigh_beta.unwrap_or(0.0002);

    // Newmark integration coefficients (Bathe 1996, Hughes 2000)
    let a0 = 1.0 / (beta * dt * dt);
    let a1 = gamma / (beta * dt);
    let a2 = 1.0 / (beta * dt);
    let a3 = 1.0 / (2.0 * beta) - 1.0;
    let a4 = gamma / beta - 1.0;
    let a5 = (dt / 2.0) * (gamma / beta - 2.0);
    let a6 = dt * (1.0 - gamma);
    let a7 = gamma * dt;

    // 4. Assemble Effective Dynamic Stiffness Matrix:
    // K_eff = (1 + a1 * β_R) * K + (a0 + a1 * α_R) * M
    let c_k = 1.0 + a1 * beta_r;
    let c_m = a0 + a1 * alpha_r;

    let mut k_eff_triplets = Vec::new();
    for col in 0..total_dofs {
        for (row, &val) in k_mat.row_indices_of_col(col).zip(k_mat.values_of_col(col)) {
            if fixed_dofs.contains(&row) || fixed_dofs.contains(&col) {
                if row == col {
                    k_eff_triplets.push((row, col, 1.0));
                }
            } else {
                k_eff_triplets.push((row, col, c_k * val));
            }
        }
    }

    for col in 0..total_dofs {
        for (row, &val) in m_mat.row_indices_of_col(col).zip(m_mat.values_of_col(col)) {
            if !fixed_dofs.contains(&row) && !fixed_dofs.contains(&col) {
                k_eff_triplets.push((row, col, c_m * val));
            }
        }
    }

    let k_eff = SparseColMat::try_new_from_triplets(total_dofs, total_dofs, &k_eff_triplets)
        .expect("Failed to build effective dynamic stiffness matrix for Newmark marching");

    // 5. Initial kinematic conditions at t = 0
    let mut u = vec![0.0; total_dofs];
    let mut u_dot = vec![0.0; total_dofs];
    let mut u_ddot = vec![0.0; total_dofs];

    let exc_type = excitation_type.unwrap_or("Pulse");
    let exc_freq = excitation_frequency.unwrap_or(3.0);
    let t_pulse = pulse_duration.max(total_time * 0.05);

    // Temporal excitation function p(t)
    let load_factor = |t: f64| -> f64 {
        match exc_type {
            "HarmonicSine" => {
                let omega = 2.0 * std::f64::consts::PI * exc_freq.max(0.01);
                (omega * t).sin()
            }
            "Step" => 1.0,
            "HalfSine" => {
                if t <= t_pulse {
                    let omega_p = std::f64::consts::PI / t_pulse;
                    (omega_p * t).sin()
                } else {
                    0.0
                }
            }
            _ => { // "Pulse"
                if t <= t_pulse {
                    1.0
                } else {
                    0.0
                }
            }
        }
    };

    // Calculate initial acceleration: M * u_ddot_0 = F_0 - C * u_dot_0 - K * u_0
    let f0_factor = load_factor(0.0);
    if f0_factor.abs() > 1e-12 {
        let mut f0 = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            f0[i] = base_f_spatial[i] * f0_factor;
        }
        // Approximate initial acceleration via diagonal lumped mass
        for col in 0..total_dofs {
            if fixed_dofs.contains(&col) {
                u_ddot[col] = 0.0;
                continue;
            }
            let mut m_diag = 0.0;
            for (row, &val) in m_mat.row_indices_of_col(col).zip(m_mat.values_of_col(col)) {
                if row == col {
                    m_diag = val;
                    break;
                }
            }
            if m_diag > 1e-12 {
                u_ddot[col] = f0[col] / m_diag;
            }
        }
    }

    // 6. Time Marching Loop
    let mut time_points = Vec::with_capacity(n_steps);
    let mut frames = Vec::with_capacity(n_steps);
    let mut max_overall_disp = 0.0;

    for step in 1..=n_steps {
        let t_next = step as f64 * dt;
        time_points.push(t_next);

        let p_factor = load_factor(t_next);

        // Effective RHS: F_eff = F_{n+1} + M * v_M + K * v_K
        // v_M = (a0 + a1 * α_R) u_n + (a2 + a4 * α_R) u_dot_n + (a3 + a5 * α_R) u_ddot_n
        // v_K = (a1 * β_R) u_n + (a4 * β_R) u_dot_n + (a5 * β_R) u_ddot_n
        let vm_u = a0 + a1 * alpha_r;
        let vm_v = a2 + a4 * alpha_r;
        let vm_a = a3 + a5 * alpha_r;

        let vk_u = a1 * beta_r;
        let vk_v = a4 * beta_r;
        let vk_a = a5 * beta_r;

        let mut v_m = vec![0.0; total_dofs];
        let mut v_k = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                v_m[i] = vm_u * u[i] + vm_v * u_dot[i] + vm_a * u_ddot[i];
                v_k[i] = vk_u * u[i] + vk_v * u_dot[i] + vk_a * u_ddot[i];
            }
        }

        let mv_m = sp_mat_mul(&m_mat, &v_m);
        let kv_k = sp_mat_mul(&k_mat, &v_k);

        let mut f_eff = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                f_eff[i] = base_f_spatial[i] * p_factor + mv_m[i] + kv_k[i];
            }
        }

        // Kinematic Taylor predictor as high-quality initial guess for PCG:
        // u_pred = u_n + dt * u_dot_n + dt^2 * (0.5 - beta) * u_ddot_n
        let pred_factor = dt * dt * (0.5 - beta);
        let mut u_pred = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                u_pred[i] = u[i] + dt * u_dot[i] + pred_factor * u_ddot[i];
            }
        }

        // Solve for u_{n+1}
        let u_next = solve_pcg(&k_eff, &f_eff, &u_pred, 500, 1e-8, &fixed_dofs);

        // Update kinematic vectors:
        // u_ddot_{n+1} = a0 * (u_{n+1} - u_n) - a2 * u_dot_n - a3 * u_ddot_n
        // u_dot_{n+1} = u_dot_n + a6 * u_ddot_n + a7 * u_ddot_{n+1}
        let mut u_ddot_next = vec![0.0; total_dofs];
        let mut u_dot_next = vec![0.0; total_dofs];

        for i in 0..total_dofs {
            if !fixed_dofs.contains(&i) {
                let acc = a0 * (u_next[i] - u[i]) - a2 * u_dot[i] - a3 * u_ddot[i];
                let vel = u_dot[i] + a6 * u_ddot[i] + a7 * acc;
                u_ddot_next[i] = acc;
                u_dot_next[i] = vel;
            }
        }

        u = u_next;
        u_dot = u_dot_next;
        u_ddot = u_ddot_next;

        // Extract frame results
        let mut displacements = Vec::with_capacity(num_nodes);
        let mut deformed_nodes = Vec::with_capacity(num_nodes);
        let mut frame_max_d = 0.0;

        for i in 0..num_nodes {
            let is_fixed = fixed_nodes.contains(&i);
            let ux = if is_fixed { 0.0 } else { u[i * dim + 0] };
            let uy = if is_fixed { 0.0 } else { u[i * dim + 1] };
            let uz = if is_fixed || dim < 3 { 0.0 } else { u[i * dim + 2] };

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
            time: t_next,
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
