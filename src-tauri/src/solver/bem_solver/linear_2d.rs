use std::collections::HashSet;
use crate::solver::true_bem_fem::{kelvin_2d_kernels, evaluate_somigliana_displacement_at_point};

use super::effective_bem_nu;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::types::{ConvergenceLog, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

/// Solves elastostatic problem using Continuous Linear (Line2) Boundary Elements without double nodes with default Plane Stress.
pub fn solve_pure_bem_linear(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_linear_with_stress_state(mesh, reference, strategy, load_magnitude, StressState2D::PlaneStress)
}

/// Solves elastostatic problem using Continuous Linear (Line2) Boundary Elements without double nodes with specified 2D stress state.
pub fn solve_pure_bem_linear_with_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    stress_state: StressState2D,
) -> SimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_bnd_elems = mesh.boundary_elements.len();
    let num_nodes = mesh.nodes.len();

    let mu = reference.mu0();
    let nu = effective_bem_nu(reference.nu0, stress_state);

    // 1. Identify unique boundary nodes and build index map
    let mut bnd_nodes_set = HashSet::new();
    let mut left_nodes = HashSet::new();
    let mut right_nodes = HashSet::new();
    for bnd in &mesh.boundary_elements {
        for &n in &bnd.nodes {
            bnd_nodes_set.insert(n);
            if bnd.tag == "Left" {
                left_nodes.insert(n);
            } else if bnd.tag == "Right" {
                right_nodes.insert(n);
            }
        }
    }

    let mut bnd_nodes: Vec<usize> = bnd_nodes_set.into_iter().collect();
    bnd_nodes.sort();
    let num_bnd_nodes = bnd_nodes.len();
    let total_bnd_dofs = num_bnd_nodes * dim;

    let mut node_to_bnd_idx = std::collections::HashMap::new();
    for (idx, &n) in bnd_nodes.iter().enumerate() {
        node_to_bnd_idx.insert(n, idx);
    }

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6)
        })
        .collect();

    // 2. Build Linear BEM matrices H and G via collocation at boundary nodes
    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
    let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

    let factor_g = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * std::f64::consts::PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    for i in 0..num_bnd_nodes {
        let global_i = bnd_nodes[i];
        let p_i = [mesh.nodes[global_i][0], mesh.nodes[global_i][1]];

        for j in 0..num_bnd_elems {
            let bnd = &mesh.boundary_elements[j];
            let n_a = bnd.nodes[0];
            let n_b = bnd.nodes[1];
            let idx_a = node_to_bnd_idx[&n_a];
            let idx_b = node_to_bnd_idx[&n_b];

            let p0 = [mesh.nodes[n_a][0], mesh.nodes[n_a][1]];
            let p1 = [mesh.nodes[n_b][0], mesh.nodes[n_b][1]];
            let len_j = elem_lengths[j];
            let normal_j = [bnd.normal[0], bnd.normal[1]];
            let tau = [(p1[0] - p0[0]) / len_j, (p1[1] - p0[1]) / len_j];

            if global_i == n_a {
                // Singular at node a (\xi = -1)
                let int_ln1 = len_j * (0.5 * (1.0 / len_j).ln() + 0.75);
                let int_ln2 = len_j * (0.5 * (1.0 / len_j).ln() + 0.25);

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        let g1 = factor_g * (kappa * int_ln1 * delta + 0.5 * len_j * tau[d1] * tau[d2]);
                        let g2 = factor_g * (kappa * int_ln2 * delta + 0.5 * len_j * tau[d1] * tau[d2]);

                        g_mat[(i * dim + d1, idx_a * dim + d2)] += g1;
                        g_mat[(i * dim + d1, idx_b * dim + d2)] += g2;

                        // Analytical H contribution to adjacent node b
                        let h2 = factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]);
                        h_mat[(i * dim + d1, idx_b * dim + d2)] += h2;
                    }
                }
            } else if global_i == n_b {
                // Singular at node b (\xi = +1)
                let int_ln1 = len_j * (0.5 * (1.0 / len_j).ln() + 0.25);
                let int_ln2 = len_j * (0.5 * (1.0 / len_j).ln() + 0.75);

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        let g1 = factor_g * (kappa * int_ln1 * delta + 0.5 * len_j * tau[d1] * tau[d2]);
                        let g2 = factor_g * (kappa * int_ln2 * delta + 0.5 * len_j * tau[d1] * tau[d2]);

                        g_mat[(i * dim + d1, idx_a * dim + d2)] += g1;
                        g_mat[(i * dim + d1, idx_b * dim + d2)] += g2;

                        // Analytical H contribution to adjacent node a
                        let h1 = -factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]);
                        h_mat[(i * dim + d1, idx_a * dim + d2)] += h1;
                    }
                }
            } else {
                // Regular element: 4-point Gauss quadrature
                for g in 0..4 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n1_val = 0.5 * (1.0 - xi);
                    let n2_val = 0.5 * (1.0 + xi);
                    let x_g = [n1_val * p0[0] + n2_val * p1[0], n1_val * p0[1] + n2_val * p1[1]];
                    let d_gamma = 0.5 * len_j * wt;

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            h_mat[(i * dim + d1, idx_a * dim + d2)] += t_star[d1][d2] * n1_val * d_gamma;
                            h_mat[(i * dim + d1, idx_b * dim + d2)] += t_star[d1][d2] * n2_val * d_gamma;

                            g_mat[(i * dim + d1, idx_a * dim + d2)] += u_star[d1][d2] * n1_val * d_gamma;
                            g_mat[(i * dim + d1, idx_b * dim + d2)] += u_star[d1][d2] * n2_val * d_gamma;
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H
    for i in 0..num_bnd_nodes {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..num_bnd_nodes {
                    if i != k {
                        sum += h_mat[(row, k * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // 3. Boundary Conditions
    // Left boundary: u = 0 (Fixed). Node is fixed if it belongs to left_nodes.
    // Right boundary: t_y = -load_magnitude / Area.
    let right_area: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_magnitude / right_area;

    // Set up linear system A * x = rhs
    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![0.0; total_bnd_dofs];

    for k in 0..num_bnd_nodes {
        let global_node = bnd_nodes[k];
        let is_left = left_nodes.contains(&global_node);
        let is_right = !is_left && right_nodes.contains(&global_node);

        for d in 0..dim {
            let col = k * dim + d;
            if is_left {
                // u = 0 known, t unknown: column is -G * mu
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * mu;
                }
            } else {
                // t known, u unknown: keep H on LHS
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let t_known = if is_right && d == 1 { traction_y } else { 0.0 };
                if t_known.abs() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * t_known;
                    }
                }
            }
        }
    }

    // Solve system
    let lu = a_mat.full_piv_lu();
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);
    for i in 0..total_bnd_dofs {
        rhs_mat[(i, 0)] = rhs[i];
    }
    let solution = lu.solve(&rhs_mat);

    // Extract nodal boundary displacements and tractions
    let mut bnd_nodal_u = vec![[0.0, 0.0]; num_bnd_nodes];
    let mut bnd_nodal_t = vec![[0.0, 0.0]; num_bnd_nodes];

    for k in 0..num_bnd_nodes {
        let global_node = bnd_nodes[k];
        let is_left = left_nodes.contains(&global_node);
        let is_right = !is_left && right_nodes.contains(&global_node);

        if is_left {
            bnd_nodal_u[k] = [0.0, 0.0];
            bnd_nodal_t[k] = [solution[(k * dim + 0, 0)] * mu, solution[(k * dim + 1, 0)] * mu];
        } else {
            bnd_nodal_u[k] = [solution[(k * dim + 0, 0)], solution[(k * dim + 1, 0)]];
            let ty = if is_right { traction_y } else { 0.0 };
            bnd_nodal_t[k] = [0.0, ty];
        }
    }

    // Reconstruct full mesh displacements
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let (ux, uy) = if left_nodes.contains(&i) {
            (0.0, 0.0)
        } else if let Some(&bnd_idx) = node_to_bnd_idx.get(&i) {
            // Node is directly on boundary: use exact solved boundary displacement!
            (bnd_nodal_u[bnd_idx][0], bnd_nodal_u[bnd_idx][1])
        } else {
            // Interior volume node: evaluate Somigliana with linear shape functions
            evaluate_somigliana_displacement_linear(
                [p[0], p[1]],
                mesh,
                &node_to_bnd_idx,
                &bnd_nodal_u,
                &bnd_nodal_t,
                mu,
                nu,
            )
        };

        let mag = (ux * ux + uy * uy).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }

        displacements.push([ux, uy, 0.0]);
        deformed_nodes.push([p[0] + ux, p[1] + uy, p[2]]);
    }

    let von_mises = crate::solver::fem_solver::compute_nodal_von_mises(mesh, &displacements, reference, strategy);
    let max_vm = von_mises.iter().cloned().fold(0.0, f64::max);
    let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;

    SimulationResult {
        solver_mode: SolverMode::PureBEM,
        converged: true,
        iterations: 1,
        displacements,
        deformed_nodes,
        von_mises_stresses: von_mises,
        max_displacement: max_disp,
        max_von_mises: max_vm,
        convergence_history: vec![ConvergenceLog {
            iteration: 1,
            residual_norm: 0.0,
            relative_error: 0.0,
        }],
        execution_time_ms: exec_time,
    }
}

/// Evaluates interior displacement vector u(p) at point p via Somigliana boundary integral
/// using linear interpolation of boundary displacements and tractions.
pub fn evaluate_somigliana_displacement_linear(
    p: [f64; 2],
    mesh: &MeshData,
    node_to_bnd_idx: &std::collections::HashMap<usize, usize>,
    bnd_nodal_u: &[[f64; 2]],
    bnd_nodal_t: &[[f64; 2]],
    mu: f64,
    nu: f64,
) -> (f64, f64) {
    let mut ux = 0.0;
    let mut uy = 0.0;

    let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
    let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

    for bnd in &mesh.boundary_elements {
        let n_a = bnd.nodes[0];
        let n_b = bnd.nodes[1];
        let idx_a = node_to_bnd_idx[&n_a];
        let idx_b = node_to_bnd_idx[&n_b];

        let p0 = [mesh.nodes[n_a][0], mesh.nodes[n_a][1]];
        let p1 = [mesh.nodes[n_b][0], mesh.nodes[n_b][1]];
        let len_j = ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6);
        let normal = [bnd.normal[0], bnd.normal[1]];

        let u_a = bnd_nodal_u[idx_a];
        let u_b = bnd_nodal_u[idx_b];
        let t_a = bnd_nodal_t[idx_a];
        let t_b = bnd_nodal_t[idx_b];

        for g in 0..4 {
            let xi = g_pts[g];
            let wt = g_wts[g];
            let n1 = 0.5 * (1.0 - xi);
            let n2 = 0.5 * (1.0 + xi);
            let x_g = [n1 * p0[0] + n2 * p1[0], n1 * p0[1] + n2 * p1[1]];
            let d_gamma = 0.5 * len_j * wt;

            let u_g = [n1 * u_a[0] + n2 * u_b[0], n1 * u_a[1] + n2 * u_b[1]];
            let t_g = [n1 * t_a[0] + n2 * t_b[0], n1 * t_a[1] + n2 * t_b[1]];

            let (u_star, t_star) = kelvin_2d_kernels(x_g, p, normal, mu, nu);

            let du_x = (u_star[0][0] * t_g[0] + u_star[0][1] * t_g[1])
                - (t_star[0][0] * u_g[0] + t_star[0][1] * u_g[1]);
            let du_y = (u_star[1][0] * t_g[0] + u_star[1][1] * t_g[1])
                - (t_star[1][0] * u_g[0] + t_star[1][1] * u_g[1]);

            ux += du_x * d_gamma;
            uy += du_y * d_gamma;
        }
    }

    (ux, uy)
}

/// Solves elastostatic problem using Continuous Linear (Line2) Boundary Elements with Double Nodes at corners with default Plane Stress.
pub fn solve_pure_bem_linear_dn(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_linear_dn_with_stress_state(mesh, reference, strategy, load_magnitude, StressState2D::PlaneStress)
}

/// Solves elastostatic problem using Continuous Linear (Line2) Boundary Elements with Double Nodes at corners with specified 2D stress state.
pub fn solve_pure_bem_linear_dn_with_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    stress_state: StressState2D,
) -> SimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_bnd_elems = mesh.boundary_elements.len();
    let num_nodes = mesh.nodes.len();

    let mu = reference.mu0();
    let nu = effective_bem_nu(reference.nu0, stress_state);

    // 1. Identify unique boundary nodes and build index map
    let mut bnd_nodes_set = HashSet::new();
    let mut left_nodes = HashSet::new();
    let mut right_nodes = HashSet::new();
    for bnd in &mesh.boundary_elements {
        for &n in &bnd.nodes {
            bnd_nodes_set.insert(n);
            if bnd.tag == "Left" {
                left_nodes.insert(n);
            } else if bnd.tag == "Right" {
                right_nodes.insert(n);
            }
        }
    }

    let mut bnd_nodes: Vec<usize> = bnd_nodes_set.into_iter().collect();
    bnd_nodes.sort();
    let num_bnd_nodes = bnd_nodes.len();
    let total_bnd_dofs = num_bnd_nodes * dim;

    let mut node_to_bnd_idx = std::collections::HashMap::new();
    for (idx, &n) in bnd_nodes.iter().enumerate() {
        node_to_bnd_idx.insert(n, idx);
    }

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6)
        })
        .collect();

    // 2. Build Linear BEM matrices H and G_left
    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_left_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![0.0; total_bnd_dofs];

    let right_area: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_magnitude / right_area;

    let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
    let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

    let factor_g = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * std::f64::consts::PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    for i in 0..num_bnd_nodes {
        let global_i = bnd_nodes[i];
        let p_i = [mesh.nodes[global_i][0], mesh.nodes[global_i][1]];

        for j in 0..num_bnd_elems {
            let bnd = &mesh.boundary_elements[j];
            let is_left_elem = bnd.tag == "Left";
            let is_right_elem = bnd.tag == "Right";

            let n_a = bnd.nodes[0];
            let n_b = bnd.nodes[1];
            let idx_a = node_to_bnd_idx[&n_a];
            let idx_b = node_to_bnd_idx[&n_b];

            let p0 = [mesh.nodes[n_a][0], mesh.nodes[n_a][1]];
            let p1 = [mesh.nodes[n_b][0], mesh.nodes[n_b][1]];
            let len_j = elem_lengths[j];
            let normal_j = [bnd.normal[0], bnd.normal[1]];
            let tau = [(p1[0] - p0[0]) / len_j, (p1[1] - p0[1]) / len_j];

            let mut elem_g1 = [[0.0; 2]; 2];
            let mut elem_g2 = [[0.0; 2]; 2];
            let mut elem_h1 = [[0.0; 2]; 2];
            let mut elem_h2 = [[0.0; 2]; 2];

            if global_i == n_a {
                // Singular at node a (\xi = -1)
                let int_ln1 = len_j * (0.5 * (1.0 / len_j).ln() + 0.75);
                let int_ln2 = len_j * (0.5 * (1.0 / len_j).ln() + 0.25);

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        elem_g1[d1][d2] = factor_g * (kappa * int_ln1 * delta + 0.5 * len_j * tau[d1] * tau[d2]);
                        elem_g2[d1][d2] = factor_g * (kappa * int_ln2 * delta + 0.5 * len_j * tau[d1] * tau[d2]);

                        elem_h2[d1][d2] = factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]);
                    }
                }
            } else if global_i == n_b {
                // Singular at node b (\xi = +1)
                let int_ln1 = len_j * (0.5 * (1.0 / len_j).ln() + 0.25);
                let int_ln2 = len_j * (0.5 * (1.0 / len_j).ln() + 0.75);

                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        elem_g1[d1][d2] = factor_g * (kappa * int_ln1 * delta + 0.5 * len_j * tau[d1] * tau[d2]);
                        elem_g2[d1][d2] = factor_g * (kappa * int_ln2 * delta + 0.5 * len_j * tau[d1] * tau[d2]);

                        elem_h1[d1][d2] = -factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]);
                    }
                }
            } else {
                // Regular element: 4-point Gauss quadrature
                for g in 0..4 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n1_val = 0.5 * (1.0 - xi);
                    let n2_val = 0.5 * (1.0 + xi);
                    let x_g = [n1_val * p0[0] + n2_val * p1[0], n1_val * p0[1] + n2_val * p1[1]];
                    let d_gamma = 0.5 * len_j * wt;

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            elem_h1[d1][d2] += t_star[d1][d2] * n1_val * d_gamma;
                            elem_h2[d1][d2] += t_star[d1][d2] * n2_val * d_gamma;

                            elem_g1[d1][d2] += u_star[d1][d2] * n1_val * d_gamma;
                            elem_g2[d1][d2] += u_star[d1][d2] * n2_val * d_gamma;
                        }
                    }
                }
            }

            // Assemble H matrix (geometric matrix for entire boundary)
            for d1 in 0..dim {
                for d2 in 0..dim {
                    h_mat[(i * dim + d1, idx_a * dim + d2)] += elem_h1[d1][d2];
                    h_mat[(i * dim + d1, idx_b * dim + d2)] += elem_h2[d1][d2];

                    if is_left_elem {
                        // G_left receives G terms ONLY from Left elements!
                        g_left_mat[(i * dim + d1, idx_a * dim + d2)] += elem_g1[d1][d2];
                        g_left_mat[(i * dim + d1, idx_b * dim + d2)] += elem_g2[d1][d2];
                    }
                }
            }

            // RHS contribution from known tractions on Right elements
            if is_right_elem {
                let ty = traction_y;
                for d1 in 0..dim {
                    rhs[i * dim + d1] += elem_g1[d1][1] * ty + elem_g2[d1][1] * ty;
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H
    for i in 0..num_bnd_nodes {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..num_bnd_nodes {
                    if i != k {
                        sum += h_mat[(row, k * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Form final linear system A * x = rhs
    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    for k in 0..num_bnd_nodes {
        let global_node = bnd_nodes[k];
        let is_left = left_nodes.contains(&global_node);

        for d in 0..dim {
            let col = k * dim + d;
            if is_left {
                // Column is -G_left * mu
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_left_mat[(row, col)] * mu;
                }
            } else {
                // Column is H
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
            }
        }
    }

    // Solve system
    let lu = a_mat.full_piv_lu();
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);
    for i in 0..total_bnd_dofs {
        rhs_mat[(i, 0)] = rhs[i];
    }
    let solution = lu.solve(&rhs_mat);

    // Extract nodal boundary displacements and tractions
    let mut bnd_nodal_u = vec![[0.0, 0.0]; num_bnd_nodes];
    let mut bnd_nodal_t = vec![[0.0, 0.0]; num_bnd_nodes];

    for k in 0..num_bnd_nodes {
        let global_node = bnd_nodes[k];
        let is_left = left_nodes.contains(&global_node);
        let is_right = !is_left && right_nodes.contains(&global_node);

        if is_left {
            bnd_nodal_u[k] = [0.0, 0.0];
            bnd_nodal_t[k] = [solution[(k * dim + 0, 0)] * mu, solution[(k * dim + 1, 0)] * mu];
        } else {
            bnd_nodal_u[k] = [solution[(k * dim + 0, 0)], solution[(k * dim + 1, 0)]];
            let ty = if is_right { traction_y } else { 0.0 };
            bnd_nodal_t[k] = [0.0, ty];
        }
    }

    // Reconstruct full mesh displacements
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let (ux, uy) = if left_nodes.contains(&i) {
            (0.0, 0.0)
        } else if let Some(&bnd_idx) = node_to_bnd_idx.get(&i) {
            (bnd_nodal_u[bnd_idx][0], bnd_nodal_u[bnd_idx][1])
        } else {
            evaluate_somigliana_displacement_linear(
                [p[0], p[1]],
                mesh,
                &node_to_bnd_idx,
                &bnd_nodal_u,
                &bnd_nodal_t,
                mu,
                nu,
            )
        };

        let mag = (ux * ux + uy * uy).sqrt();
        if mag > max_disp {
            max_disp = mag;
        }

        displacements.push([ux, uy, 0.0]);
        deformed_nodes.push([p[0] + ux, p[1] + uy, p[2]]);
    }

    let von_mises = crate::solver::fem_solver::compute_nodal_von_mises(mesh, &displacements, reference, strategy);
    let max_vm = von_mises.iter().cloned().fold(0.0, f64::max);
    let exec_time = start_time.elapsed().as_secs_f64() * 1000.0;

    SimulationResult {
        solver_mode: SolverMode::PureBEM,
        converged: true,
        iterations: 1,
        displacements,
        deformed_nodes,
        von_mises_stresses: von_mises,
        max_displacement: max_disp,
        max_von_mises: max_vm,
        convergence_history: vec![ConvergenceLog {
            iteration: 1,
            residual_norm: 0.0,
            relative_error: 0.0,
        }],
        execution_time_ms: exec_time,
    }
}
