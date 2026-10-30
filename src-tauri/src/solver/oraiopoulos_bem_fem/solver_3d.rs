use super::mesh::*;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::anderson::AndersonAccelerator;
use crate::solver::fem_solver::{
    assemble_reference_stiffness_with_stress_state, compute_diagonal_inverse,
    compute_inhomogeneous_forces_with_stress_state, compute_nodal_von_mises,
    solve_pcg_sparse_with_preconditioner, sparse_mat_vec_mult,
};
use crate::solver::types::{ConvergenceLog, HybridAcceleration, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

/// Solves 3D elastostatics using Oraiopoulos Decoupled BEM Predictor (Collocated Quad4 Boundary Elements)
/// followed by FEM Volume Corrector (Iterative Residual Relaxation) on an independent 3D volume mesh (Hex8/Tet4).
pub fn solve_oraiopoulos_bem_fem_3d(
    length: f64,
    height: f64,
    depth: f64,
    bnd_nx: usize,
    bnd_ny: usize,
    bnd_nz: usize,
    fem_mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    max_iterations: usize,
    tolerance: f64,
    acceleration: HybridAcceleration,
) -> (SimulationResult, f64, f64) {
    let total_start = std::time::Instant::now();
    let dim = 3;
    let mu = reference.mu0();
    let nu = reference.nu0;

    let dx = length / bnd_nx.max(1) as f64;
    let dy = height / bnd_ny.max(1) as f64;
    let dz = depth / bnd_nz.max(1) as f64;

    // =========================================================================
    // PHASE 1: 3D BEM BOUNDARY DISCRETIZATION & SYSTEM ASSEMBLY
    // =========================================================================
    let pred_start = std::time::Instant::now();
    let bnd_elems = generate_decoupled_boundary_mesh_3d(length, height, depth, bnd_nx, bnd_ny, bnd_nz);
    let num_bnd_elems = bnd_elems.len();
    let total_bnd_dofs = num_bnd_elems * dim;

    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let g2_xi = [-1.0 / 3.0_f64.sqrt(), 1.0 / 3.0_f64.sqrt()];

    let factor_g_sing = 1.0 / (16.0 * std::f64::consts::PI * mu * (1.0 - nu));

    for i in 0..num_bnd_elems {
        let xi_pt = bnd_elems[i].center;

        for j in 0..num_bnd_elems {
            let elem_j = &bnd_elems[j];
            let normal_j = elem_j.normal;
            let area_j = elem_j.area;

            if i == j {
                // Diagonal singular integration: Flat rectangular patch
                let len_v1 = (elem_j.v1[0].powi(2) + elem_j.v1[1].powi(2) + elem_j.v1[2].powi(2)).sqrt();
                let len_v2 = (elem_j.v2[0].powi(2) + elem_j.v2[1].powi(2) + elem_j.v2[2].powi(2)).sqrt();
                let a = 0.5 * len_v1;
                let b = 0.5 * len_v2;
                let e1 = [elem_j.v1[0] / len_v1, elem_j.v1[1] / len_v1, elem_j.v1[2] / len_v1];
                let e2 = [elem_j.v2[0] / len_v2, elem_j.v2[1] / len_v2, elem_j.v2[2] / len_v2];

                let hyp = (a * a + b * b).sqrt();
                let i0 = 4.0 * (a * ((b + hyp) / a).ln() + b * ((a + hyp) / b).ln());
                let i1 = 4.0 * b * ((a + hyp) / b).ln();
                let i2 = 4.0 * a * ((b + hyp) / a).ln();

                let kappa = 3.0 - 4.0 * nu;
                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        let g_val = factor_g_sing * (kappa * delta * i0 + i1 * e1[d1] * e1[d2] + i2 * e2[d1] * e2[d2]);
                        g_mat[(i * dim + d1, j * dim + d2)] += g_val;
                    }
                }
            } else {
                // 2x2 Gauss Quadrature for regular element integral
                for &xi_val in &g2_xi {
                    for &eta_val in &g2_xi {
                        let n0 = 0.5 * (1.0 + xi_val);
                        let n1 = 0.5 * (1.0 + eta_val);
                        let x_g = [
                            elem_j.p0[0] + n0 * elem_j.v1[0] + n1 * elem_j.v2[0],
                            elem_j.p0[1] + n0 * elem_j.v1[1] + n1 * elem_j.v2[1],
                            elem_j.p0[2] + n0 * elem_j.v1[2] + n1 * elem_j.v2[2],
                        ];
                        let (u_star, t_star) = kelvin_3d_kernels(x_g, xi_pt, normal_j, mu, nu);
                        let d_gamma = 0.25 * area_j; // 4 points, total weight sum = 4

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_mat[(i * dim + d1, j * dim + d2)] += t_star[d1][d2] * d_gamma;
                                g_mat[(i * dim + d1, j * dim + d2)] += u_star[d1][d2] * d_gamma;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H
    for i in 0..num_bnd_elems {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..num_bnd_elems {
                    if i != k {
                        sum += h_mat[(row, k * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Apply BCs:
    // Left (x = 0): Dirichlet u = 0. Column unknown is traction t, scale by mu on LHS.
    // Right (x = L): Shear load -load_magnitude in Y direction. Area = height * depth.
    // Other faces: Traction-free t = 0.
    let area_right = height * depth;
    let traction_y = -load_magnitude / area_right;

    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);

    for j in 0..num_bnd_elems {
        let tag = &bnd_elems[j].tag;
        for d in 0..dim {
            let col = j * dim + d;
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

    let lu = a_mat.full_piv_lu();
    let solution = lu.solve(&rhs_mat);

    // Reconstruct boundary values
    let mut bnd_u = vec![[0.0; 3]; num_bnd_elems];
    let mut bnd_t = vec![[0.0; 3]; num_bnd_elems];

    for j in 0..num_bnd_elems {
        let tag = &bnd_elems[j].tag;
        if tag == "Left" {
            bnd_u[j] = [0.0, 0.0, 0.0];
            bnd_t[j] = [
                solution[(j * dim + 0, 0)] * mu,
                solution[(j * dim + 1, 0)] * mu,
                solution[(j * dim + 2, 0)] * mu,
            ];
        } else {
            bnd_u[j] = [
                solution[(j * dim + 0, 0)],
                solution[(j * dim + 1, 0)],
                solution[(j * dim + 2, 0)],
            ];
            let ty = if tag == "Right" { traction_y } else { 0.0 };
            bnd_t[j] = [0.0, ty, 0.0];
        }
    }

    // =========================================================================
    // PHASE 2: SOMIGLIANA PREDICTOR PROJECTION ONTO 3D FEM NODES
    // =========================================================================
    let num_fem_nodes = fem_mesh.nodes.len();
    let total_fem_dofs = num_fem_nodes * dim;
    let mut u_raw = vec![0.0; total_fem_dofs];

    for i in 0..num_fem_nodes {
        let p = fem_mesh.nodes[i];

        // Fixed Left Support -> exact 0
        if p[0] <= 1e-6 {
            u_raw[i * dim + 0] = 0.0;
            u_raw[i * dim + 1] = 0.0;
            u_raw[i * dim + 2] = 0.0;
            continue;
        }

        // Check if node lies on an exterior boundary face
        let is_bottom = p[1].abs() <= 1e-6;
        let is_top = (p[1] - height).abs() <= 1e-6;
        let is_right = (p[0] - length).abs() <= 1e-6;
        let is_back = p[2].abs() <= 1e-6;
        let is_front = (p[2] - depth).abs() <= 1e-6;

        let mut interpolated = false;
        if is_bottom || is_top || is_right || is_back || is_front {
            let mut min_dist_sq = f64::MAX;
            let mut best_elem = 0;
            for (j, elem) in bnd_elems.iter().enumerate() {
                let d_sq = (p[0] - elem.center[0]).powi(2)
                    + (p[1] - elem.center[1]).powi(2)
                    + (p[2] - elem.center[2]).powi(2);
                if d_sq < min_dist_sq {
                    min_dist_sq = d_sq;
                    best_elem = j;
                }
            }
            if min_dist_sq.sqrt() <= 0.75 * (dx.max(dy).max(dz)) {
                u_raw[i * dim + 0] = bnd_u[best_elem][0];
                u_raw[i * dim + 1] = bnd_u[best_elem][1];
                u_raw[i * dim + 2] = bnd_u[best_elem][2];
                interpolated = true;
            }
        }

        if !interpolated {
            // Strictly interior node -> Somigliana Integral
            let mut u_pt = [0.0; 3];
            for (j, elem) in bnd_elems.iter().enumerate() {
                let normal_j = elem.normal;
                let area_j = elem.area;
                let u_j = bnd_u[j];
                let t_j = bnd_t[j];

                for &xi_val in &g2_xi {
                    for &eta_val in &g2_xi {
                        let n0 = 0.5 * (1.0 + xi_val);
                        let n1 = 0.5 * (1.0 + eta_val);
                        let x_g = [
                            elem.p0[0] + n0 * elem.v1[0] + n1 * elem.v2[0],
                            elem.p0[1] + n0 * elem.v1[1] + n1 * elem.v2[1],
                            elem.p0[2] + n0 * elem.v1[2] + n1 * elem.v2[2],
                        ];
                        let (u_star, t_star) = kelvin_3d_kernels(x_g, p, normal_j, mu, nu);
                        let d_gamma = 0.25 * area_j;

                        for d1 in 0..dim {
                            let mut du_d1 = 0.0;
                            for d2 in 0..dim {
                                du_d1 += (u_star[d1][d2] * t_j[d2] - t_star[d1][d2] * u_j[d2]) * d_gamma;
                            }
                            u_pt[d1] += du_d1;
                        }
                    }
                }
            }

            u_raw[i * dim + 0] = u_pt[0];
            u_raw[i * dim + 1] = u_pt[1];
            u_raw[i * dim + 2] = u_pt[2];
        }
    }

    let pred_time_ms = pred_start.elapsed().as_secs_f64() * 1000.0;

    // =========================================================================
    // PHASE 3: VOLUME CORRECTOR LOOP ON 3D FEM MESH
    // =========================================================================
    let corr_start = std::time::Instant::now();
    let (k0_mat, fixed_nodes, loaded_nodes) =
        assemble_reference_stiffness_with_stress_state(fem_mesh, reference, StressState2D::PlaneStress);
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

    let u_0 = u_raw.clone();
    let mut delta_u = vec![0.0; total_fem_dofs];

    let f_inh_0 = compute_inhomogeneous_forces_with_stress_state(
        fem_mesh,
        &u_0,
        reference,
        strategy,
        StressState2D::PlaneStress,
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
        converged = true;
        convergence_history.push(ConvergenceLog {
            iteration: 0,
            residual_norm: init_r_norm,
            relative_error: init_rel_error,
        });
    } else {
        convergence_history.push(ConvergenceLog {
            iteration: 0,
            residual_norm: init_r_norm,
            relative_error: init_rel_error,
        });

        for iter in 1..=max_iterations.max(1) {
            final_iter = iter;

            let mut u_total = vec![0.0; total_fem_dofs];
            for i in 0..total_fem_dofs {
                u_total[i] = u_0[i] + delta_u[i];
            }

            let f_inh = compute_inhomogeneous_forces_with_stress_state(
                fem_mesh,
                &u_total,
                reference,
                strategy,
                StressState2D::PlaneStress,
            );

            let k0_delta_u = sparse_mat_vec_mult(&k0_mat, &delta_u);

            let mut residual = vec![0.0; total_fem_dofs];
            let mut r_norm_sq = 0.0;
            for i in 0..total_fem_dofs {
                let node_idx = i / dim;
                if fixed_nodes.contains(&node_idx) {
                    residual[i] = 0.0;
                } else {
                    let r_i = -k0_delta_u[i] - f_inh[i];
                    residual[i] = r_i;
                    r_norm_sq += r_i * r_i;
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

            let d_delta_u = solve_pcg_sparse_with_preconditioner(
                &k0_mat,
                &residual,
                &k0_diag_inv,
                None,
                1000,
                1e-8,
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

    for i in 0..total_fem_dofs {
        u_raw[i] = u_0[i] + delta_u[i];
    }

    let corr_time_ms = corr_start.elapsed().as_secs_f64() * 1000.0;
    let total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;

    let mut displacements = Vec::with_capacity(num_fem_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_fem_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_fem_nodes {
        let ux = u_raw[i * dim + 0];
        let uy = u_raw[i * dim + 1];
        let uz = u_raw[i * dim + 2];
        let mag = (ux * ux + uy * uy + uz * uz).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }
        displacements.push([ux, uy, uz]);
        let p = fem_mesh.nodes[i];
        deformed_nodes.push([p[0] + ux, p[1] + uy, p[2] + uz]);
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

/// Solves 3D elastostatics using standalone Pure Boundary Element Method (BEM)
/// on the rectangular beam [0, length] x [0, height] x [0, depth]
/// and evaluates displacements and stresses on the nodes of `eval_mesh`.
pub fn solve_pure_bem_3d(
    length: f64,
    height: f64,
    depth: f64,
    bnd_nx: usize,
    bnd_ny: usize,
    bnd_nz: usize,
    eval_mesh: &MeshData,
    reference: &ReferenceMaterial,
    load_magnitude: f64,
) -> SimulationResult {
    let strategy = crate::material::strategies::DirectionalGradientField::new(
        crate::material::types::Axis::X,
        0.0,
        length,
        1.0,
        crate::material::types::GradientForm::Linear,
    );
    let (mut res, _, _) = solve_oraiopoulos_bem_fem_3d(
        length,
        height,
        depth,
        bnd_nx,
        bnd_ny,
        bnd_nz,
        eval_mesh,
        reference,
        &strategy,
        load_magnitude,
        1,
        1e-4,
        HybridAcceleration::Relaxation(1.0),
    );
    res.solver_mode = SolverMode::PureBEM;
    res
}