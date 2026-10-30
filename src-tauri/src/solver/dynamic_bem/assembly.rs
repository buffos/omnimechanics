use std::f64::consts::PI;
use super::kernels::*;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::oraiopoulos_bem_fem::DecoupledBndElem3D;
use nalgebra::DMatrix;
use num_complex::Complex64;

/// Assembles complex-valued dynamic BEM system matrices H(s) and G(s)
/// for a 2D mesh at Laplace parameter s.
pub fn assemble_dynamic_bem_matrices(
    mesh: &MeshData,
    reference: &ReferenceMaterial,
    s: Complex64,
) -> (DMatrix<Complex64>, DMatrix<Complex64>) {
    let dim = 2;
    let num_bnd = mesh.boundary_elements.len();
    let total_dofs = num_bnd * dim;
    let mu = reference.mu0();
    let nu = reference.nu0;

    let mut h_mat = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);
    let mut g_mat = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);

    let elem_centers: Vec<[f64; 2]> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            [(p0[0] + p1[0]) * 0.5, (p0[1] + p1[1]) * 0.5]
        })
        .collect();

    let elem_lengths: Vec<f64> = mesh
        .boundary_elements
        .iter()
        .map(|bnd| {
            let p0 = mesh.nodes[bnd.nodes[0]];
            let p1 = mesh.nodes[bnd.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-5)
        })
        .collect();

    let g_pts = [-0.5773502691896257, 0.5773502691896257];
    let g_wts = [1.0, 1.0];

    for i in 0..num_bnd {
        let p_i = elem_centers[i];

        for j in 0..num_bnd {
            let len_j = elem_lengths[j];
            let normal_j = [mesh.boundary_elements[j].normal[0], mesh.boundary_elements[j].normal[1]];

            if i == j {
                // Free term C_i = 0.5 * I for smooth boundary (integral of T* is 0 since r . n = 0 on flat element)
                for d in 0..dim {
                    h_mat[(i * dim + d, j * dim + d)] = Complex64::new(0.5, 0.0);
                    // Exact analytical integration of singular logarithmic G asymptote on straight line element
                    let tau = [-normal_j[1], normal_j[0]];
                    let int_ln = len_j * (1.0 - (0.5 * len_j).ln());
                    let factor = 1.0 / (8.0 * PI * mu * (1.0 - nu));
                    let delta = 1.0;
                    g_mat[(i * dim + d, j * dim + d)] = Complex64::new(
                        factor * ((3.0 - 4.0 * nu) * int_ln * delta + len_j * tau[d] * tau[d]),
                        0.0,
                    );
                }
            } else {
                let p0 = mesh.nodes[mesh.boundary_elements[j].nodes[0]];
                let p1 = mesh.nodes[mesh.boundary_elements[j].nodes[1]];

                for g in 0..2 {
                    let xi = g_pts[g];
                    let wt = g_wts[g];
                    let n0 = 0.5 * (1.0 - xi);
                    let n1 = 0.5 * (1.0 + xi);
                    let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];
                    let dx = x_g[0] - p_i[0];
                    let dy = x_g[1] - p_i[1];
                    let r = (dx * dx + dy * dy).sqrt().max(1e-6);
                    let dr = [dx / r, dy / r];

                    let u_star = dynamic_kelvin_u(r, dr, s, reference);
                    let t_star = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                    let d_gamma = 0.5 * len_j * wt;

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

    (h_mat, g_mat)
}


/// Assembles complex-valued dynamic BEM system matrices H(s) and G(s)
pub fn assemble_dynamic_bem_matrices_3d(
    bnd_elems: &[DecoupledBndElem3D],
    reference: &ReferenceMaterial,
    s: Complex64,
) -> (DMatrix<Complex64>, DMatrix<Complex64>) {
    let dim = 3;
    let num_bnd = bnd_elems.len();
    let total_dofs = num_bnd * dim;
    let mu = reference.mu0();
    let nu = reference.nu0;
    let rho = reference.rho0;
    let lambda = reference.lambda0();
    let cp = ((lambda + 2.0 * mu) / rho).sqrt();
    let cs = (mu / rho).sqrt();
    let gamma = cs / cp;
    let gamma_cubed = gamma * gamma * gamma;

    let mut h_stat = DMatrix::<f64>::zeros(total_dofs, total_dofs);
    let mut g_stat = DMatrix::<f64>::zeros(total_dofs, total_dofs);
    let mut delta_h = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);
    let mut delta_g = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);

    let g2_xi = [-1.0 / 3.0_f64.sqrt(), 1.0 / 3.0_f64.sqrt()];
    let factor_g_sing = 1.0 / (16.0 * PI * mu * (1.0 - nu));

    for i in 0..num_bnd {
        let xi_pt = bnd_elems[i].center;

        for j in 0..num_bnd {
            let elem_j = &bnd_elems[j];
            let normal_j = elem_j.normal;
            let area_j = elem_j.area;

            if i == j {
                // Diagonal singular integration: Flat rectangular patch analytical singular integral
                let len_v1 = (elem_j.v1[0].powi(2) + elem_j.v1[1].powi(2) + elem_j.v1[2].powi(2)).sqrt().max(1e-6);
                let len_v2 = (elem_j.v2[0].powi(2) + elem_j.v2[1].powi(2) + elem_j.v2[2].powi(2)).sqrt().max(1e-6);
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
                        g_stat[(i * dim + d1, j * dim + d2)] += g_val;
                    }
                }

                // Dynamic self-element difference: Delta G_ii = -s * (2 + gamma^3) / (12 * pi * mu * cs) * area
                let delta_g_self = -s * Complex64::new((2.0 + gamma_cubed) * area_j / (12.0 * PI * mu * cs), 0.0);
                for d in 0..dim {
                    delta_g[(i * dim + d, j * dim + d)] = delta_g_self;
                }
            } else {
                // 2x2 Gauss Quadrature for regular 3D surface element
                for &xi_val in &g2_xi {
                    for &eta_val in &g2_xi {
                        let n0 = 0.5 * (1.0 + xi_val);
                        let n1 = 0.5 * (1.0 + eta_val);
                        let x_g = [
                            elem_j.p0[0] + n0 * elem_j.v1[0] + n1 * elem_j.v2[0],
                            elem_j.p0[1] + n0 * elem_j.v1[1] + n1 * elem_j.v2[1],
                            elem_j.p0[2] + n0 * elem_j.v1[2] + n1 * elem_j.v2[2],
                        ];
                        let rx = x_g[0] - xi_pt[0];
                        let ry = x_g[1] - xi_pt[1];
                        let rz = x_g[2] - xi_pt[2];
                        let r = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-7);
                        let dr = [rx / r, ry / r, rz / r];

                        let (u_stat, t_stat) = crate::solver::oraiopoulos_bem_fem::kelvin_3d_kernels(x_g, xi_pt, normal_j, mu, nu);
                        let u_dyn = dynamic_kelvin_3d_u(r, dr, s, reference);
                        let t_dyn = dynamic_kelvin_3d_t(r, dr, normal_j, s, reference);
                        let d_area = 0.25 * area_j;

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_stat[(i * dim + d1, j * dim + d2)] += t_stat[d1][d2] * d_area;
                                g_stat[(i * dim + d1, j * dim + d2)] += u_stat[d1][d2] * d_area;

                                delta_h[(i * dim + d1, j * dim + d2)] += (t_dyn[d1][d2] - Complex64::new(t_stat[d1][d2], 0.0)) * d_area;
                                delta_g[(i * dim + d1, j * dim + d2)] += (u_dyn[d1][d2] - Complex64::new(u_stat[d1][d2], 0.0)) * d_area;
                            }
                        }
                    }
                }
            }
        }
    }

    // Rigid body translation condition for diagonal blocks of H_stat:
    // sum_k H_ik = 0  =>  H_ii = -sum_{k != i} H_ik
    for i in 0..num_bnd {
        for d1 in 0..dim {
            let row = i * dim + d1;
            for d2 in 0..dim {
                let mut sum = 0.0;
                for k in 0..num_bnd {
                    if i != k {
                        sum += h_stat[(row, k * dim + d2)];
                    }
                }
                h_stat[(row, i * dim + d2)] = -sum;
            }
        }
    }

    // Combine static and dynamic: H(s) = H_stat + Delta H, G(s) = G_stat + Delta G
    let mut h_mat = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);
    let mut g_mat = DMatrix::<Complex64>::zeros(total_dofs, total_dofs);
    for r in 0..total_dofs {
        for c in 0..total_dofs {
            h_mat[(r, c)] = Complex64::new(h_stat[(r, c)], 0.0) + delta_h[(r, c)];
            g_mat[(r, c)] = Complex64::new(g_stat[(r, c)], 0.0) + delta_g[(r, c)];
        }
    }

    (h_mat, g_mat)
}
