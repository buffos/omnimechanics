use super::fem_solver::{compute_internal_forces, compute_nodal_von_mises};
use super::types::{ConvergenceLog, SimulationResult, SolverMode};
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use faer::prelude::SpSolver;
use faer::Mat;
use std::collections::HashSet;

/// Evaluates 2D Kelvin fundamental solution tensors U*(x, xi) and T*(x, xi).
/// x is field point on boundary, xi is source / interior evaluation point.
/// Returns (U*, T*) as 2x2 matrices: [[xx, xy], [yx, yy]].
pub fn kelvin_2d_kernels(
    x: [f64; 2],
    xi: [f64; 2],
    normal_x: [f64; 2],
    mu: f64,
    nu: f64,
) -> ([[f64; 2]; 2], [[f64; 2]; 2]) {
    let dx = x[0] - xi[0];
    let dy = x[1] - xi[1];
    let r = (dx * dx + dy * dy).sqrt().max(1e-10);

    let dr_dx = dx / r;
    let dr_dy = dy / r;
    let dr_dn = dr_dx * normal_x[0] + dr_dy * normal_x[1];

    // Constant factors for 2D plane strain Kelvin fundamental solution
    let c1 = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let ln_inv_r = (1.0 / r).ln();
    let kappa = 3.0 - 4.0 * nu;

    let u_xx = c1 * (kappa * ln_inv_r + dr_dx * dr_dx);
    let u_yy = c1 * (kappa * ln_inv_r + dr_dy * dr_dy);
    let u_xy = c1 * (dr_dx * dr_dy);
    let u_yx = u_xy;

    let c2 = -1.0 / (4.0 * std::f64::consts::PI * (1.0 - nu) * r);
    let one_minus_2nu = 1.0 - 2.0 * nu;

    let t_xx = c2 * (dr_dn * (one_minus_2nu + 2.0 * dr_dx * dr_dx));
    let t_yy = c2 * (dr_dn * (one_minus_2nu + 2.0 * dr_dy * dr_dy));
    let t_xy = c2 * (dr_dn * (2.0 * dr_dx * dr_dy) + one_minus_2nu * (dr_dy * normal_x[0] - dr_dx * normal_x[1]));
    let t_yx = c2 * (dr_dn * (2.0 * dr_dy * dr_dx) + one_minus_2nu * (dr_dx * normal_x[1] - dr_dy * normal_x[0]));

    ([[u_xx, u_xy], [u_yx, u_yy]], [[t_xx, t_xy], [t_yx, t_yy]])
}

/// Evaluates interior displacement vector u(p) at point p via Somigliana boundary integral:
/// u_i(p) = \int_\Gamma [ U_{ik}^*(x, p) t_k(x) - T_{ik}^*(x, p) u_k(x) ] d\Gamma(x)
/// using 2-point Gauss quadrature on each boundary element.
pub fn evaluate_somigliana_displacement_at_point(
    p: [f64; 2],
    mesh: &MeshData,
    bnd_u: &[[f64; 2]],
    bnd_t: &[[f64; 2]],
    mu: f64,
    nu: f64,
) -> [f64; 2] {
    let mut ux = 0.0;
    let mut uy = 0.0;

    let gauss_xi = [-1.0 / (3.0_f64).sqrt(), 1.0 / (3.0_f64).sqrt()];
    let gauss_w = [1.0, 1.0];

    for (j, bnd) in mesh.boundary_elements.iter().enumerate() {
        let p0 = mesh.nodes[bnd.nodes[0]];
        let p1 = mesh.nodes[bnd.nodes[1]];
        let dx = p1[0] - p0[0];
        let dy = p1[1] - p0[1];
        let len_j = (dx * dx + dy * dy).sqrt().max(1e-6);
        let normal = [bnd.normal[0], bnd.normal[1]];

        let u_elem = bnd_u[j];
        let t_elem = bnd_t[j];

        for g in 0..2 {
            let xi = gauss_xi[g];
            let w = gauss_w[g];

            // Midpoint / linear mapping to boundary element coordinates
            let n0 = 0.5 * (1.0 - xi);
            let n1 = 0.5 * (1.0 + xi);
            let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];

            let (u_star, t_star) = kelvin_2d_kernels(x_g, p, normal, mu, nu);

            let d_gamma = 0.5 * len_j * w;

            // Integrand: U* * t - T* * u
            let du_x = (u_star[0][0] * t_elem[0] + u_star[0][1] * t_elem[1])
                - (t_star[0][0] * u_elem[0] + t_star[0][1] * u_elem[1]);
            let du_y = (u_star[1][0] * t_elem[0] + u_star[1][1] * t_elem[1])
                - (t_star[1][0] * u_elem[0] + t_star[1][1] * u_elem[1]);

            ux += du_x * d_gamma;
            uy += du_y * d_gamma;
        }
    }

    [ux, uy]
}

/// Solves 2D elastostatics via True BEM Predictor -- FEM Corrector (Method 2).
///
/// 1. BEM Predictor: Solves boundary H u = G t system on boundary elements Gamma (dim: N_bnd x N_bnd).
/// 2. Somigliana Projection: Reconstructs interior displacement field u^(0)(x) directly from boundary solution
///    without any volume matrix inversion.
/// 3. FEM Volume Corrector: Integrates internal forces at element Gauss points with true inhomogeneous material law.
/// 4. Variational Residual Feedback: Corrects displacement field iteratively.
pub fn solve_true_bem_fem_static(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    max_iterations: usize,
    tolerance: f64,
    omega: f64,
) -> SimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let num_bnd_elems = mesh.boundary_elements.len();
    let total_bnd_dofs = num_bnd_elems * dim;
    let total_dofs = num_nodes * dim;

    let mu = reference.mu0();
    let nu = reference.nu0;

    // --- PHASE 1: ASSEMBLE TRUE BEM MATRICES H AND G ON BOUNDARY ELEMENTS ---
    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let elem_centers: Vec<[f64; 2]> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            [0.5 * (p0[0] + p1[0]), 0.5 * (p0[1] + p1[1])]
        })
        .collect();

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6)
        })
        .collect();

    for i in 0..num_bnd_elems {
        let p_i = elem_centers[i];

        for j in 0..num_bnd_elems {
            let p_j = elem_centers[j];
            let len_j = elem_lengths[j];
            let normal_j = [mesh.boundary_elements[j].normal[0], mesh.boundary_elements[j].normal[1]];

            if i == j {
                // Exact analytical integration of singular G kernel on straight line element
                let tau = [-normal_j[1], normal_j[0]];
                let int_ln = len_j * (1.0 - (0.5 * len_j).ln());
                let factor = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        g_mat[(i * dim + d1, j * dim + d2)] = factor * ((3.0 - 4.0 * nu) * int_ln * delta + len_j * tau[d1] * tau[d2]);
                    }
                }
            } else {
                let (u_star, t_star) = kelvin_2d_kernels(p_j, p_i, normal_j, mu, nu);

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        h_mat[(i * dim + d1, j * dim + d2)] = t_star[d1][d2] * len_j;
                        g_mat[(i * dim + d1, j * dim + d2)] = u_star[d1][d2] * len_j;
                    }
                }
            }
        }
    }

    // Compute diagonal blocks of H using the exact Rigid Body Displacement condition:
    // For rigid body translation in direction d: \sum_j H_{row, j*dim + d} = 0
    // => H_{i*dim + d, i*dim + d} = - \sum_{j != i} H_{i*dim + d, j*dim + d}
    for i in 0..num_bnd_elems {
        for d in 0..dim {
            let row = i * dim + d;
            let mut sum_d = 0.0;
            for j in 0..num_bnd_elems {
                if i != j {
                    sum_d += h_mat[(row, j * dim + d)];
                }
            }
            h_mat[(row, row)] = -sum_d;
            if dim == 2 {
                h_mat[(row, i * dim + (1 - d))] = 0.0; // Self-element cross-term is zero by symmetry
            }
        }
    }

    // --- PHASE 2: BOUNDARY CONDITIONS & BEM PREDICTOR SYSTEM A * x = RHS ---
    // Identify loaded boundary (Right) and fixed support (Left)
    let right_len: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_magnitude / right_len;

    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![0.0; total_bnd_dofs];

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        for d in 0..dim {
            let col = j * dim + d;
            if tag == "Left" {
                // Fixed Dirichlet boundary (u = 0 known, t unknown) -> move -G to LHS
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)];
                }
            } else {
                // Neumann / Free boundary (t known, u unknown) -> keep H on LHS
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let t_known = if tag == "Right" && d == 1 { traction_y } else { 0.0 };
                if t_known.abs() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * t_known;
                    }
                }
            }
        }
    }

    // Solve dense BEM boundary system via Full LU
    let lu = a_mat.full_piv_lu();
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);
    for i in 0..total_bnd_dofs {
        rhs_mat[(i, 0)] = rhs[i];
    }
    let solution = lu.solve(&rhs_mat);

    // Extract boundary element displacements and tractions
    let mut bnd_u = vec![[0.0, 0.0]; num_bnd_elems];
    let mut bnd_t = vec![[0.0, 0.0]; num_bnd_elems];

    let mut right_tip_uy = 0.0;
    let mut right_count = 0;

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        if tag == "Left" {
            bnd_u[j] = [0.0, 0.0];
            bnd_t[j] = [solution[(j * dim + 0, 0)], solution[(j * dim + 1, 0)]];
        } else {
            bnd_u[j] = [solution[(j * dim + 0, 0)], solution[(j * dim + 1, 0)]];
            if tag == "Right" {
                right_tip_uy += bnd_u[j][1];
                right_count += 1;
            }
            let ty = if tag == "Right" { traction_y } else { 0.0 };
            bnd_t[j] = [0.0, ty];
        }
    }
    if right_count > 0 {
        println!("       ==> BEM Direct Boundary Right Tip uy: {:.6} mm", (right_tip_uy / right_count as f64) * 1000.0);
    }

    // --- PHASE 3: SOMIGLIANA RECONSTRUCTION ACROSS ALL NODES ---
    // Collect boundary nodes for direct boundary assignment
    let mut bnd_node_displacements: std::collections::HashMap<usize, (f64, f64, usize)> = std::collections::HashMap::new();
    let mut fixed_nodes = HashSet::new();

    for (j, bnd) in mesh.boundary_elements.iter().enumerate() {
        let u_elem = bnd_u[j];
        for &node_idx in &bnd.nodes {
            if bnd.tag == "Left" {
                fixed_nodes.insert(node_idx);
            }
            let entry = bnd_node_displacements.entry(node_idx).or_insert((0.0, 0.0, 0));
            entry.0 += u_elem[0];
            entry.1 += u_elem[1];
            entry.2 += 1;
        }
    }

    let mut u_raw = vec![0.0; total_dofs];

    for i in 0..num_nodes {
        if fixed_nodes.contains(&i) {
            u_raw[i * dim + 0] = 0.0;
            u_raw[i * dim + 1] = 0.0;
            continue;
        }

        if let Some(&(sum_ux, sum_uy, count)) = bnd_node_displacements.get(&i) {
            // Node is on boundary -> direct boundary solution
            u_raw[i * dim + 0] = sum_ux / count as f64;
            u_raw[i * dim + 1] = sum_uy / count as f64;
        } else {
            // Node is interior -> Somigliana boundary integral representation
            let p = [mesh.nodes[i][0], mesh.nodes[i][1]];
            let u_som = evaluate_somigliana_displacement_at_point(
                p,
                mesh,
                &bnd_u,
                &bnd_t,
                mu,
                nu,
            );
            u_raw[i * dim + 0] = u_som[0];
            u_raw[i * dim + 1] = u_som[1];
        }
    }

    // Assemble external force vector F_ext for residual normalization
    let u0_max = u_raw.iter().cloned().fold(0.0, f64::max);
    let u0_min = u_raw.iter().cloned().fold(0.0, f64::min);
    println!("    -> BEM Initial Predictor u^(0): min = {:.6} mm, max = {:.6} mm", u0_min * 1000.0, u0_max * 1000.0);

    let mut f_ext = vec![0.0; total_dofs];
    let loaded_nodes: Vec<usize> = mesh
        .boundary_elements
        .iter()
        .filter(|b| b.tag == "Right")
        .flat_map(|b| b.nodes.iter().copied())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if !loaded_nodes.is_empty() {
        let f_per_node = -load_magnitude / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            f_ext[node_idx * dim + 1] += f_per_node;
        }
    }

    let f_ext_norm = f_ext.iter().map(|&x| x * x).sum::<f64>().sqrt().max(1.0);

    // --- PHASE 4: ITERATIVE FEM GAUSS-POINT CORRECTOR LOOP ---
    let mut convergence_history = Vec::new();
    let mut converged = false;
    let mut final_iter = 1;

    for iter in 1..=max_iterations.max(1) {
        final_iter = iter;

        // Evaluate actual internal resisting force vector at Gauss points with inhomogeneous material
        let f_int = compute_internal_forces(mesh, &u_raw, reference, strategy);

        let mut residual = vec![0.0; total_dofs];
        let mut r_norm_sq = 0.0;

        for i in 0..total_dofs {
            let node_idx = i / dim;
            if fixed_nodes.contains(&node_idx) {
                residual[i] = 0.0;
            } else {
                let r_val = f_ext[i] - f_int[i];
                residual[i] = r_val;
                r_norm_sq += r_val * r_val;
            }
        }

        let r_norm = r_norm_sq.sqrt();
        let rel_error = r_norm / f_ext_norm;
        println!("       Iter {}: rel_error = {:.6e}, r_norm = {:.4e}", iter, rel_error, r_norm);

        convergence_history.push(ConvergenceLog {
            iteration: iter,
            residual_norm: r_norm,
            relative_error: rel_error,
        });

        if rel_error < tolerance {
            converged = true;
            break;
        }

        // --- PHASE 5: VARIATIONAL BOUNDARY CORRECTION VIA BEM ---
        // Convert boundary residuals to boundary traction increments
        let mut delta_bnd_t = vec![[0.0, 0.0]; num_bnd_elems];
        let mut delta_rhs = vec![0.0; total_bnd_dofs];

        for (j, bnd) in mesh.boundary_elements.iter().enumerate() {
            if bnd.tag == "Left" {
                continue;
            }
            let len_j = elem_lengths[j];
            let n0 = bnd.nodes[0];
            let n1 = bnd.nodes[1];

            // Average residual force on element nodes converted to traction increment
            let r_x = 0.5 * (residual[n0 * dim + 0] + residual[n1 * dim + 0]) / len_j;
            let r_y = 0.5 * (residual[n0 * dim + 1] + residual[n1 * dim + 1]) / len_j;
            delta_bnd_t[j] = [r_x, r_y];

            for row in 0..total_bnd_dofs {
                delta_rhs[row] += g_mat[(row, j * dim + 0)] * r_x + g_mat[(row, j * dim + 1)] * r_y;
            }
        }

        let mut delta_rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);
        for i in 0..total_bnd_dofs {
            delta_rhs_mat[(i, 0)] = delta_rhs[i];
        }
        let delta_sol = lu.solve(&delta_rhs_mat);

        let mut delta_bnd_u = vec![[0.0, 0.0]; num_bnd_elems];
        for j in 0..num_bnd_elems {
            if mesh.boundary_elements[j].tag != "Left" {
                delta_bnd_u[j] = [delta_sol[(j * dim + 0, 0)], delta_sol[(j * dim + 1, 0)]];
            }
        }

        // Apply boundary correction to nodal displacements
        for i in 0..num_nodes {
            if fixed_nodes.contains(&i) {
                continue;
            }

            if let Some(&(sum_ux, sum_uy, count)) = bnd_node_displacements.get(&i) {
                // Average delta_bnd_u for boundary node
                let mut d_ux = 0.0;
                let mut d_uy = 0.0;
                let mut c = 0;
                for (j, bnd) in mesh.boundary_elements.iter().enumerate() {
                    if bnd.nodes.contains(&i) && bnd.tag != "Left" {
                        d_ux += delta_bnd_u[j][0];
                        d_uy += delta_bnd_u[j][1];
                        c += 1;
                    }
                }
                if c > 0 {
                    u_raw[i * dim + 0] += omega * (d_ux / c as f64);
                    u_raw[i * dim + 1] += omega * (d_uy / c as f64);
                }
            } else {
                // Interior node: propagate delta_u via Somigliana boundary integral
                let p = [mesh.nodes[i][0], mesh.nodes[i][1]];
                let delta_u_som = evaluate_somigliana_displacement_at_point(
                    p,
                    mesh,
                    &delta_bnd_u,
                    &delta_bnd_t,
                    mu,
                    nu,
                );
                u_raw[i * dim + 0] += omega * delta_u_som[0];
                u_raw[i * dim + 1] += omega * delta_u_som[1];
            }
        }
    }

    // --- PHASE 6: EXTRACT RESULTS ---
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let ux = u_raw[i * dim + 0];
        let uy = u_raw[i * dim + 1];
        let uz = 0.0;

        let mag = (ux * ux + uy * uy).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }
        displacements.push([ux, uy, uz]);

        let p = mesh.nodes[i];
        deformed_nodes.push([p[0] + ux, p[1] + uy, p[2]]);
    }

    let von_mises_stresses = compute_nodal_von_mises(mesh, &displacements, reference, strategy);
    let max_vm = von_mises_stresses.iter().cloned().fold(0.0, f64::max);
    let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;

    SimulationResult {
        solver_mode: SolverMode::BEMFEM,
        converged,
        iterations: final_iter,
        displacements,
        deformed_nodes,
        von_mises_stresses,
        max_displacement: max_disp,
        max_von_mises: max_vm,
        convergence_history,
        execution_time_ms: exec_time,
    }
}
