use std::collections::HashSet;
use crate::solver::true_bem_fem::{kelvin_2d_kernels, evaluate_somigliana_displacement_at_point};

use super::effective_bem_nu;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::types::{ConvergenceLog, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

/// Solves elastostatic problem using Constant Boundary Elements with default Plane Stress.
pub fn solve_pure_bem_constant(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    solve_pure_bem_constant_with_stress_state(mesh, reference, strategy, load_magnitude, StressState2D::PlaneStress)
}

/// Solves elastostatic problem using Constant Boundary Elements with specified 2D stress state.
pub fn solve_pure_bem_constant_with_stress_state(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
    stress_state: StressState2D,
) -> SimulationResult {
    let start_time = std::time::Instant::now();
    let dim = mesh.dimension;
    let num_bnd_elems = mesh.boundary_elements.len();
    let total_bnd_dofs = num_bnd_elems * dim;
    let num_nodes = mesh.nodes.len();

    let mu = reference.mu0();
    let nu = effective_bem_nu(reference.nu0, stress_state);

    // 1. Build BEM matrices H and G via collocation at element midpoints
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
                // 4-point Gauss-Legendre quadrature for accurate integration of 1/r and ln(1/r)
                let p0 = mesh.nodes[mesh.boundary_elements[j].nodes[0]];
                let p1 = mesh.nodes[mesh.boundary_elements[j].nodes[1]];
                let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
                let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

                for g in 0..4 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n0 = 0.5 * (1.0 - xi);
                    let n1 = 0.5 * (1.0 + xi);
                    let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];
                    let d_gamma = 0.5 * len_j * wt;

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

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

    // Compute diagonal blocks of H using the exact Rigid Body Displacement condition:
    // For rigid body translation in direction d: \sum_j H_{row, j*dim + d} = 0
    // => H_{i*dim + d, i*dim + d} = - \sum_{j != i} H_{i*dim + d, j*dim + d}
    for i in 0..num_bnd_elems {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for j in 0..num_bnd_elems {
                    if i != j {
                        sum += h_mat[(row, j * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
        if i == 0 {
            println!("  [DEBUG H_00 Block]: H_xx={:.4}, H_xy={:.4}, H_yx={:.4}, H_yy={:.4}",
                h_mat[(0, 0)], h_mat[(0, 1)], h_mat[(1, 0)], h_mat[(1, 1)]);
        }
    }

    // 2. Prescribed Boundary Conditions
    // Left boundary: u = 0 (Fixed)
    // Right boundary: t_y = -load_magnitude / Area
    let right_area: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_magnitude / right_area;

    // 3. Set up A * x = y system with scaling by mu for optimal conditioning
    // Scale tractions: t_scale = t / mu  =>  A has columns of H (~1) and -G*mu (~L ~1)
    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![0.0; total_bnd_dofs];

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        for d in 0..dim {
            let col = j * dim + d;
            if tag == "Left" {
                // u is known (= 0), t is unknown: column is -G * mu, unknown is t / mu
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * mu;
                }
            } else {
                // t is known, u is unknown: keep H on LHS
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

    // 4. Solve BEM linear system A * x = rhs using Full LU
    let lu = a_mat.full_piv_lu();
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);
    for i in 0..total_bnd_dofs {
        rhs_mat[(i, 0)] = rhs[i];
    }
    let solution = lu.solve(&rhs_mat);

    // Extract boundary element displacements and tractions
    let mut bnd_u = vec![[0.0, 0.0]; num_bnd_elems];
    let mut bnd_t = vec![[0.0, 0.0]; num_bnd_elems];
    let mut fixed_nodes = HashSet::new();
    let mut bnd_node_displacements: std::collections::HashMap<usize, (f64, f64, usize)> = std::collections::HashMap::new();

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        if tag == "Left" {
            bnd_u[j] = [0.0, 0.0];
            // Unscale traction: t = t_scaled * mu
            bnd_t[j] = [solution[(j * dim + 0, 0)] * mu, solution[(j * dim + 1, 0)] * mu];
        } else {
            bnd_u[j] = [solution[(j * dim + 0, 0)], solution[(j * dim + 1, 0)]];
            let ty = if tag == "Right" { traction_y } else { 0.0 };
            bnd_t[j] = [0.0, ty];
        }

        for &n in &mesh.boundary_elements[j].nodes {
            if tag == "Left" {
                fixed_nodes.insert(n);
            }
            let entry = bnd_node_displacements.entry(n).or_insert((0.0, 0.0, 0));
            entry.0 += bnd_u[j][0];
            entry.1 += bnd_u[j][1];
            entry.2 += 1;
        }
    }

    let mut total_react_y = 0.0;
    let mut total_moment = 0.0;
    for j in 0..num_bnd_elems {
        let fx = bnd_t[j][0] * elem_lengths[j];
        let fy = bnd_t[j][1] * elem_lengths[j];
        total_react_y += fy;
        // Moment about origin (0, 0): r x F = x * fy - y * fx
        total_moment += elem_centers[j][0] * fy - elem_centers[j][1] * fx;
    }
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let (ux, uy) = if fixed_nodes.contains(&i) {
            (0.0, 0.0)
        } else if let Some(&(sum_ux, sum_uy, count)) = bnd_node_displacements.get(&i) {
            (sum_ux / count as f64, sum_uy / count as f64)
        } else {
            let u_som = evaluate_somigliana_displacement_at_point(
                [p[0], p[1]],
                mesh,
                &bnd_u,
                &bnd_t,
                mu,
                nu,
            );
            (u_som[0], u_som[1])
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
