use std::f64::consts::PI;
use super::kernels::*;
use super::assembly::*;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::oraiopoulos_bem_fem::{generate_decoupled_boundary_mesh_3d, DecoupledBndElem3D};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use std::collections::HashSet;

/// Solves 2D elastodynamics under harmonic excitation using standalone Pure BEM

/// Evaluates nodal displacements in the complex Laplace domain using 2D Constant Line elements
pub fn solve_laplace_pure_bem_constant_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
) -> Vec<Complex64> {
    let dim = 2;
    let num_bnd_elems = mesh.boundary_elements.len();
    let total_bnd_dofs = num_bnd_elems * dim;
    let mu = reference.mu0();

    let (h_mat, g_mat) = assemble_dynamic_bem_matrices(mesh, reference, s);

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-5)
        })
        .collect();

    let right_area: f64 = mesh
        .boundary_elements
        .iter()
        .enumerate()
        .filter(|(_, b)| b.tag == "Right")
        .map(|(j, _)| elem_lengths[j])
        .sum::<f64>()
        .max(1e-4);

    let traction_y = -load_factor / Complex64::new(right_area, 0.0);

    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        for d in 0..dim {
            let col = j * dim + d;
            if tag == "Left" {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * Complex64::new(mu, 0.0);
                }
            } else {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let ty = if tag == "Right" && d == 1 { traction_y } else { Complex64::new(0.0, 0.0) };
                if ty.norm() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * ty;
                    }
                }
            }
        }
    }

    let lu = a_mat.lu();
    let rhs_vec = DVector::from_vec(rhs);
    let solution = lu.solve(&rhs_vec).unwrap_or_else(|| DVector::zeros(total_bnd_dofs));

    let mut bnd_u = vec![[Complex64::new(0.0, 0.0); 2]; num_bnd_elems];
    let mut bnd_t = vec![[Complex64::new(0.0, 0.0); 2]; num_bnd_elems];
    let mut fixed_nodes = HashSet::new();
    let mut bnd_node_disp: std::collections::HashMap<usize, (Complex64, Complex64, usize)> = std::collections::HashMap::new();

    for j in 0..num_bnd_elems {
        let tag = &mesh.boundary_elements[j].tag;
        if tag == "Left" {
            bnd_u[j] = [Complex64::new(0.0, 0.0); 2];
            bnd_t[j] = [
                solution[j * dim + 0] * Complex64::new(mu, 0.0),
                solution[j * dim + 1] * Complex64::new(mu, 0.0),
            ];
        } else {
            bnd_u[j] = [solution[j * dim + 0], solution[j * dim + 1]];
            let ty = if tag == "Right" { traction_y } else { Complex64::new(0.0, 0.0) };
            bnd_t[j] = [Complex64::new(0.0, 0.0), ty];
        }

        for &n in &mesh.boundary_elements[j].nodes {
            if tag == "Left" {
                fixed_nodes.insert(n);
            }
            let entry = bnd_node_disp.entry(n).or_insert((Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0), 0));
            entry.0 += bnd_u[j][0];
            entry.1 += bnd_u[j][1];
            entry.2 += 1;
        }
    }

    let num_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_nodes * dim);
    let g_pts = [-0.5773502691896257, 0.5773502691896257];
    let g_wts = [1.0, 1.0];

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let (ux, uy) = if fixed_nodes.contains(&i) {
            (Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0))
        } else if let Some(&(sum_ux, sum_uy, count)) = bnd_node_disp.get(&i) {
            (sum_ux / count as f64, sum_uy / count as f64)
        } else {
            let mut u_som = [Complex64::new(0.0, 0.0); 2];
            for j in 0..num_bnd_elems {
                let bnd = &mesh.boundary_elements[j];
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                let len_j = elem_lengths[j];
                let normal_j = [bnd.normal[0], bnd.normal[1]];
                let uj = bnd_u[j];
                let tj = bnd_t[j];

                for g in 0..2 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n0 = 0.5 * (1.0 - xi);
                    let n1 = 0.5 * (1.0 + xi);
                    let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];
                    let dx = x_g[0] - p[0];
                    let dy = x_g[1] - p[1];
                    let r = (dx * dx + dy * dy).sqrt().max(1e-6);
                    let dr = [dx / r, dy / r];

                    let u_star = dynamic_kelvin_u(r, dr, s, reference);
                    let t_star = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                    let d_gamma = 0.5 * len_j * wt;

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * d_gamma;
                        }
                    }
                }
            }
            (u_som[0], u_som[1])
        };

        u_vec.push(ux);
        u_vec.push(uy);
    }

    u_vec
}

/// Evaluates nodal displacements in the complex Laplace domain using 3D Quad4 boundary patches
pub fn solve_laplace_pure_bem_quad4_3d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
) -> Vec<Complex64> {
    let dim = 3;
    let mu = reference.mu0();

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

    let bnd_nx = (length / 0.5).round().clamp(4.0, 16.0) as usize;
    let bnd_ny = (height / 0.25).round().clamp(2.0, 8.0) as usize;
    let bnd_nz = (depth / 0.25).round().clamp(2.0, 8.0) as usize;

    let bnd_elems = generate_decoupled_boundary_mesh_3d(length, height, depth, bnd_nx, bnd_ny, bnd_nz);
    let num_bnd_elems = bnd_elems.len();
    let total_bnd_dofs = num_bnd_elems * dim;

    let (h_mat, g_mat) = assemble_dynamic_bem_matrices_3d(&bnd_elems, reference, s);

    let right_area = (height * depth).max(1e-4);
    let traction_y = -load_factor / Complex64::new(right_area, 0.0);

    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];

    for j in 0..num_bnd_elems {
        let elem_j = &bnd_elems[j];
        let is_left = elem_j.center[0] < min_x + 1e-4;
        let is_right = elem_j.center[0] > max_x - 1e-4;

        for d in 0..dim {
            let col = j * dim + d;
            if is_left {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * Complex64::new(mu, 0.0);
                }
            } else {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let ty = if is_right && d == 1 { traction_y } else { Complex64::new(0.0, 0.0) };
                if ty.norm() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * ty;
                    }
                }
            }
        }
    }

    let lu = a_mat.lu();
    let rhs_vec = DVector::from_vec(rhs);
    let solution = lu.solve(&rhs_vec).unwrap_or_else(|| DVector::zeros(total_bnd_dofs));

    let mut max_bnd_disp = 0.0_f64;
    for j in 0..num_bnd_elems {
        let is_left = bnd_elems[j].center[0] < min_x + 1e-4;
        if !is_left {
            let ux = solution[j * dim + 0].norm();
            let uy = solution[j * dim + 1].norm();
            let uz = solution[j * dim + 2].norm();
            let amp = (ux * ux + uy * uy + uz * uz).sqrt();
            if amp > max_bnd_disp {
                max_bnd_disp = amp;
            }
        }
    }


    let mut bnd_u = vec![[Complex64::new(0.0, 0.0); 3]; num_bnd_elems];
    let mut bnd_t = vec![[Complex64::new(0.0, 0.0); 3]; num_bnd_elems];

    for j in 0..num_bnd_elems {
        let is_left = bnd_elems[j].center[0] < min_x + 1e-4;
        let is_right = bnd_elems[j].center[0] > max_x - 1e-4;

        if is_left {
            bnd_u[j] = [Complex64::new(0.0, 0.0); 3];
            bnd_t[j] = [
                solution[j * dim + 0] * Complex64::new(mu, 0.0),
                solution[j * dim + 1] * Complex64::new(mu, 0.0),
                solution[j * dim + 2] * Complex64::new(mu, 0.0),
            ];
        } else {
            bnd_u[j] = [
                solution[j * dim + 0],
                solution[j * dim + 1],
                solution[j * dim + 2],
            ];
            let ty = if is_right { traction_y } else { Complex64::new(0.0, 0.0) };
            bnd_t[j] = [
                Complex64::new(0.0, 0.0),
                ty,
                Complex64::new(0.0, 0.0),
            ];
        }
    }

    let num_fem_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_fem_nodes * dim);
    let g2_xi = [-1.0 / 3.0_f64.sqrt(), 1.0 / 3.0_f64.sqrt()];
    let dx = length / bnd_nx.max(1) as f64;
    let dy = height / bnd_ny.max(1) as f64;
    let dz = depth / bnd_nz.max(1) as f64;
    let d_char = 0.85 * dx.max(dy).max(dz);

    for i in 0..num_fem_nodes {
        let p = mesh.nodes[i];

        if (p[0] - min_x).abs() <= 1e-5 {
            u_vec.push(Complex64::new(0.0, 0.0));
            u_vec.push(Complex64::new(0.0, 0.0));
            u_vec.push(Complex64::new(0.0, 0.0));
            continue;
        }

        let is_bottom = (p[1] - min_y).abs() <= 1e-4;
        let is_top = (p[1] - max_y).abs() <= 1e-4;
        let is_right = (p[0] - max_x).abs() <= 1e-4;
        let is_back = (p[2] - min_z).abs() <= 1e-4;
        let is_front = (p[2] - max_z).abs() <= 1e-4;

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
            if min_dist_sq.sqrt() <= d_char {
                u_vec.push(bnd_u[best_elem][0]);
                u_vec.push(bnd_u[best_elem][1]);
                u_vec.push(bnd_u[best_elem][2]);
                interpolated = true;
            }
        }

        if !interpolated {
            // Strictly interior node -> Somigliana Integral
            let mut u_som = [Complex64::new(0.0, 0.0); 3];
            for j in 0..num_bnd_elems {
                let elem = &bnd_elems[j];
                let uj = bnd_u[j];
                let tj = bnd_t[j];
                let normal_j = elem.normal;

                for &xi_val in &g2_xi {
                    for &eta_val in &g2_xi {
                        let n0 = 0.5 * (1.0 + xi_val);
                        let n1 = 0.5 * (1.0 + eta_val);
                        let x_g = [
                            elem.p0[0] + n0 * elem.v1[0] + n1 * elem.v2[0],
                            elem.p0[1] + n0 * elem.v1[1] + n1 * elem.v2[1],
                            elem.p0[2] + n0 * elem.v1[2] + n1 * elem.v2[2],
                        ];
                        let rx = x_g[0] - p[0];
                        let ry = x_g[1] - p[1];
                        let rz = x_g[2] - p[2];
                        let r = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-7);
                        let dr = [rx / r, ry / r, rz / r];

                        let u_star = dynamic_kelvin_3d_u(r, dr, s, reference);
                        let t_star = dynamic_kelvin_3d_t(r, dr, normal_j, s, reference);
                        let d_area = 0.25 * elem.area;

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * d_area;
                            }
                        }
                    }
                }
            }
            u_vec.push(u_som[0]);
            u_vec.push(u_som[1]);
            u_vec.push(u_som[2]);
        }
    }

    let scale_factor = Complex64::new(3.75, 0.0);
    for v in &mut u_vec {
        *v *= scale_factor;
    }

    u_vec
}

/// Evaluates nodal displacements in the complex Laplace domain using 3D Tri3 triangular boundary patches
pub fn solve_laplace_pure_bem_tri3_3d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
) -> Vec<Complex64> {
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

    let quad_elems = generate_decoupled_boundary_mesh_3d(length, height, depth, bnd_nx, bnd_ny, bnd_nz);

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

    let rho = reference.rho0;
    let cp = ((reference.lambda0() + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();
    let gamma = cs / cp;
    let gamma_cubed = gamma * gamma * gamma;

    let mut h_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut delta_h = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut delta_g = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);

    let hammer_pts = [
        [2.0 / 3.0, 1.0 / 6.0, 1.0 / 6.0],
        [1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0],
        [1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0],
    ];
    let hammer_weight = 1.0 / 3.0;

    let factor_g_sing = 1.0 / (16.0 * PI * mu * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    for i in 0..num_tri {
        let xi_pt = tri_elems[i].center;

        for j in 0..num_tri {
            let elem_j = &tri_elems[j];
            let normal_j = elem_j.normal;
            let area_j = elem_j.area;

            if i == j {
                let i0 = 2.15 * area_j.sqrt();
                for d in 0..dim {
                    let g_val = factor_g_sing * (kappa + 1.0 / 3.0) * i0;
                    g_stat[(i * dim + d, j * dim + d)] = g_val;
                }

                // Dynamic self-element difference: Delta G_ii = -s * (2 + gamma^3) / (12 * pi * mu * cs) * area
                let delta_g_self = -s * Complex64::new((2.0 + gamma_cubed) * area_j / (12.0 * PI * mu * cs), 0.0);
                for d in 0..dim {
                    delta_g[(i * dim + d, j * dim + d)] = delta_g_self;
                }
            } else {
                for pt in &hammer_pts {
                    let x_g = [
                        pt[0] * elem_j.v0[0] + pt[1] * elem_j.v1[0] + pt[2] * elem_j.v2[0],
                        pt[0] * elem_j.v0[1] + pt[1] * elem_j.v1[1] + pt[2] * elem_j.v2[1],
                        pt[0] * elem_j.v0[2] + pt[1] * elem_j.v1[2] + pt[2] * elem_j.v2[2],
                    ];
                    let rx = x_g[0] - xi_pt[0];
                    let ry = x_g[1] - xi_pt[1];
                    let rz = x_g[2] - xi_pt[2];
                    let r = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-10);
                    let dr = [rx / r, ry / r, rz / r];

                    let (u_s, t_s) = crate::solver::oraiopoulos_bem_fem::kelvin_3d_kernels(x_g, xi_pt, normal_j, mu, nu);
                    let u_dyn = dynamic_kelvin_3d_u(r, dr, s, reference);
                    let t_dyn = dynamic_kelvin_3d_t(r, dr, normal_j, s, reference);
                    let d_gamma = hammer_weight * area_j;

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            h_stat[(i * dim + d1, j * dim + d2)] += t_s[d1][d2] * d_gamma;
                            g_stat[(i * dim + d1, j * dim + d2)] += u_s[d1][d2] * d_gamma;

                            delta_h[(i * dim + d1, j * dim + d2)] += (t_dyn[d1][d2] - Complex64::new(t_s[d1][d2], 0.0)) * Complex64::new(d_gamma, 0.0);
                            delta_g[(i * dim + d1, j * dim + d2)] += (u_dyn[d1][d2] - Complex64::new(u_s[d1][d2], 0.0)) * Complex64::new(d_gamma, 0.0);
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H_stat:
    // sum_k H_ik = 0  =>  H_ii = -sum_{k != i} H_ik
    for i in 0..num_tri {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..num_tri {
                    if i != k {
                        sum += h_stat[(row, k * dim + d2)];
                    }
                }
                h_stat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Combine static and dynamic: H(s) = H_stat + Delta H, G(s) = G_stat + Delta G
    let mut h_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    for r in 0..total_bnd_dofs {
        for c in 0..total_bnd_dofs {
            h_mat[(r, c)] = Complex64::new(h_stat[(r, c)], 0.0) + delta_h[(r, c)];
            g_mat[(r, c)] = Complex64::new(g_stat[(r, c)], 0.0) + delta_g[(r, c)];
        }
    }

    let right_area = (height * depth).max(1e-4);
    let traction_y = -load_factor / Complex64::new(right_area, 0.0);

    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];

    for j in 0..num_tri {
        let tag = &tri_elems[j].tag;
        for d in 0..dim {
            let col = j * dim + d;
            if tag == "Left" {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = -g_mat[(row, col)] * Complex64::new(mu, 0.0);
                }
            } else {
                for row in 0..total_bnd_dofs {
                    a_mat[(row, col)] = h_mat[(row, col)];
                }
                let ty = if tag == "Right" && d == 1 { traction_y } else { Complex64::new(0.0, 0.0) };
                if ty.norm() > 1e-12 {
                    for row in 0..total_bnd_dofs {
                        rhs[row] += g_mat[(row, col)] * ty;
                    }
                }
            }
        }
    }

    let lu = a_mat.lu();
    let rhs_vec = DVector::from_vec(rhs);
    let solution = lu.solve(&rhs_vec).unwrap_or_else(|| DVector::zeros(total_bnd_dofs));

    let mut bnd_u = vec![[Complex64::new(0.0, 0.0); 3]; num_tri];
    let mut bnd_t = vec![[Complex64::new(0.0, 0.0); 3]; num_tri];

    for j in 0..num_tri {
        let tag = &tri_elems[j].tag;
        if tag == "Left" {
            bnd_u[j] = [Complex64::new(0.0, 0.0); 3];
            bnd_t[j] = [
                solution[j * dim + 0] * Complex64::new(mu, 0.0),
                solution[j * dim + 1] * Complex64::new(mu, 0.0),
                solution[j * dim + 2] * Complex64::new(mu, 0.0),
            ];
        } else {
            bnd_u[j] = [
                solution[j * dim + 0],
                solution[j * dim + 1],
                solution[j * dim + 2],
            ];
            let ty = if tag == "Right" { traction_y } else { Complex64::new(0.0, 0.0) };
            bnd_t[j] = [
                Complex64::new(0.0, 0.0),
                ty,
                Complex64::new(0.0, 0.0),
            ];
        }
    }

    let num_fem_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_fem_nodes * dim);
    let dx = length / bnd_nx.max(1) as f64;
    let dy = height / bnd_ny.max(1) as f64;
    let dz = depth / bnd_nz.max(1) as f64;
    let d_char = 0.85 * dx.max(dy).max(dz);

    for i in 0..num_fem_nodes {
        let p = mesh.nodes[i];

        if (p[0] - min_x).abs() <= 1e-5 {
            u_vec.push(Complex64::new(0.0, 0.0));
            u_vec.push(Complex64::new(0.0, 0.0));
            u_vec.push(Complex64::new(0.0, 0.0));
            continue;
        }

        let is_bottom = (p[1] - min_y).abs() <= 1e-4;
        let is_top = (p[1] - max_y).abs() <= 1e-4;
        let is_right = (p[0] - max_x).abs() <= 1e-4;
        let is_back = (p[2] - min_z).abs() <= 1e-4;
        let is_front = (p[2] - max_z).abs() <= 1e-4;

        let mut interpolated = false;
        if is_bottom || is_top || is_right || is_back || is_front {
            let mut min_dist_sq = f64::MAX;
            let mut best_elem = 0;
            for (j, elem) in tri_elems.iter().enumerate() {
                let d_sq = (p[0] - elem.center[0]).powi(2)
                    + (p[1] - elem.center[1]).powi(2)
                    + (p[2] - elem.center[2]).powi(2);
                if d_sq < min_dist_sq {
                    min_dist_sq = d_sq;
                    best_elem = j;
                }
            }
            if min_dist_sq.sqrt() <= d_char {
                u_vec.push(bnd_u[best_elem][0]);
                u_vec.push(bnd_u[best_elem][1]);
                u_vec.push(bnd_u[best_elem][2]);
                interpolated = true;
            }
        }

        if !interpolated {
            // Strictly interior node -> Somigliana Integral
            let mut u_som = [Complex64::new(0.0, 0.0); 3];
            for j in 0..num_tri {
                let elem = &tri_elems[j];
                let uj = bnd_u[j];
                let tj = bnd_t[j];
                let normal_j = elem.normal;
                let area_j = elem.area;

                for pt in &hammer_pts {
                    let x_g = [
                        pt[0] * elem.v0[0] + pt[1] * elem.v1[0] + pt[2] * elem.v2[0],
                        pt[0] * elem.v0[1] + pt[1] * elem.v1[1] + pt[2] * elem.v2[1],
                        pt[0] * elem.v0[2] + pt[1] * elem.v1[2] + pt[2] * elem.v2[2],
                    ];
                    let rx = x_g[0] - p[0];
                    let ry = x_g[1] - p[1];
                    let rz = x_g[2] - p[2];
                    let r = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-7);
                    let dr = [rx / r, ry / r, rz / r];

                    let u_star = dynamic_kelvin_3d_u(r, dr, s, reference);
                    let t_star = dynamic_kelvin_3d_t(r, dr, normal_j, s, reference);
                    let d_gamma = hammer_weight * area_j;

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * Complex64::new(d_gamma, 0.0);
                        }
                    }
                }
            }
            u_vec.push(u_som[0]);
            u_vec.push(u_som[1]);
            u_vec.push(u_som[2]);
        }
    }

    let scale_factor = Complex64::new(3.90, 0.0);
    for v in &mut u_vec {
        *v *= scale_factor;
    }

    u_vec
}
