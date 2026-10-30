use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::types::{ConvergenceLog, SimulationResult, SolverMode};
use faer::prelude::SpSolver;
use faer::Mat;

/// 3D Pure BEM solver using linear triangular surface patches (Tri3 boundary elements).
pub fn solve_pure_bem_tri3_3d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    strategy: &dyn MaterialFieldStrategy,
    load_magnitude: f64,
) -> SimulationResult {
    let start_time = std::time::Instant::now();
    let dim = 3;
    let mu = reference.mu0();
    let nu = reference.nu0;

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;

    for p in &mesh.nodes {
        if p[0] < min_x { min_x = p[0]; }
        if p[0] > max_x { max_x = p[0]; }
        if p[1] < min_y { min_y = p[1]; }
        if p[1] > max_y { max_y = p[1]; }
        if p[2] < min_z { min_z = p[2]; }
        if p[2] > max_z { max_z = p[2]; }
    }

    let length = (max_x - min_x).max(1e-3);
    let height = (max_y - min_y).max(1e-3);
    let depth = (max_z - min_z).max(1e-3);

    let bnd_nx = (length / 0.5).round().clamp(2.0, 8.0) as usize;
    let bnd_ny = (height / 0.5).round().clamp(1.0, 4.0) as usize;
    let bnd_nz = (depth / 0.5).round().clamp(1.0, 4.0) as usize;

    let quad_elems = crate::solver::oraiopoulos_bem_fem::generate_decoupled_boundary_mesh_3d(
        length, height, depth, bnd_nx, bnd_ny, bnd_nz,
    );

    // Subdivide each rectangular boundary patch into 2 Tri3 elements
    struct Tri3Patch {
        v0: [f64; 3],
        v1: [f64; 3],
        v2: [f64; 3],
        center: [f64; 3],
        normal: [f64; 3],
        area: f64,
        tag: String,
    }

    let mut tri_elems = Vec::with_capacity(quad_elems.len() * 2);
    for q in &quad_elems {
        let p0 = q.p0;
        let p1 = [p0[0] + q.v1[0], p0[1] + q.v1[1], p0[2] + q.v1[2]];
        let p2 = [p0[0] + q.v2[0], p0[1] + q.v2[1], p0[2] + q.v2[2]];
        let p3 = [p0[0] + q.v1[0] + q.v2[0], p0[1] + q.v1[1] + q.v2[1], p0[2] + q.v1[2] + q.v2[2]];

        let tri1_center = [
            (p0[0] + p1[0] + p2[0]) / 3.0,
            (p0[1] + p1[1] + p2[1]) / 3.0,
            (p0[2] + p1[2] + p2[2]) / 3.0,
        ];
        let tri2_center = [
            (p3[0] + p2[0] + p1[0]) / 3.0,
            (p3[1] + p2[1] + p1[1]) / 3.0,
            (p3[2] + p2[2] + p1[2]) / 3.0,
        ];
        let half_area = 0.5 * q.area;

        tri_elems.push(Tri3Patch {
            v0: p0,
            v1: p1,
            v2: p2,
            center: tri1_center,
            normal: q.normal,
            area: half_area,
            tag: q.tag.clone(),
        });

        tri_elems.push(Tri3Patch {
            v0: p3,
            v1: p2,
            v2: p1,
            center: tri2_center,
            normal: q.normal,
            area: half_area,
            tag: q.tag.clone(),
        });
    }

    let num_tri = tri_elems.len();
    let total_bnd_dofs = num_tri * dim;

    let mut h_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let factor_g_sing = 1.0 / (16.0 * std::f64::consts::PI * mu * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    // 3-point Hammer quadrature barycentric coordinates
    let hammer_pts = [
        [2.0 / 3.0, 1.0 / 6.0, 1.0 / 6.0],
        [1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0],
        [1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0],
    ];
    let hammer_weight = 1.0 / 3.0;

    for i in 0..num_tri {
        let xi_pt = tri_elems[i].center;

        for j in 0..num_tri {
            let elem_j = &tri_elems[j];
            let normal_j = elem_j.normal;
            let area_j = elem_j.area;

            if i == j {
                // Singular integration over flat triangle
                let i0 = 2.15 * area_j.sqrt();
                for d1 in 0..dim {
                    for d2 in 0..dim {
                        let delta = if d1 == d2 { 1.0 } else { 0.0 };
                        let g_val = factor_g_sing * (kappa * delta + (1.0 / 3.0) * delta) * i0;
                        g_mat[(i * dim + d1, j * dim + d2)] += g_val;
                    }
                }
            } else {
                for pt in &hammer_pts {
                    let x_g = [
                        pt[0] * elem_j.v0[0] + pt[1] * elem_j.v1[0] + pt[2] * elem_j.v2[0],
                        pt[0] * elem_j.v0[1] + pt[1] * elem_j.v1[1] + pt[2] * elem_j.v2[1],
                        pt[0] * elem_j.v0[2] + pt[1] * elem_j.v1[2] + pt[2] * elem_j.v2[2],
                    ];
                    let (u_star, t_star) = crate::solver::oraiopoulos_bem_fem::kelvin_3d_kernels(
                        x_g, xi_pt, normal_j, mu, nu,
                    );
                    let d_gamma = hammer_weight * area_j;

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

    // Rigid body translation condition for diagonal blocks of H
    for i in 0..num_tri {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for j in 0..num_tri {
                    if i != j {
                        sum += h_mat[(row, j * dim + d2)];
                    }
                }
                h_mat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Apply Boundary Conditions
    let area_right = height * depth;
    let traction_y = -load_magnitude / area_right;

    let mut a_mat = Mat::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs_mat = Mat::<f64>::zeros(total_bnd_dofs, 1);

    for j in 0..num_tri {
        let tag = &tri_elems[j].tag;
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

    let mut bnd_u = vec![[0.0; 3]; num_tri];
    let mut bnd_t = vec![[0.0; 3]; num_tri];

    for j in 0..num_tri {
        let tag = &tri_elems[j].tag;
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

    // Evaluate Somigliana displacement projection on all mesh nodes
    let num_nodes = mesh.nodes.len();
    let mut displacements = Vec::with_capacity(num_nodes);
    let mut deformed_nodes = Vec::with_capacity(num_nodes);
    let mut max_disp = 0.0;

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        if p[0] <= 1e-6 {
            displacements.push([0.0, 0.0, 0.0]);
            deformed_nodes.push(p);
            continue;
        }

        let mut u_pt = [0.0; 3];
        for j in 0..num_tri {
            let elem = &tri_elems[j];
            let u_j = bnd_u[j];
            let t_j = bnd_t[j];
            let normal_j = elem.normal;
            let area_j = elem.area;

            for pt in &hammer_pts {
                let x_g = [
                    pt[0] * elem.v0[0] + pt[1] * elem.v1[0] + pt[2] * elem.v2[0],
                    pt[0] * elem.v0[1] + pt[1] * elem.v1[1] + pt[2] * elem.v2[1],
                    pt[0] * elem.v0[2] + pt[1] * elem.v1[2] + pt[2] * elem.v2[2],
                ];
                let (u_star, t_star) = crate::solver::oraiopoulos_bem_fem::kelvin_3d_kernels(
                    x_g, p, normal_j, mu, nu,
                );
                let d_gamma = hammer_weight * area_j;

                for d1 in 0..dim {
                    let mut du = 0.0;
                    for d2 in 0..dim {
                        du += (u_star[d1][d2] * t_j[d2] - t_star[d1][d2] * u_j[d2]) * d_gamma;
                    }
                    u_pt[d1] += du;
                }
            }
        }

        let disp_mag = (u_pt[0].powi(2) + u_pt[1].powi(2) + u_pt[2].powi(2)).sqrt();
        if disp_mag > max_disp {
            max_disp = disp_mag;
        }

        displacements.push(u_pt);
        deformed_nodes.push([p[0] + u_pt[0], p[1] + u_pt[1], p[2] + u_pt[2]]);
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
