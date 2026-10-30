use crate::material::strategies::*;
use crate::material::types::*;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::solver::*;
use faer::Mat;
use num_complex::Complex64;

    #[test]
    fn test_all_transient_elements_talbot_and_newmark() {
        use crate::material::types::{Axis, GradientForm, MaterialStrategyConfig, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::types::{BemElementType, TransientSimulationRequest};
        use crate::solver::run_transient_simulation;

        let ref_mat = ReferenceMaterial::default();
        let strategy_cfg = MaterialStrategyConfig::DirectionalGradient {
            axis: Axis::X,
            alpha: 0.0,
            length_scale: 4.0,
            power: 1.0,
            form: GradientForm::Linear,
        };

        let mesh_2d_tri = generate_beam_2d(3.0, 1.0, 4, 2, true);
        let mesh_2d_quad = generate_beam_2d(3.0, 1.0, 4, 2, false);
        let mesh_3d_tet = generate_beam_3d(2.0, 1.0, 1.0, 2, 2, 2, true);
        let mesh_3d_hex = generate_beam_3d(2.0, 1.0, 1.0, 2, 2, 2, false);

        // 1. Pure FEM Transient: Newmark & Talbot across all 4 element types
        let fem_configs = [
            ("2D Tri3", &mesh_2d_tri),
            ("2D Quad4", &mesh_2d_quad),
            ("3D Tet4", &mesh_3d_tet),
            ("3D Hex8", &mesh_3d_hex),
        ];

        for (name, mesh) in &fem_configs {
            // Newmark
            let res_newmark = run_transient_simulation(TransientSimulationRequest {
                mesh: (*mesh).clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                total_time: 0.2,
                time_steps: 4,
                num_talbot_points: 8,
                load_magnitude: 50_000.0,
                pulse_duration: 0.05,
                excitation_type: Some("Pulse".into()),
                excitation_frequency: None,
                integration_method: Some("Newmark".into()),
                hht_alpha: None,
                rayleigh_alpha: None,
                rayleigh_beta: None,
                solver_mode: Some("PureFEM".into()),
                bem_element_type: None,
            });
            assert!(res_newmark.max_overall_displacement > 0.0, "Newmark failed for {}", name);

            // Talbot FEM
            let res_talbot = run_transient_simulation(TransientSimulationRequest {
                mesh: (*mesh).clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                total_time: 0.2,
                time_steps: 4,
                num_talbot_points: 8,
                load_magnitude: 50_000.0,
                pulse_duration: 0.05,
                excitation_type: Some("Pulse".into()),
                excitation_frequency: None,
                integration_method: Some("Talbot".into()),
                hht_alpha: None,
                rayleigh_alpha: None,
                rayleigh_beta: None,
                solver_mode: Some("PureFEM".into()),
                bem_element_type: None,
            });
            assert!(res_talbot.max_overall_displacement > 0.0, "Talbot FEM failed for {}", name);
        }

        // 2. Pure BEM Transient (Talbot NILT) across all 5 2D elements
        let bem_2d_types = [
            BemElementType::Constant,
            BemElementType::Linear,
            BemElementType::LinearDoubleNodes,
            BemElementType::Quadratic,
            BemElementType::QuadraticDiscontinuous,
        ];

        for &elem_type in &bem_2d_types {
            let res_talbot_bem_2d = run_transient_simulation(TransientSimulationRequest {
                mesh: mesh_2d_quad.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                total_time: 0.2,
                time_steps: 4,
                num_talbot_points: 8,
                load_magnitude: 50_000.0,
                pulse_duration: 0.05,
                excitation_type: Some("Pulse".into()),
                excitation_frequency: None,
                integration_method: Some("Talbot".into()),
                hht_alpha: None,
                rayleigh_alpha: None,
                rayleigh_beta: None,
                solver_mode: Some("PureBEM".into()),
                bem_element_type: Some(elem_type),
            });
            assert!(
                res_talbot_bem_2d.max_overall_displacement > 0.0,
                "Transient 2D Pure BEM failed for {:?}",
                elem_type
            );
            assert_eq!(res_talbot_bem_2d.frames.len(), 4);
        }

        // 3. Pure BEM Transient (Talbot NILT) across both 3D elements
        let bem_3d_types = [
            BemElementType::Quad4,
            BemElementType::Tri3,
        ];

        for &elem_type in &bem_3d_types {
            let res_talbot_bem_3d = run_transient_simulation(TransientSimulationRequest {
                mesh: mesh_3d_hex.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                total_time: 0.2,
                time_steps: 4,
                num_talbot_points: 8,
                load_magnitude: 50_000.0,
                pulse_duration: 0.05,
                excitation_type: Some("Pulse".into()),
                excitation_frequency: None,
                integration_method: Some("Talbot".into()),
                hht_alpha: None,
                rayleigh_alpha: None,
                rayleigh_beta: None,
                solver_mode: Some("PureBEM".into()),
                bem_element_type: Some(elem_type),
            });
            assert!(
                res_talbot_bem_3d.max_overall_displacement > 0.0,
                "Transient 3D Pure BEM failed for {:?}",
                elem_type
            );
            assert_eq!(res_talbot_bem_3d.frames.len(), 4);
        }

        println!("\n=========================================================================");
        println!(">>> TIME DOMAIN MATRIX VERIFICATION: ALL COMBINATIONS TESTED & VERIFIED");
        println!("  - Pure FEM Newmark: Tri3, Quad4, Tet4, Hex8 -> ALL PASSED");
        println!("  - Pure FEM Talbot: Tri3, Quad4, Tet4, Hex8 -> ALL PASSED");
        println!("  - Pure BEM Talbot 2D: Constant, Linear, Linear DN, Quadratic, Quadratic Disc -> ALL PASSED");
        println!("  - Pure BEM Talbot 3D: Quad4, Tri3 -> ALL PASSED");
        println!("=========================================================================\n");
    }

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
        let s = Complex64::new(1e-3, 0.0);
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

        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm};
        let strategy = DirectionalGradientField::new(Axis::Y, 0.0, 1.0, 1.0, GradientForm::Linear);
        let static_bem = crate::solver::bem_solver::solve_pure_bem_quadratic_discontinuous_with_stress_state(
            &mesh,
            &ref_mat,
            &strategy,
            50_000.0,
            crate::solver::types::StressState2D::PlaneStress,
        );
        println!("Static Pure BEM max disp = {:.6e} m", static_bem.max_displacement);
        for (idx, bnd) in mesh.boundary_elements.iter().enumerate() {
            println!("  bnd elem {}: tag={}, nodes={:?}, p0={:?}, p1={:?}",
                idx, bnd.tag, bnd.nodes, mesh.nodes[bnd.nodes[0]], mesh.nodes[bnd.nodes[1]]);
        }
        for (i, p) in mesh.nodes.iter().enumerate() {
            println!("  node {}: coord={:?}", i, p);
        }

        let static_fem = crate::solver::fem_solver::solve_pure_fem(
            &mesh,
            &ref_mat,
            &strategy,
            50_000.0,
        );
        println!("Static Pure FEM max disp = {:.6e} m", static_fem.max_displacement);

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

        let lu = a_mat.clone().lu();
        let rhs_vec = DVector::from_vec(rhs.clone());
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

                let x_test = [2.0, 1.0];
        let p_test = [1.0, 0.5];
        let n_test = [0.0, 1.0];
        let rx: f64 = x_test[0] - p_test[0];
        let ry: f64 = x_test[1] - p_test[1];
        let r_dist: f64 = (rx*rx + ry*ry).sqrt();
        let dr_vec = [rx / r_dist, ry / r_dist];

        let (u_stat, t_stat) = crate::solver::true_bem_fem::kelvin_2d_kernels(x_test, p_test, n_test, mu, nu);
        let u_dyn = crate::solver::dynamic_bem::dynamic_kelvin_u(r_dist, dr_vec, s, &ref_mat);
        let t_dyn = crate::solver::dynamic_bem::dynamic_kelvin_t(r_dist, dr_vec, n_test, s, &ref_mat);

        println!("KERNEL CHECK:");
        println!("u_stat: {:?}", u_stat);
        println!("u_dyn : {:?}", u_dyn);
        println!("t_stat: {:?}", t_stat);
        println!("t_dyn : {:?}", t_dyn);

                // Assemble static H and G exactly as in bem_solver:
        let mut h_stat_mat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
        let mut g_stat_mat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);

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
                                g_stat_mat[(i * dim + d1, col_node * dim + d2)] += g_val;
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
                                    h_stat_mat[(i * dim + d1, col_node * dim + d2)] += h_val;
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

                        let (u_star, t_star) = crate::solver::true_bem_fem::kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);

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
                                    h_stat_mat[(i * dim + d1, col_node * dim + d2)] += t_star[d1][d2] * n_alpha * d_gamma;
                                    g_stat_mat[(i * dim + d1, col_node * dim + d2)] += u_star[d1][d2] * n_alpha * d_gamma;
                                }
                            }
                        }
                    }
                }
            }
        }

        for i in 0..total_bnd_nodes {
            for d1 in 0..dim {
                let row = i * dim + d1;
                for d2 in 0..dim {
                    let mut sum = 0.0;
                    for k in 0..total_bnd_nodes {
                        if i != k {
                            sum += h_stat_mat[(row, k * dim + d2)];
                        }
                    }
                    h_stat_mat[(row, i * dim + d2)] = -sum;
                }
            }
        }

        println!("H_stat[(0,0)] = {}, H_dyn[(0,0)] = {:?}", h_stat_mat[(0,0)], h_mat[(0,0)]);
        println!("H_stat[(0,1)] = {}, H_dyn[(0,1)] = {:?}", h_stat_mat[(0,1)], h_mat[(0,1)]);
        println!("G_stat[(0,0)] = {}, G_dyn[(0,0)] = {:?}", g_stat_mat[(0,0)], g_mat[(0,0)]);
        println!("G_stat[(0,2)] = {}, G_dyn[(0,2)] = {:?}", g_stat_mat[(0,2)], g_mat[(0,2)]);

        let p_0 = bnd_node_coords[0];
        let bnd_1 = &mesh.boundary_elements[1];
        let p0_bnd = [mesh.nodes[bnd_1.nodes[0]][0], mesh.nodes[bnd_1.nodes[0]][1]];
        let p1_bnd = [mesh.nodes[bnd_1.nodes[1]][0], mesh.nodes[bnd_1.nodes[1]][1]];
        let normal_1 = [bnd_1.normal[0], bnd_1.normal[1]];
        for g in 0..2 {
            let xi_g = g_pts[g];
            let n0_geom = 0.5 * (1.0 - xi_g);
            let n1_geom = 0.5 * (1.0 + xi_g);
            let x_g = [n0_geom * p0_bnd[0] + n1_geom * p1_bnd[0], n0_geom * p0_bnd[1] + n1_geom * p1_bnd[1]];
            let (u_s, t_s) = crate::solver::true_bem_fem::kelvin_2d_kernels(x_g, p_0, normal_1, mu, nu);
            let rx = x_g[0] - p_0[0];
            let ry = x_g[1] - p_0[1];
            let r = (rx*rx + ry*ry).sqrt();
            let dr = [rx/r, ry/r];
            let t_d = crate::solver::dynamic_bem::dynamic_kelvin_t(r, dr, normal_1, s, &ref_mat);
            println!("GAUSS pt {}: r={}, dr={:?}, normal={:?}", g, r, dr, normal_1);
            println!("  t_stat: {:?}", t_s);
            println!("  t_dyn : {:?}", t_d);
        }

        let diff_h = (0..total_bnd_dofs).map(|r| (0..total_bnd_dofs).map(|c| (h_stat_mat[(r, c)] - h_mat[(r, c)].re).powi(2)).sum::<f64>()).sum::<f64>().sqrt();
        let diff_g = (0..total_bnd_dofs).map(|r| (0..total_bnd_dofs).map(|c| (g_stat_mat[(r, c)] - g_mat[(r, c)].re).powi(2)).sum::<f64>()).sum::<f64>().sqrt();
        println!("DIFF MATRIX: ||H_stat - H_dyn|| = {:.6e}, ||G_stat - G_dyn|| = {:.6e}", diff_h, diff_g);
        for r in 0..10 {
            for c in 0..10 {
                let d = (h_stat_mat[(r, c)] - h_mat[(r, c)].re).abs();
                if d > 1e-4 {
                    println!("H diff at ({}, {}): stat = {:.6e}, dyn = {:.6e}", r, c, h_stat_mat[(r, c)], h_mat[(r, c)].re);
                }
            }
        }

        // Check static BEM solution u at Right boundary nodes:
        println!("STATIC BEM RIGHT NODES:");
        for i in 0..mesh.nodes.len() {
            if (mesh.nodes[i][0] - 4.0).abs() < 1e-4 {
                println!("  Node {}: coord={:?}, u=({}, {})", i, mesh.nodes[i], static_bem.displacements[i][0], static_bem.displacements[i][1]);
            }
        }
        println!("STATIC FEM RIGHT NODES:");
        for i in 0..mesh.nodes.len() {
            if (mesh.nodes[i][0] - 4.0).abs() < 1e-4 {
                println!("  Node {}: coord={:?}, u=({}, {})", i, mesh.nodes[i], static_fem.displacements[i][0], static_fem.displacements[i][1]);
            }
        }
        // Compare static A matrix vs dynamic A matrix:
        let mut a_stat = DMatrix::<f64>::zeros(total_bnd_dofs, total_bnd_dofs);
        let mut rhs_stat = vec![0.0; total_bnd_dofs];
        let traction_y_stat = -50_000.0 / right_area;
        for k in 0..total_bnd_nodes {
            let tag = &bnd_node_tags[k];
            let is_left = tag == "Left";
            let is_right = tag == "Right";
            for d in 0..dim {
                let col = k * dim + d;
                if is_left {
                    for row in 0..total_bnd_dofs {
                        a_stat[(row, col)] = -g_stat_mat[(row, col)] * mu;
                    }
                } else {
                    for row in 0..total_bnd_dofs {
                        a_stat[(row, col)] = h_stat_mat[(row, col)];
                    }
                    let t_val = if is_right && d == 1 { traction_y_stat } else { 0.0 };
                    if t_val.abs() > 1e-12 {
                        for row in 0..total_bnd_dofs {
                            rhs_stat[row] += g_stat_mat[(row, col)] * t_val;
                        }
                    }
                }
            }
        }
        let lu_stat = a_stat.clone().lu();
        let sol_stat = lu_stat.solve(&DVector::from_vec(rhs_stat.clone())).unwrap();
        println!("Right node 13 displacement:");
        println!("  stat: ux={}, uy={}", sol_stat[13*dim], sol_stat[13*dim+1]);
        println!("  dyn : ux={:?}, uy={:?}", solution[13*dim], solution[13*dim+1]);
        println!("a_stat[(0,0)] = {}, a_mat[(0,0)] = {:?}", a_stat[(0,0)], a_mat[(0,0)]);
        println!("rhs_stat[0] = {}, rhs[0] = {:?}", rhs_stat[0], rhs[0]);
        let sol_check = lu_stat.solve(&DVector::from_vec(rhs_stat.clone())).unwrap();
        println!("sol_check[13*dim+1] = {}", sol_check[13*dim+1]);
        let a_dyn_re = a_mat.map(|c| c.re);
        let sol_dyn_re = a_dyn_re.lu().solve(&DVector::from_vec(rhs_stat.clone())).unwrap();
        println!("sol_dyn_re[13*dim+1] = {}", sol_dyn_re[13*dim+1]);
        let diff_rhs = (0..total_bnd_dofs).map(|k| (rhs_stat[k] - rhs[k].re).powi(2)).sum::<f64>().sqrt();
        println!("diff_rhs = {:.6e}", diff_rhs);
        println!("traction_y = {:?}, traction_y_stat = {}", traction_y, traction_y_stat);
        for j in 0..total_bnd_nodes {
            if bnd_node_tags[j] == "Right" {
                let col = j * dim + 1;
                println!("Right node j={}: g_stat[(1, col)] = {}, g_dyn[(1, col)] = {:?}", j, g_stat_mat[(1, col)], g_mat[(1, col)]);
            }
        }
        for k in 0..total_bnd_dofs {
            if (rhs_stat[k] - rhs[k].re).abs() > 1e-8 {
                println!("rhs diff at k={}: stat={}, dyn={}", k, rhs_stat[k], rhs[k].re);
                break;
            }
        }
        let sol_complex_with_real_rhs = lu.solve(&DVector::from_iterator(total_bnd_dofs, rhs_stat.iter().map(|&v| Complex64::new(v, 0.0)))).unwrap();
        println!("sol_complex_with_real_rhs[13*dim+1] = {:?}", sol_complex_with_real_rhs[13*dim+1]);

        println!("--- PROPER REGULARIZED BEM TEST ---");
        // Check bnd_u at Right boundary nodes:
        for j in 0..total_bnd_nodes {
            if bnd_node_tags[j] == "Right" {
                println!("Right boundary node {}: coord={:?}, u=({:?}, {:?})", 
                    j, bnd_node_coords[j], bnd_u[j][0], bnd_u[j][1]);
            }
        }

        let res_fem = crate::solver::dynamic_solver::solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            1.0,
            20,
            16,
            50_000.0,
            0.05,
            Some("Step"),
            None,
        );
        let res_bem = crate::solver::dynamic_solver::solve_transient_talbot_bem(
            &mesh,
            &ref_mat,
            &strategy,
            1.0,
            20,
            16,
            50_000.0,
            0.05,
            Some("Step"),
            None,
            crate::solver::types::BemElementType::QuadraticDiscontinuous,
        );
        println!("TRANSIENT STEP FEM max_disp = {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("TRANSIENT STEP BEM max_disp = {:.6} mm", res_bem.max_overall_displacement * 1000.0);
        for m in 0..res_fem.frames.len() {
            let t = res_fem.time_points[m];
            let d_fem = res_fem.frames[m].max_displacement * 1000.0;
            let d_bem = res_bem.frames[m].max_displacement * 1000.0;
            println!("  t = {:.3} s: FEM = {:.6} mm, BEM = {:.6} mm", t, d_fem, d_bem);
        }

        for &freq in &[1e-3, 0.1, 1.0, 3.0, 5.0, 10.0] {
            let s_val = Complex64::new(freq, 0.0);
            let u_b = crate::solver::dynamic_bem::solve_laplace_pure_bem_nodal_displacements(
                &mesh, &ref_mat, s_val, Complex64::new(50_000.0, 0.0), crate::solver::types::BemElementType::QuadraticDiscontinuous
            );
            println!("Freq s = {:5.2}: BEM tip node 4 uy = {:.6e}", freq, u_b[4*2+1].re);
        }

        let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);
        for (idx, pt) in talbot_pts.iter().enumerate().take(5) {
            let s = pt.s;
            let pulse_factor = Complex64::new(1.0, 0.0) / s;
            let load_factor = Complex64::new(50_000.0, 0.0) * pulse_factor;
            let u_bem = crate::solver::dynamic_bem::solve_laplace_pure_bem_nodal_displacements(
                &mesh, &ref_mat, s, load_factor, crate::solver::types::BemElementType::QuadraticDiscontinuous
            );
            let (k0_mat, m_mat, _) = crate::solver::fem_solver::assemble_dynamic_stiffness_and_mass(&mesh, &ref_mat, &strategy);
            let s_sq = s * s;
            let mut f_fem = vec![Complex64::new(0.0, 0.0); mesh.nodes.len() * 2];
            f_fem[4*2+1] = Complex64::new(-25_000.0, 0.0) * pulse_factor;
            f_fem[9*2+1] = Complex64::new(-25_000.0, 0.0) * pulse_factor;
            let n_dofs = mesh.nodes.len() * 2;
            let apply_a = |v: &[Complex64]| {
                let kv = crate::solver::dynamic_solver::sp_mat_mul_complex(&k0_mat, v);
                let mv = crate::solver::dynamic_solver::sp_mat_mul_complex(&m_mat, v);
                let mut out = vec![Complex64::new(0.0, 0.0); n_dofs];
                for i in 0..n_dofs {
                    out[i] = kv[i] + s_sq * mv[i];
                }
                out
            };
            let u_fem = crate::solver::krylov_solver::solve_complex_gmres(n_dofs, &f_fem, apply_a, 100, 1e-7, 100).solution;
            let r = u_fem[4*2+1] / u_bem[4*2+1];
            println!("Talbot pt {:2}: s = {:10.4} + {:10.4}i | FEM/BEM ratio = {:8.4} + {:8.4}i (|r| = {:.3})",
                idx, s.re, s.im, r.re, r.im, r.norm());
        }
    }
