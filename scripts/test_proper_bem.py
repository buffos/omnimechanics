with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::bem_solver::integrate_poly_ln;
        use num_complex::Complex64;
        use nalgebra::{DMatrix, DVector};
        use std::f64::consts::PI;

        let mesh = generate_beam_2d(4.0, 1.0, 4, 1, false);
        let ref_mat = ReferenceMaterial::default();
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let s = Complex64::new(0.02 * 2.0 * PI * 3.0, 2.0 * PI * 3.0);
        let load_factor = Complex64::new(50_000.0, 0.0);

        let dim = 2;
        let num_bnd_elems = mesh.boundary_elements.len();
        let num_elem_nodes = 3;
        let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
        let total_bnd_dofs = total_bnd_nodes * dim;

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

        let mut h_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
        let mut g_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);

        let shape_coeffs = [
            [0.0, -0.75, 1.125],
            [1.0, 0.0, -2.25],
            [0.0, 0.75, 1.125],
        ];
        let shape_integrals = [0.75, 0.5, 0.75];

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
                        let int_poly_ln = integrate_poly_ln(c[0], c[1], c[2], xi_0);
                        let int_ln_r = 0.5 * len_j * (shape_integrals[alpha] * (2.0 / len_j).ln() - int_poly_ln);
                        let int_tau = 0.5 * len_j * shape_integrals[alpha];

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                let delta = if d1 == d2 { 1.0 } else { 0.0 };
                                let g_val = factor_g_sing * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                                g_mat[(i * dim + d1, col_node * dim + d2)] += Complex64::new(g_val, 0.0);
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
                                    h_mat[(i * dim + d1, col_node * dim + d2)] += Complex64::new(h_val, 0.0);
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

                        let rx = x_g[0] - p_i[0];
                        let ry = x_g[1] - p_i[1];
                        let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                        let dr = [rx / r, ry / r];

                        let u_star = crate::solver::dynamic_bem::dynamic_kelvin_u(r, dr, s, &ref_mat);
                        let t_star = crate::solver::dynamic_bem::dynamic_kelvin_t(r, dr, normal_j, s, &ref_mat);
                        let d_gamma = 0.5 * len_j * w_g;

                        let shapes = [
                            shape_coeffs[0][0] + shape_coeffs[0][1] * xi_g + shape_coeffs[0][2] * xi_g * xi_g,
                            shape_coeffs[1][0] + shape_coeffs[1][1] * xi_g + shape_coeffs[1][2] * xi_g * xi_g,
                            shape_coeffs[2][0] + shape_coeffs[2][1] * xi_g + shape_coeffs[2][2] * xi_g * xi_g,
                        ];

                        for alpha in 0..num_elem_nodes {
                            let col_node = j * num_elem_nodes + alpha;
                            let n_alpha = shapes[alpha];

                            for d1 in 0..dim {
                                for d2 in 0..dim {
                                    h_mat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * Complex64::new(n_alpha * d_gamma, 0.0);
                                    g_mat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * Complex64::new(n_alpha * d_gamma, 0.0);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply Rigid Body Translation condition to H:
        for i in 0..total_bnd_nodes {
            for d1 in 0..dim {
                let row = i * dim + d1;
                for d2 in 0..dim {
                    let mut sum = Complex64::new(0.0, 0.0);
                    for k in 0..total_bnd_nodes {
                        if i != k {
                            sum += h_mat[(row, k * dim + d2)];
                        }
                    }
                    h_mat[(row, i * dim + d2)] = -sum;
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
        for j in 0..total_bnd_nodes {
            let tag = &bnd_node_tags[j];
            if tag == "Left" {
                bnd_u[j] = [Complex64::new(0.0, 0.0); 2];
            } else {
                bnd_u[j] = [solution[j * dim + 0], solution[j * dim + 1]];
            }
        }

        println!("--- PROPER REGULARIZED BEM TEST ---");
        // Check bnd_u at Right boundary nodes:
        for j in 0..total_bnd_nodes {
            if bnd_node_tags[j] == "Right" {
                println!("Right boundary node {}: coord={:?}, u=({:?}, {:?})", 
                    j, bnd_node_coords[j], bnd_u[j][0], bnd_u[j][1]);
            }
        }
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Test written")
