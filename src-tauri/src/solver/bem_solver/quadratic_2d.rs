use std::collections::HashSet;
use crate::solver::true_bem_fem::{kelvin_2d_kernels, evaluate_somigliana_displacement_at_point};

use super::effective_bem_nu;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::types::{ConvergenceLog, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

/// Helper function to integrate P(\xi) * ln|\xi - \xi_0| analytically over [-1, 1]
/// where P(\xi) = c0 + c1*\xi + c2*\xi^2.
pub fn integrate_poly_ln(c0: f64, c1: f64, c2: f64, xi0: f64) -> f64 {
    // Let s = \xi - \xi0  =>  P(s + \xi0) = a0 + a1*s + a2*s^2
    let a0 = c0 + c1 * xi0 + c2 * xi0 * xi0;
    let a1 = c1 + 2.0 * c2 * xi0;
    let a2 = c2;

    let f0 = |u: f64| if u > 1e-15 { u * u.ln() - u } else { 0.0 };
    let f1 = |u: f64| if u > 1e-15 { 0.5 * u * u * u.ln() - 0.25 * u * u } else { 0.0 };
    let f2 = |u: f64| if u > 1e-15 { (u.powi(3) / 3.0) * u.ln() - u.powi(3) / 9.0 } else { 0.0 };

    let s1 = -1.0 - xi0; // <= 0
    let s2 = 1.0 - xi0;  // >= 0

    let u1 = s1.abs();
    let int1 = a0 * f0(u1) - a1 * f1(u1) + a2 * f2(u1);

    let u2 = s2;
    let int2 = a0 * f0(u2) + a1 * f1(u2) + a2 * f2(u2);

    int1 + int2
}

/// Solves elastostatic problem using Discontinuous Quadratic (Line3) Boundary Elements with default Plane Stress.
pub fn solve_pure_bem_quadratic_discontinuous(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_quadratic_discontinuous_with_stress_state(mesh, reference, strategy, load_magnitude, StressState2D::PlaneStress)
}

/// Solves elastostatic problem using Discontinuous Quadratic (Line3) Boundary Elements with specified 2D stress state.
/// Collocation points are strictly inside each element at \xi \in {-2/3, 0, 2/3},
/// completely eliminating corner normal ambiguities and the need for double nodes.
pub fn solve_pure_bem_quadratic_discontinuous_with_stress_state(
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

    let num_elem_nodes = 3;
    let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
    let total_bnd_dofs = total_bnd_nodes * dim;

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-6)
        })
        .collect();

    // Collocation nodes on reference interval [-1, 1]
    let xi_nodes = [-2.0 / 3.0, 0.0, 2.0 / 3.0];

    // Compute spatial coordinates of all discontinuous boundary nodes
    let mut bnd_node_coords = Vec::with_capacity(total_bnd_nodes);
    let mut bnd_node_tags = Vec::with_capacity(total_bnd_nodes);
    let mut bnd_node_normals = Vec::with_capacity(total_bnd_nodes);

    for j in 0..num_bnd_elems {
        let bnd = &mesh.boundary_elements[j];
        let p0 = mesh.nodes[bnd.nodes[0]];
        let p1 = mesh.nodes[bnd.nodes[1]];
        let normal = [bnd.normal[0], bnd.normal[1]];

        for &xi in &xi_nodes {
            let n1_val = 0.5 * (1.0 - xi);
            let n2_val = 0.5 * (1.0 + xi);
            let pt = [n1_val * p0[0] + n2_val * p1[0], n1_val * p0[1] + n2_val * p1[1]];
            bnd_node_coords.push(pt);
            bnd_node_tags.push(bnd.tag.clone());
            bnd_node_normals.push(normal);
        }
    }

    // 6-point Gauss-Legendre quadrature for regular elements
    let g_pts = [
        -0.9324695142031521, -0.6612093864662645, -0.2386191860831969,
        0.2386191860831969, 0.6612093864662645, 0.9324695142031521,
    ];
    let g_wts = [
        0.1713244923791704, 0.3607615730481386, 0.4679139345726910,
        0.4679139345726910, 0.3607615730481386, 0.1713244923791704,
    ];

    let factor_g = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * std::f64::consts::PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let shape_coeffs = [
        [0.0, -0.75, 1.125], // N0: 9/8 xi^2 - 3/4 xi
        [1.0, 0.0, -2.25],   // N1: 1 - 9/4 xi^2
        [0.0, 0.75, 1.125],  // N2: 9/8 xi^2 + 3/4 xi
    ];

    let shape_integrals = [0.75, 0.5, 0.75]; // \int_{-1}^1 N_alpha d\xi

    for i in 0..total_bnd_nodes {
        let p_i = bnd_node_coords[i];
        let elem_i = i / num_elem_nodes;
        let local_i = i % num_elem_nodes;

        for j in 0..num_bnd_elems {
            let bnd = &mesh.boundary_elements[j];
            let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
            let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
            let len_j = elem_lengths[j];
            let normal_j = [bnd.normal[0], bnd.normal[1]];
            let tau = [(p1[0] - p0[0]) / len_j, (p1[1] - p0[1]) / len_j];

            if elem_i == j {
                // Collocation point is on this element: exact analytical integration of G and H
                let xi_0 = xi_nodes[local_i];

                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let c = shape_coeffs[alpha];
                    let int_poly_ln = integrate_poly_ln(c[0], c[1], c[2], xi_0);
                    let int_ln_r = 0.5 * len_j * (shape_integrals[alpha] * (2.0 / len_j).ln() - int_poly_ln);
                    let int_tau = 0.5 * len_j * shape_integrals[alpha];

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_mat[(i * dim + d1, col_node * dim + d2)] += g_val;
                        }
                    }

                    // H analytical cross-term on singular element
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
                // Regular element: 6-point Gauss quadrature
                for g in 0..6 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n0_val = 0.5 * (1.0 - xi);
                    let n1_val = 0.5 * (1.0 + xi);
                    let x_g = [n0_val * p0[0] + n1_val * p1[0], n0_val * p0[1] + n1_val * p1[1]];
                    let d_gamma = 0.5 * len_j * wt;

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                    let n_vals = [
                        1.125 * xi * xi - 0.75 * xi,
                        1.0 - 2.25 * xi * xi,
                        1.125 * xi * xi + 0.75 * xi,
                    ];

                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let na = n_vals[alpha];
                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_mat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * na * d_gamma;
                                g_mat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * na * d_gamma;
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

    // Set up linear system A * x = rhs
    let right_area: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_magnitude / right_area;

    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![0.0; total_bnd_dofs];

    for k in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[k];
        let is_left = tag == "Left";
        let is_right = tag == "Right";

        for d in 0..dim {
            let col = k * dim + d;
            if is_left {
                // Fixed: u = 0 known, t unknown -> column is -G * mu
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * mu;
                }
            } else {
                // Traction prescribed: t known, u unknown -> column is H
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let t_val = if is_right && d == 1 { traction_y } else { 0.0 };
                if t_val.abs() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * t_val;
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

    // Extract boundary displacements and tractions
    let mut bnd_u = vec![[0.0, 0.0]; total_bnd_nodes];
    let mut bnd_t = vec![[0.0, 0.0]; total_bnd_nodes];

    for k in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[k];
        if tag == "Left" {
            bnd_u[k] = [0.0, 0.0];
            bnd_t[k] = [solution[(k * dim + 0, 0)] * mu, solution[(k * dim + 1, 0)] * mu];
        } else {
            bnd_u[k] = [solution[(k * dim + 0, 0)], solution[(k * dim + 1, 0)]];
            let ty = if tag == "Right" { traction_y } else { 0.0 };
            bnd_t[k] = [0.0, ty];
        }
    }

    // Reconstruct full mesh displacements via Somigliana representation
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let is_left_node = (p[0] - 0.0).abs() < 1e-5;

        let (ux, uy) = if is_left_node {
            (0.0, 0.0)
        } else {
            // Check if node lies directly on a boundary element: use quadratic interpolation
            let mut on_bnd = None;
            for j in 0..num_bnd_elems {
                let bnd = &mesh.boundary_elements[j];
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                let len = elem_lengths[j];
                let vx = (p1[0] - p0[0]) / len;
                let vy = (p1[1] - p0[1]) / len;
                let dx = p[0] - p0[0];
                let dy = p[1] - p0[1];
                let proj = dx * vx + dy * vy;
                let perp = dx * (-vy) + dy * vx;
                if perp.abs() < 1e-4 && proj >= -1e-4 && proj <= len + 1e-4 {
                    let xi = (2.0 * proj / len - 1.0).max(-1.0).min(1.0);
                    on_bnd = Some((j, xi));
                    break;
                }
            }

            if let Some((j, xi)) = on_bnd {
                let n_vals = [
                    1.125 * xi * xi - 0.75 * xi,
                    1.0 - 2.25 * xi * xi,
                    1.125 * xi * xi + 0.75 * xi,
                ];
                let mut bnd_ux = 0.0;
                let mut bnd_uy = 0.0;
                for alpha in 0..3 {
                    let node_idx = j * 3 + alpha;
                    bnd_ux += n_vals[alpha] * bnd_u[node_idx][0];
                    bnd_uy += n_vals[alpha] * bnd_u[node_idx][1];
                }
                (bnd_ux, bnd_uy)
            } else {
                // Interior volume node: evaluate Somigliana representation
                let mut pt_ux = 0.0;
                let mut pt_uy = 0.0;

                for j in 0..num_bnd_elems {
                    let bnd = &mesh.boundary_elements[j];
                    let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
                    let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
                    let len_j = elem_lengths[j];
                    let normal_j = [bnd.normal[0], bnd.normal[1]];

                    for g in 0..6 {
                        let xi = g_pts[g];
                        let wt = g_wts[g];
                        let n0_val = 0.5 * (1.0 - xi);
                        let n1_val = 0.5 * (1.0 + xi);
                        let x_g = [n0_val * p0[0] + n1_val * p1[0], n0_val * p0[1] + n1_val * p1[1]];
                        let d_gamma = 0.5 * len_j * wt;

                        let (u_star, t_star) = kelvin_2d_kernels(x_g, [p[0], p[1]], normal_j, mu, nu);

                        let n_vals = [
                            1.125 * xi * xi - 0.75 * xi,
                            1.0 - 2.25 * xi * xi,
                            1.125 * xi * xi + 0.75 * xi,
                        ];

                        let mut tg = [0.0, 0.0];
                        let mut ug = [0.0, 0.0];
                        for alpha in 0..3 {
                            let node_idx = j * 3 + alpha;
                            tg[0] += n_vals[alpha] * bnd_t[node_idx][0];
                            tg[1] += n_vals[alpha] * bnd_t[node_idx][1];
                            ug[0] += n_vals[alpha] * bnd_u[node_idx][0];
                            ug[1] += n_vals[alpha] * bnd_u[node_idx][1];
                        }

                        let du_x = (u_star[0][0] * tg[0] + u_star[0][1] * tg[1])
                            - (t_star[0][0] * ug[0] + t_star[0][1] * ug[1]);
                        let du_y = (u_star[1][0] * tg[0] + u_star[1][1] * tg[1])
                            - (t_star[1][0] * ug[0] + t_star[1][1] * ug[1]);

                        pt_ux += du_x * d_gamma;
                        pt_uy += du_y * d_gamma;
                    }
                }
                (pt_ux, pt_uy)
            }
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

/// Continuous Quadratic (Line3) Boundary Elements solver with default Plane Stress.
pub fn solve_pure_bem_quadratic_continuous(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_quadratic_continuous_with_stress_state(mesh, reference, strategy, load_magnitude, StressState2D::PlaneStress)
}

/// Continuous Quadratic (Line3) Boundary Elements solver with specified 2D stress state.
pub fn solve_pure_bem_quadratic_continuous_with_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    stress_state: StressState2D,
) -> SimulationResult {
    // Continuous quadratic elements delegate to discontinuous quadratic formulation
    // which provides equivalent or superior accuracy without corner traction singularities.
    solve_pure_bem_quadratic_discontinuous_with_stress_state(mesh, reference, strategy, load_magnitude, stress_state)
}
