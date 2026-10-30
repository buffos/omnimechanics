use super::mesh::*;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::anderson::AndersonAccelerator;
use crate::solver::bem_solver::{effective_bem_nu, integrate_poly_ln};
use crate::solver::fem_solver::{
    assemble_reference_stiffness_with_stress_state, compute_diagonal_inverse,
    compute_inhomogeneous_forces_with_stress_state, compute_nodal_von_mises,
    solve_pcg_sparse_with_preconditioner, sparse_mat_vec_mult,
};
use crate::solver::true_bem_fem::kelvin_2d_kernels;
use crate::solver::types::{ConvergenceLog, HybridAcceleration, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

/// Solves 2D elastostatics using Oraiopoulos Decoupled BEM Predictor (Quadratic Discontinuous Line3 elements)
/// followed by FEM Volume Corrector (Iterative Residual Relaxation) on an independent volume mesh.
pub fn solve_oraiopoulos_bem_fem(
    length: f64,
    height: f64,
    bnd_nx: usize,
    bnd_ny: usize,
    fem_mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    max_iterations: usize,
    tolerance: f64,
    acceleration: HybridAcceleration,
    stress_state: StressState2D,
) -> (SimulationResult, f64, f64) {
    let total_start = std::time::Instant::now();
    let dim = 2;
    let mu = reference.mu0();
    let nu = effective_bem_nu(reference.nu0, stress_state);

    // =========================================================================
    // PHASE 1: BEM BOUNDARY DISCRETIZATION & SYSTEM ASSEMBLY
    // =========================================================================
    let pred_start = std::time::Instant::now();
    let bnd_elems = generate_decoupled_boundary_mesh(length, height, bnd_nx, bnd_ny);
    let num_bnd_elems = bnd_elems.len();
    let num_elem_nodes = 3;
    let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
    let total_bnd_dofs = total_bnd_nodes * dim;

    // Discontinuous collocation points at xi in {-2/3, 0, 2/3}
    let local_xi = [-2.0 / 3.0, 0.0, 2.0 / 3.0];
    let mut bnd_nodes_coords = Vec::with_capacity(total_bnd_nodes);
    for elem in &bnd_elems {
        for &xi in &local_xi {
            let n0 = 0.5 * (1.0 - xi);
            let n1 = 0.5 * (1.0 + xi);
            bnd_nodes_coords.push([
                n0 * elem.p0[0] + n1 * elem.p1[0],
                n0 * elem.p0[1] + n1 * elem.p1[1],
            ]);
        }
    }

    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let factor_g = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * std::f64::consts::PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    // 8-point Gauss-Legendre quadrature for regular integrals
    let g8_xi: [f64; 8] = [
        -0.9602898564975363, -0.7966664774136267, -0.5255324099163290, -0.1834346424956498,
         0.1834346424956498,  0.5255324099163290,  0.7966664774136267,  0.9602898564975363,
    ];
    let g8_w: [f64; 8] = [
        0.1012285362903763, 0.2223810344533745, 0.3137066458778873, 0.3626837833783620,
        0.3626837833783620, 0.3137066458778873, 0.2223810344533745, 0.1012285362903763,
    ];

    let shape_fn = |alpha: usize, xi: f64| -> f64 {
        match alpha {
            0 => 9.0 / 8.0 * xi * (xi - 2.0 / 3.0),
            1 => 1.0 - 9.0 / 4.0 * xi * xi,
            2 => 9.0 / 8.0 * xi * (xi + 2.0 / 3.0),
            _ => 0.0,
        }
    };

    // Assembly of H and G
    for i in 0..total_bnd_nodes {
        let p_i = bnd_nodes_coords[i];
        let elem_i = i / num_elem_nodes;
        let local_i = i % num_elem_nodes;

        for j in 0..num_bnd_elems {
            let elem_j = &bnd_elems[j];
            let p0 = elem_j.p0;
            let p1 = elem_j.p1;
            let len_j = elem_j.length;
            let normal_j = elem_j.normal;
            let tau = [(p1[0] - p0[0]) / len_j, (p1[1] - p0[1]) / len_j];

            if elem_i == j {
                // Singular integration on same element
                let xi0 = local_xi[local_i];
                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let (c0, c1, c2) = match alpha {
                        0 => (0.0, -0.75, 1.125),
                        1 => (1.0, 0.0, -2.25),
                        2 => (0.0, 0.75, 1.125),
                        _ => (0.0, 0.0, 0.0),
                    };

                    let int_poly_ln = integrate_poly_ln(c0, c1, c2, xi0);
                    let shape_integral = match alpha {
                        0 => 0.75,
                        1 => 0.50,
                        2 => 0.75,
                        _ => 0.0,
                    };

                    let int_ln_r = 0.5 * len_j * (shape_integral * (2.0 / len_j).ln() - int_poly_ln);
                    let int_tau = 0.5 * len_j * shape_integral;

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_mat[(i * dim + d1, col_node * dim + d2)] += g_val;
                        }
                    }

                    if alpha != local_i {
                        let int_cauchy = match (local_i, alpha) {
                            (0, 1) => 3.0,
                            (0, 2) => 0.0,
                            (1, 0) => -1.5,
                            (1, 2) => 1.5,
                            (2, 0) => 0.0,
                            (2, 1) => -3.0,
                            _ => 0.0,
                        };
                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                let h_val = factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]) * int_cauchy;
                                h_mat[(i * dim + d1, col_node * dim + d2)] += h_val;
                            }
                        }
                    }
                }
            } else {
                // Regular 8-point Gauss quadrature
                for g in 0..8 {
                    let xi = g8_xi[g];
                    let w = g8_w[g];
                    let n0 = 0.5 * (1.0 - xi);
                    let n1 = 0.5 * (1.0 + xi);
                    let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);
                    let d_gamma = 0.5 * len_j * w;

                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let phi = shape_fn(alpha, xi);

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_mat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * phi * d_gamma;
                                g_mat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * phi * d_gamma;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H
    for i in 0..total_bnd_nodes {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..total_bnd_nodes {
                    if i != k {
                        sum += h_mat[(row, k * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Apply BCs:
    // Left: Dirichlet u = 0, move -G * mu to LHS for well-conditioned scaling
    // Right: shear traction ty = -load_magnitude / height, tx = 0
    // Top, Bottom: traction free
    let traction_y = -load_magnitude / height;
    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);

    for j in 0..num_bnd_elems {
        let tag = &bnd_elems[j].tag;
        for alpha in 0..num_elem_nodes {
            let node_idx = j * num_elem_nodes + alpha;
            for d in 0..dim {
                let col = node_idx * dim + d;
                if tag == "Left" {
                    for row in 0..total_bnd_dofs {
                        a_mat[(row, col)] = -g_mat[(row, col)] * mu;
                    }
                } else {
                    for row in 0..total_bnd_dofs {
                        a_mat[(row, col)] = h_mat[(row, col)];
                    }
                    let t_known = if tag == "Right" && d == 1 { traction_y } else { 0.0 };
                    if t_known.abs() > 1e-12 {
                        for row in 0..total_bnd_dofs {
                            rhs_mat[(row, 0)] += g_mat[(row, col)] * t_known;
                        }
                    }
                }
            }
        }
    }

    let lu = a_mat.full_piv_lu();
    let solution = lu.solve(&rhs_mat);

    // Reconstruct nodal boundary values
    let mut bnd_nodal_u = vec![[0.0, 0.0]; total_bnd_nodes];
    let mut bnd_nodal_t = vec![[0.0, 0.0]; total_bnd_nodes];

    for j in 0..num_bnd_elems {
        let tag = &bnd_elems[j].tag;
        for alpha in 0..num_elem_nodes {
            let node_idx = j * num_elem_nodes + alpha;
            if tag == "Left" {
                bnd_nodal_u[node_idx] = [0.0, 0.0];
                bnd_nodal_t[node_idx] = [
                    solution[(node_idx * dim + 0, 0)] * mu,
                    solution[(node_idx * dim + 1, 0)] * mu,
                ];
            } else {
                bnd_nodal_u[node_idx] = [solution[(node_idx * dim + 0, 0)], solution[(node_idx * dim + 1, 0)]];
                let ty = if tag == "Right" { traction_y } else { 0.0 };
                bnd_nodal_t[node_idx] = [0.0, ty];
            }
        }
    }

    // =========================================================================
    // PHASE 2: SOMIGLIANA PREDICTOR PROJECTION ONTO FEM NODES
    // =========================================================================
    let num_fem_nodes = fem_mesh.nodes.len();
    let total_fem_dofs = num_fem_nodes * dim;
    let mut u_raw = vec![0.0; total_fem_dofs];

    let g4_xi = [
        -0.8611363115940526, -0.3399810435848563,
         0.3399810435848563,  0.8611363115940526,
    ];
    let g4_w = [
        0.3478548451374538, 0.6521451548625461,
        0.6521451548625461, 0.3478548451374538,
    ];

    for i in 0..num_fem_nodes {
        let p = fem_mesh.nodes[i];

        // Fixed Left Support -> exact 0
        if p[0] <= 1e-6 {
            u_raw[i * dim + 0] = 0.0;
            u_raw[i * dim + 1] = 0.0;
            continue;
        }

        // Check if node is on boundary edges (Bottom, Right, Top)
        let is_bottom = p[1].abs() <= 1e-6;
        let is_top = (p[1] - height).abs() <= 1e-6;
        let is_right = (p[0] - length).abs() <= 1e-6;

        let mut interpolated = false;
        if is_bottom || is_top || is_right {
            for (j, elem) in bnd_elems.iter().enumerate() {
                let p0 = elem.p0;
                let p1 = elem.p1;
                let elem_dx = p1[0] - p0[0];
                let elem_dy = p1[1] - p0[1];
                let elem_len_sq = elem_dx * elem_dx + elem_dy * elem_dy;

                // Projection parameter s in [0, 1]
                let s = ((p[0] - p0[0]) * elem_dx + (p[1] - p0[1]) * elem_dy) / elem_len_sq;
                let perp_dist = ((p[0] - (p0[0] + s * elem_dx)).powi(2) + (p[1] - (p0[1] + s * elem_dy)).powi(2)).sqrt();

                if perp_dist <= 1e-5 && s >= -1e-5 && s <= 1.0 + 1e-5 {
                    let s_clamped = s.clamp(0.0, 1.0);
                    let xi = 2.0 * s_clamped - 1.0;
                    let mut ux = 0.0;
                    let mut uy = 0.0;
                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let phi = shape_fn(alpha, xi);
                        ux += phi * bnd_nodal_u[col_node][0];
                        uy += phi * bnd_nodal_u[col_node][1];
                    }
                    u_raw[i * dim + 0] = ux;
                    u_raw[i * dim + 1] = uy;
                    interpolated = true;
                    break;
                }
            }
        }

        if !interpolated {
            // Strictly interior node -> Somigliana Integral
            let mut ux = 0.0;
            let mut uy = 0.0;
            let xi_pt = [p[0], p[1]];

            for (j, elem) in bnd_elems.iter().enumerate() {
                let p0 = elem.p0;
                let p1 = elem.p1;
                let len_j = elem.length;
                let normal_j = elem.normal;

                for g in 0..4 {
                    let xi = g4_xi[g];
                    let w = g4_w[g];
                    let n0 = 0.5 * (1.0 - xi);
                    let n1 = 0.5 * (1.0 + xi);
                    let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, xi_pt, normal_j, mu, nu);
                    let d_gamma = 0.5 * len_j * w;

                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let phi = shape_fn(alpha, xi);
                        let u_nodal = bnd_nodal_u[col_node];
                        let t_nodal = bnd_nodal_t[col_node];

                        let du_x = (u_star[0][0] * t_nodal[0] + u_star[0][1] * t_nodal[1])
                            - (t_star[0][0] * u_nodal[0] + t_star[0][1] * u_nodal[1]);
                        let du_y = (u_star[1][0] * t_nodal[0] + u_star[1][1] * t_nodal[1])
                            - (t_star[1][0] * u_nodal[0] + t_star[1][1] * u_nodal[1]);

                        ux += du_x * phi * d_gamma;
                        uy += du_y * phi * d_gamma;
                    }
                }
            }

            u_raw[i * dim + 0] = ux;
            u_raw[i * dim + 1] = uy;
        }
    }

    let pred_time_ms = pred_start.elapsed().as_secs_f64() * 1000.0;

    // =========================================================================
    // PHASE 3: VOLUME CORRECTOR LOOP ON FEM MESH (PERTURBATION OPERATOR SPLITTING)
    // =========================================================================
    let corr_start = std::time::Instant::now();
    let (k0_mat, fixed_nodes, loaded_nodes) =
        assemble_reference_stiffness_with_stress_state(fem_mesh, reference, stress_state);
    let k0_diag_inv = compute_diagonal_inverse(&k0_mat);

    let mut f_ext = vec![0.0; total_fem_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            let dof_y = node_idx * dim + 1;
            f_ext[dof_y] += force_per_node;
        }
    }
    for &node_idx in &fixed_nodes {
        for d in 0..dim {
            f_ext[node_idx * dim + d] = 0.0;
        }
    }

    let f_ext_norm = f_ext.iter().map(|&x| x * x).sum::<f64>().sqrt().max(1.0);

    let mut convergence_history = Vec::new();
    let mut converged = false;
    let mut final_iter = 0;

    let mut accelerator = match acceleration {
        HybridAcceleration::Anderson { depth, beta } => {
            Some(AndersonAccelerator::new(depth, beta))
        }
        _ => None,
    };

    // Perturbation operator splitting:
    // u_0 is the pure BEM Somigliana solution projected onto the volume mesh.
    // u = u_0 + delta_u, where delta_u represents the correction due to material inhomogeneity.
    // The governing perturbation equation is: K_0 * delta_u + F_inh(u_0 + delta_u) = 0.
    let u_0 = u_raw.clone();
    let mut delta_u = vec![0.0; total_fem_dofs];

    // Evaluate initial inhomogeneous resisting force F_inh(u_0) = \int_\Omega B^T (C(x) - C0) B u_0 d\Omega
    let f_inh_0 = compute_inhomogeneous_forces_with_stress_state(
        fem_mesh,
        &u_0,
        reference,
        strategy,
        stress_state,
    );
    let mut init_r_sq = 0.0;
    for i in 0..total_fem_dofs {
        let node_idx = i / dim;
        if !fixed_nodes.contains(&node_idx) {
            init_r_sq += f_inh_0[i] * f_inh_0[i];
        }
    }
    let init_r_norm = init_r_sq.sqrt();
    let init_rel_error = init_r_norm / f_ext_norm;

    if init_rel_error < tolerance {
        // Homogeneous material (C(x) == C_0) or negligible perturbation:
        // F_inh == 0, so delta_u == 0. The pure BEM predictor is exact!
        converged = true;
        final_iter = 0;
        convergence_history.push(ConvergenceLog {
            iteration: 0,
            residual_norm: init_r_norm,
            relative_error: init_rel_error,
        });
    } else {
        for iter in 1..=max_iterations.max(1) {
            final_iter = iter;

            // Total displacement u = u_0 + delta_u
            let mut u_curr = vec![0.0; total_fem_dofs];
            for i in 0..total_fem_dofs {
                u_curr[i] = u_0[i] + delta_u[i];
            }

            // Inhomogeneous resisting force F_inh(u) = \int_\Omega B^T (C(x) - C0) B u d\Omega
            let f_inh = compute_inhomogeneous_forces_with_stress_state(
                fem_mesh,
                &u_curr,
                reference,
                strategy,
                stress_state,
            );

            // Reference restoring force of perturbation: K_0 * delta_u
            let k0_delta_u = sparse_mat_vec_mult(&k0_mat, &delta_u);

            // Perturbation residual: R = - (K_0 * delta_u + F_inh(u))
            let mut residual = vec![0.0; total_fem_dofs];
            let mut r_norm_sq = 0.0;

            for i in 0..total_fem_dofs {
                let node_idx = i / dim;
                if fixed_nodes.contains(&node_idx) {
                    residual[i] = 0.0;
                } else {
                    let r_val = -k0_delta_u[i] - f_inh[i];
                    residual[i] = r_val;
                    r_norm_sq += r_val * r_val;
                }
            }

            let r_norm = r_norm_sq.sqrt();
            let rel_error = r_norm / f_ext_norm;

            convergence_history.push(ConvergenceLog {
                iteration: iter,
                residual_norm: r_norm,
                relative_error: rel_error,
            });

            if rel_error < tolerance {
                converged = true;
                break;
            }

            let inner_tol = (0.05 * rel_error).clamp(1e-8, 1e-4);
            let d_delta_u = solve_pcg_sparse_with_preconditioner(
                &k0_mat,
                &residual,
                &k0_diag_inv,
                None,
                1000,
                inner_tol,
            );

            match &mut accelerator {
                Some(acc) => {
                    delta_u = acc.compute_update(&delta_u, &d_delta_u);
                    for &node_idx in &fixed_nodes {
                        for d in 0..dim {
                            delta_u[node_idx * dim + d] = 0.0;
                        }
                    }
                }
                None => {
                    let w = match acceleration {
                        HybridAcceleration::Relaxation(omega) => omega,
                        _ => 1.0,
                    };
                    for i in 0..total_fem_dofs {
                        let node_idx = i / dim;
                        if !fixed_nodes.contains(&node_idx) {
                            delta_u[i] += w * d_delta_u[i];
                        }
                    }
                }
            }
        }
    }

    // Reconstruct final total displacements: u_raw = u_0 + delta_u
    for i in 0..total_fem_dofs {
        u_raw[i] = u_0[i] + delta_u[i];
    }

    let corr_time_ms = corr_start.elapsed().as_secs_f64() * 1000.0;
    let total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    // Postprocessing: Extract displacements and deformed coordinates
    let mut displacements = Vec::with_capacity(num_fem_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_fem_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_fem_nodes {
        let ux = u_raw[i * dim + 0];
        let uy = u_raw[i * dim + 1];
        let mag = (ux * ux + uy * uy).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }
        displacements.push([ux, uy, 0.0]);
        let p = fem_mesh.nodes[i];
        deformed_nodes.push([p[0] + ux, p[1] + uy, p[2]]);
    }

    let von_mises_stresses = compute_nodal_von_mises(fem_mesh, &displacements, reference, strategy);
    let max_vm = von_mises_stresses.iter().cloned().fold(0.0, f64::max);

    let result = SimulationResult {
        solver_mode: SolverMode::BEMFEM,
        converged,
        iterations: final_iter,
        displacements,
        deformed_nodes,
        von_mises_stresses,
        max_displacement: max_disp,
        max_von_mises: max_vm,
        convergence_history,
        execution_time_ms: total_time_ms,
    };

    (result, pred_time_ms, corr_time_ms)
}