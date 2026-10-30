use std::f64::consts::PI;
use super::kernels::*;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::true_bem_fem::kelvin_2d_kernels;
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use std::collections::HashSet;

/// Evaluates nodal displacements in the complex Laplace domain using 2D Linear boundary elements
pub fn solve_laplace_pure_bem_linear_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
) -> Vec<Complex64> {
    let dim = 2;
    let num_bnd_elems = mesh.boundary_elements.len();
    let num_elem_nodes = 2;
    let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
    let total_bnd_dofs = total_bnd_nodes * dim;
    let mu = reference.mu0();
    let nu = reference.nu0;

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

    let xi_nodes = [-1.0 / 3.0_f64.sqrt(), 1.0 / 3.0_f64.sqrt()];
    let mut bnd_node_coords = Vec::with_capacity(total_bnd_nodes);
    let mut bnd_node_tags = Vec::with_capacity(total_bnd_nodes);

    for j in 0..num_bnd_elems {
        let bnd = &mesh.boundary_elements[j];
        let p0 = mesh.nodes[bnd.nodes[0]];
        let p1 = mesh.nodes[bnd.nodes[1]];

        for &xi in &xi_nodes {
            let n0 = 0.5 * (1.0 - xi);
            let n1 = 0.5 * (1.0 + xi);
            bnd_node_coords.push([n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]]);
            bnd_node_tags.push(bnd.tag.clone());
        }
    }

    let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
    let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

    let factor_g_sing = 1.0 / (8.0 * PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    let shape_coeffs = [
        [0.5, -0.5 * 3.0_f64.sqrt(), 0.0],
        [0.5, 0.5 * 3.0_f64.sqrt(), 0.0],
    ];

    let mut h_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

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
                let xi_0 = xi_nodes[local_i];
                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let c = shape_coeffs[alpha];
                    let int_poly_ln = crate::solver::bem_solver::integrate_poly_ln(c[0], c[1], c[2], xi_0);
                    let int_ln_r = 0.5 * len_j * ((2.0 / len_j).ln() - int_poly_ln);
                    let int_tau = 0.5 * len_j;

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g_sing * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_stat[(i * dim + d1, col_node * dim + d2)] += g_val;
                        }
                    }

                    if alpha != local_i {
                        let int_cauchy = if local_i == 0 && alpha == 1 { 3.0_f64.sqrt() } else { -3.0_f64.sqrt() };
                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                let h_val = factor_h_sing * (tau[d1] * normal_j[d2] - tau[d2] * normal_j[d1]) * int_cauchy;
                                h_stat[(i * dim + d1, col_node * dim + d2)] += h_val;
                            }
                        }
                    }
                }
            } else {
                for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                    let w_g = g_wts[g_idx];
                    let n0_geom = 0.5 * (1.0 - xi_g);
                    let n1_geom = 0.5 * (1.0 + xi_g);
                    let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];
                    let d_gamma = 0.5 * len_j * w_g;

                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                    let n0 = 0.5 * (1.0 - 3.0_f64.sqrt() * xi_g);
                    let n1 = 0.5 * (1.0 + 3.0_f64.sqrt() * xi_g);
                    let shapes = [n0, n1];

                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let na = shapes[alpha];

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_stat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * na * d_gamma;
                                g_stat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * na * d_gamma;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H_stat
    for i in 0..total_bnd_nodes {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..total_bnd_nodes {
                    if i != k {
                        sum += h_stat[(row, k * dim + d2)];
                    }
                }
                h_stat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    let mut h_mat = h_stat.map(|v| Complex64::new(v, 0.0));
    let mut g_mat = g_stat.map(|v| Complex64::new(v, 0.0));

    // Add regularized difference between dynamic and static kernels
    for i in 0..total_bnd_nodes {
        let p_i = bnd_node_coords[i];
        let elem_i = i / num_elem_nodes;

        for j in 0..num_bnd_elems {
            let bnd = &mesh.boundary_elements[j];
            let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
            let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
            let len_j = elem_lengths[j];
            let normal_j = [bnd.normal[0], bnd.normal[1]];

            for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                let w_g = g_wts[g_idx];
                let n0_geom = 0.5 * (1.0 - xi_g);
                let n1_geom = 0.5 * (1.0 + xi_g);
                let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];
                let d_gamma = 0.5 * len_j * w_g;

                let rx = x_g[0] - p_i[0];
                let ry = x_g[1] - p_i[1];
                let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                let dr = [rx / r, ry / r];

                let u_star_d = dynamic_kelvin_u(r, dr, s, reference);
                let t_star_d = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                let (u_star_s, t_star_s) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                let n0 = 0.5 * (1.0 - 3.0_f64.sqrt() * xi_g);
                let n1 = 0.5 * (1.0 + 3.0_f64.sqrt() * xi_g);
                let shapes = [n0, n1];

                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let na = shapes[alpha];

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let u_diff = u_star_d[d1][d2] - Complex64::new(u_star_s[d1][d2], 0.0);
                            let t_diff = t_star_d[d1][d2] - Complex64::new(t_star_s[d1][d2], 0.0);

                            g_mat[(i * dim + d1, col_node * dim + d2)] += u_diff * Complex64::new(na * d_gamma, 0.0);
                            if elem_i != j {
                                h_mat[(i * dim + d1, col_node * dim + d2)] += t_diff * Complex64::new(na * d_gamma, 0.0);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];

    for j in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[j];
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

    let mut bnd_u = vec![[Complex64::new(0.0, 0.0); 2]; total_bnd_nodes];
    let mut bnd_t = vec![[Complex64::new(0.0, 0.0); 2]; total_bnd_nodes];

    for j in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[j];
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
    }

    let num_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_nodes * dim);

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let is_left_node = p[0] <= 1e-5;

        let (ux, uy) = if is_left_node {
            (Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0))
        } else {
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
                if perp.abs() < 1e-3 && proj >= -1e-3 && proj <= len + 1e-3 {
                    let xi = (2.0 * proj / len - 1.0).max(-1.0).min(1.0);
                    on_bnd = Some((j, xi));
                    break;
                }
            }

            if let Some((j, xi)) = on_bnd {
                let n0 = 0.5 * (1.0 - 3.0_f64.sqrt() * xi);
                let n1 = 0.5 * (1.0 + 3.0_f64.sqrt() * xi);
                let u0 = bnd_u[j * 2];
                let u1 = bnd_u[j * 2 + 1];
                let bnd_ux = Complex64::new(n0, 0.0) * u0[0] + Complex64::new(n1, 0.0) * u1[0];
                let bnd_uy = Complex64::new(n0, 0.0) * u0[1] + Complex64::new(n1, 0.0) * u1[1];
                (bnd_ux, bnd_uy)
            } else {
                let mut u_som = [Complex64::new(0.0, 0.0); 2];
                for j in 0..num_bnd_elems {
                    let bnd = &mesh.boundary_elements[j];
                    let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
                    let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
                    let len_j = elem_lengths[j];
                    let normal_j = [bnd.normal[0], bnd.normal[1]];

                    for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                        let w_g = g_wts[g_idx];
                        let n0_geom = 0.5 * (1.0 - xi_g);
                        let n1_geom = 0.5 * (1.0 + xi_g);
                        let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];
                        let d_gamma = 0.5 * len_j * w_g;

                        let rx = x_g[0] - p[0];
                        let ry = x_g[1] - p[1];
                        let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                        let dr = [rx / r, ry / r];

                        let u_star = dynamic_kelvin_u(r, dr, s, reference);
                        let t_star = dynamic_kelvin_t(r, dr, normal_j, s, reference);

                        let n0 = 0.5 * (1.0 - 3.0_f64.sqrt() * xi_g);
                        let n1 = 0.5 * (1.0 + 3.0_f64.sqrt() * xi_g);
                        let shapes = [n0, n1];
                        let uj = [
                            shapes[0] * bnd_u[j * 2][0] + shapes[1] * bnd_u[j * 2 + 1][0],
                            shapes[0] * bnd_u[j * 2][1] + shapes[1] * bnd_u[j * 2 + 1][1],
                        ];
                        let tj = [
                            shapes[0] * bnd_t[j * 2][0] + shapes[1] * bnd_t[j * 2 + 1][0],
                            shapes[0] * bnd_t[j * 2][1] + shapes[1] * bnd_t[j * 2 + 1][1],
                        ];

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * Complex64::new(d_gamma, 0.0);
                            }
                        }
                    }
                }
                (u_som[0], u_som[1])
            }
        };

        u_vec.push(ux);
        u_vec.push(uy);
    }

    u_vec
}


/// Evaluates nodal displacements in the complex Laplace domain using 2D Quadratic boundary elements
pub fn solve_laplace_pure_bem_quadratic_2d(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
    load_factor: Complex64,
) -> Vec<Complex64> {
    let dim = 2;
    let num_bnd_elems = mesh.boundary_elements.len();
    let num_elem_nodes = 3;
    let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
    let total_bnd_dofs = total_bnd_nodes * dim;
    let mu = reference.mu0();
    let nu = reference.nu0;

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

    let xi_nodes = [-2.0 / 3.0, 0.0, 2.0 / 3.0];
    let mut bnd_node_coords = Vec::with_capacity(total_bnd_nodes);
    let mut bnd_node_tags = Vec::with_capacity(total_bnd_nodes);

    for j in 0..num_bnd_elems {
        let bnd = &mesh.boundary_elements[j];
        let p0 = mesh.nodes[bnd.nodes[0]];
        let p1 = mesh.nodes[bnd.nodes[1]];

        for &xi in &xi_nodes {
            let n0 = 0.5 * (1.0 - xi);
            let n1 = 0.5 * (1.0 + xi);
            bnd_node_coords.push([n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]]);
            bnd_node_tags.push(bnd.tag.clone());
        }
    }

    let g_pts = [
        -0.9324695142031521, -0.6612093864662645, -0.2386191860831969,
        0.2386191860831969, 0.6612093864662645, 0.9324695142031521,
    ];
    let g_wts = [
        0.1713244923791704, 0.3607615730481386, 0.4679139345726910,
        0.4679139345726910, 0.3607615730481386, 0.1713244923791704,
    ];

    let factor_g_sing = 1.0 / (8.0 * PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;

    let shape_coeffs = [
        [0.0, -0.75, 1.125],
        [1.0, 0.0, -2.25],
        [0.0, 0.75, 1.125],
    ];
    let shape_integrals = [0.75, 0.5, 0.75];

    let mut h_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut g_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

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
                let xi_0 = xi_nodes[local_i];
                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let c = shape_coeffs[alpha];
                    let int_poly_ln = crate::solver::bem_solver::integrate_poly_ln(c[0], c[1], c[2], xi_0);
                    let int_ln_r = 0.5 * len_j * (shape_integrals[alpha] * (2.0 / len_j).ln() - int_poly_ln);
                    let int_tau = 0.5 * len_j * shape_integrals[alpha];

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g_sing * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_stat[(i * dim + d1, col_node * dim + d2)] += g_val;
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
                                h_stat[(i * dim + d1, col_node * dim + d2)] += h_val;
                            }
                        }
                    }
                }
            } else {
                for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                    let w_g = g_wts[g_idx];
                    let n0_geom = 0.5 * (1.0 - xi_g);
                    let n1_geom = 0.5 * (1.0 + xi_g);
                    let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];
                    let d_gamma = 0.5 * len_j * w_g;
                    let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                    let shapes = [
                        shape_coeffs[0][0] + shape_coeffs[0][1] * xi_g + shape_coeffs[0][2] * xi_g * xi_g,
                        shape_coeffs[1][0] + shape_coeffs[1][1] * xi_g + shape_coeffs[1][2] * xi_g * xi_g,
                        shape_coeffs[2][0] + shape_coeffs[2][1] * xi_g + shape_coeffs[2][2] * xi_g * xi_g,
                    ];

                    for alpha in 0..num_elem_nodes {
                        let col_node = j * num_elem_nodes + alpha;
                        let na = shapes[alpha];
                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_stat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * na * d_gamma;
                                g_stat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * na * d_gamma;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H_stat
    for i in 0..total_bnd_nodes {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..total_bnd_nodes {
                    if i != k {
                        sum += h_stat[(row, k * dim + d2)];
                    }
                }
                h_stat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    let mut h_mat = h_stat.map(|v| Complex64::new(v, 0.0));
    let mut g_mat = g_stat.map(|v| Complex64::new(v, 0.0));

    // Add regularized difference between dynamic and static kernels
    for i in 0..total_bnd_nodes {
        let p_i = bnd_node_coords[i];
        let elem_i = i / num_elem_nodes;

        for j in 0..num_bnd_elems {
            let bnd = &mesh.boundary_elements[j];
            let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
            let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
            let len_j = elem_lengths[j];
            let normal_j = [bnd.normal[0], bnd.normal[1]];

            for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                let w_g = g_wts[g_idx];
                let n0_geom = 0.5 * (1.0 - xi_g);
                let n1_geom = 0.5 * (1.0 + xi_g);
                let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];
                let d_gamma = 0.5 * len_j * w_g;

                let rx = x_g[0] - p_i[0];
                let ry = x_g[1] - p_i[1];
                let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                let dr = [rx / r, ry / r];

                let u_star_d = dynamic_kelvin_u(r, dr, s, reference);
                let t_star_d = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                let (u_star_s, t_star_s) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

                let shapes = [
                    shape_coeffs[0][0] + shape_coeffs[0][1] * xi_g + shape_coeffs[0][2] * xi_g * xi_g,
                    shape_coeffs[1][0] + shape_coeffs[1][1] * xi_g + shape_coeffs[1][2] * xi_g * xi_g,
                    shape_coeffs[2][0] + shape_coeffs[2][1] * xi_g + shape_coeffs[2][2] * xi_g * xi_g,
                ];

                for alpha in 0..num_elem_nodes {
                    let col_node = j * num_elem_nodes + alpha;
                    let na = shapes[alpha];

                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let u_diff = u_star_d[d1][d2] - Complex64::new(u_star_s[d1][d2], 0.0);
                            let t_diff = t_star_d[d1][d2] - Complex64::new(t_star_s[d1][d2], 0.0);

                            g_mat[(i * dim + d1, col_node * dim + d2)] += u_diff * Complex64::new(na * d_gamma, 0.0);
                            if elem_i != j {
                                h_mat[(i * dim + d1, col_node * dim + d2)] += t_diff * Complex64::new(na * d_gamma, 0.0);
                            }
                        }
                    }
                }
            }
        }
    }
    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];

    for j in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[j];
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

    let mut bnd_u = vec![[Complex64::new(0.0, 0.0); 2]; total_bnd_nodes];
    let mut bnd_t = vec![[Complex64::new(0.0, 0.0); 2]; total_bnd_nodes];

    for j in 0..total_bnd_nodes {
        let tag = &bnd_node_tags[j];
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
    }

    let num_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_nodes * dim);

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let is_left_node = p[0] <= 1e-5;

        let (ux, uy) = if is_left_node {
            (Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0))
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
                let mut bnd_ux = Complex64::new(0.0, 0.0);
                let mut bnd_uy = Complex64::new(0.0, 0.0);
                for alpha in 0..3 {
                    let node_idx = j * 3 + alpha;
                    bnd_ux += Complex64::new(n_vals[alpha], 0.0) * bnd_u[node_idx][0];
                    bnd_uy += Complex64::new(n_vals[alpha], 0.0) * bnd_u[node_idx][1];
                }
                (bnd_ux, bnd_uy)
            } else {
                // Interior volume node: evaluate Somigliana representation
                let mut u_som = [Complex64::new(0.0, 0.0); 2];
                for j in 0..num_bnd_elems {
                    let bnd = &mesh.boundary_elements[j];
                    let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
                    let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
                    let len_j = elem_lengths[j];
                    let normal_j = [bnd.normal[0], bnd.normal[1]];

                    for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                        let w_g = g_wts[g_idx];
                        let n0_geom = 0.5 * (1.0 - xi_g);
                        let n1_geom = 0.5 * (1.0 + xi_g);
                        let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];

                        let rx = x_g[0] - p[0];
                        let ry = x_g[1] - p[1];
                        let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                        let dr = [rx / r, ry / r];

                        let u_star = dynamic_kelvin_u(r, dr, s, reference);
                        let t_star = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                        let d_gamma = 0.5 * len_j * w_g;

                        let shapes = [
                            shape_coeffs[0][0] + shape_coeffs[0][1] * xi_g + shape_coeffs[0][2] * xi_g * xi_g,
                            shape_coeffs[1][0] + shape_coeffs[1][1] * xi_g + shape_coeffs[1][2] * xi_g * xi_g,
                            shape_coeffs[2][0] + shape_coeffs[2][1] * xi_g + shape_coeffs[2][2] * xi_g * xi_g,
                        ];

                        let uj = [
                            shapes[0] * bnd_u[j * 3][0] + shapes[1] * bnd_u[j * 3 + 1][0] + shapes[2] * bnd_u[j * 3 + 2][0],
                            shapes[0] * bnd_u[j * 3][1] + shapes[1] * bnd_u[j * 3 + 1][1] + shapes[2] * bnd_u[j * 3 + 2][1],
                        ];
                        let tj = [
                            shapes[0] * bnd_t[j * 3][0] + shapes[1] * bnd_t[j * 3 + 1][0] + shapes[2] * bnd_t[j * 3 + 2][0],
                            shapes[0] * bnd_t[j * 3][1] + shapes[1] * bnd_t[j * 3 + 1][1] + shapes[2] * bnd_t[j * 3 + 2][1],
                        ];

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * Complex64::new(d_gamma, 0.0);
                            }
                        }
                    }
                }
                (u_som[0], u_som[1])
            }
        };

        u_vec.push(ux);
        u_vec.push(uy);
    }
    u_vec
}
