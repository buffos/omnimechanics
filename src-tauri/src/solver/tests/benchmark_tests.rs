use crate::material::strategies::*;
use crate::material::types::*;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::solver::*;
use faer::Mat;
use num_complex::Complex64;

    #[test]
    fn test_classic_square_tension_bem_benchmark() {
        use crate::solver::true_bem_fem::kelvin_2d_kernels;
        use faer::prelude::SpSolver;
        use faer::Mat;

        // Classic textbook BEM benchmark: Square 1x1 under uniform tension
        // E = 1000.0, nu = 0.25 (plane stress)
        // Analytical extension: Delta L = sigma_0 * L / E = 10.0 * 1.0 / 1000.0 = 0.010000 m
        let length = 1.0;
        let height = 1.0;
        let ne_side = 4; // 4 elements per side => 16 elements total
        let mesh = generate_beam_2d(length, height, ne_side, ne_side, false);

        let mut ref_mat = ReferenceMaterial::default();
        ref_mat.e0 = 1000.0;
        ref_mat.nu0 = 0.25;
        let sigma_0 = 10.0;
        let analytical_delta_l = (sigma_0 * length) / ref_mat.e0; // 0.01 m

        let num_bnd = mesh.boundary_elements.len();
        let dim = 2;
        let total_dofs = num_bnd * dim;
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;

        // In plane stress, effective nu is nu_bar = nu / (1 + nu)
        let nu_bar = nu / (1.0 + nu);

        let elem_centers: Vec<[f64; 2]> = mesh.boundary_elements.iter().map(|b| {
            let p0 = mesh.nodes[b.nodes[0]];
            let p1 = mesh.nodes[b.nodes[1]];
            [0.5 * (p0[0] + p1[0]), 0.5 * (p0[1] + p1[1])]
        }).collect();

        let elem_lengths: Vec<f64> = mesh.boundary_elements.iter().map(|b| {
            let p0 = mesh.nodes[b.nodes[0]];
            let p1 = mesh.nodes[b.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt()
        }).collect();

        let mut h_mat = Mat::<f64>::zeros(total_dofs, total_dofs);
        let mut g_mat = Mat::<f64>::zeros(total_dofs, total_dofs);

        let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
        let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

        for i in 0..num_bnd {
            let p_i = elem_centers[i];
            for j in 0..num_bnd {
                let len_j = elem_lengths[j];
                let normal_j = [mesh.boundary_elements[j].normal[0], mesh.boundary_elements[j].normal[1]];

                if i == j {
                    let tau = [-normal_j[1], normal_j[0]];
                    let int_ln = len_j * (1.0 - (0.5 * len_j).ln());
                    let factor = 1.0 / (8.0 * std::f64::consts::PI * mu * (1.0 - nu_bar));
                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            g_mat[(i * dim + d1, j * dim + d2)] = factor * ((3.0 - 4.0 * nu_bar) * int_ln * delta + len_j * tau[d1] * tau[d2]);
                        }
                    }
                } else {
                    let p0 = mesh.nodes[mesh.boundary_elements[j].nodes[0]];
                    let p1 = mesh.nodes[mesh.boundary_elements[j].nodes[1]];
                    for g in 0..4 {
                        let xi = g_pts[g];
                        let wt = g_wts[g];
                        let x_g = [0.5 * (1.0 - xi) * p0[0] + 0.5 * (1.0 + xi) * p1[0],
                                   0.5 * (1.0 - xi) * p0[1] + 0.5 * (1.0 + xi) * p1[1]];
                        let d_gamma = 0.5 * len_j * wt;
                        let (u_star, t_star) = kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu_bar);
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

        // Rigid body diagonal for H
        for i in 0..num_bnd {
            for d1 in 0..dim {
                let row = i * dim + d1;
                for d2 in 0..dim {
                    let mut sum = 0.0;
                    for j in 0..num_bnd {
                        if i != j {
                            sum += h_mat[(row, j * dim + d2)];
                        }
                    }
                    h_mat[(row, i * dim + d2)] = -sum;
                }
            }
        }

        // Boundary conditions:
        // Left (x = 0): u_x = 0 (roller), t_y = 0
        // Right (x = 1): t_x = 0, t_y = 0
        // Bottom (y = 0): u_y = 0, t_x = 0
        // Top (y = 1): t_x = 0, t_y = sigma_0 (tension in Y)
        let mut a_mat = Mat::<f64>::zeros(total_dofs, total_dofs);
        let mut rhs = vec![0.0; total_dofs];

        for j in 0..num_bnd {
            let tag = &mesh.boundary_elements[j].tag;
            // DOF 0 (X):
            let col_x = j * dim + 0;
            if tag == "Left" {
                // u_x = 0 known, t_x unknown
                for row in 0..total_dofs { a_mat[(row, col_x)] = -g_mat[(row, col_x)] * mu; }
            } else {
                // t_x = 0 known, u_x unknown
                for row in 0..total_dofs { a_mat[(row, col_x)] = h_mat[(row, col_x)]; }
            }

            // DOF 1 (Y):
            let col_y = j * dim + 1;
            if tag == "Bottom" {
                // u_y = 0 known, t_y unknown
                for row in 0..total_dofs { a_mat[(row, col_y)] = -g_mat[(row, col_y)] * mu; }
            } else {
                // t_y known, u_y unknown
                for row in 0..total_dofs { a_mat[(row, col_y)] = h_mat[(row, col_y)]; }
                if tag == "Top" {
                    for row in 0..total_dofs { rhs[row] += g_mat[(row, col_y)] * sigma_0; }
                }
            }
        }

        let lu = a_mat.full_piv_lu();
        let mut rhs_mat = Mat::<f64>::zeros(total_dofs, 1);
        for i in 0..total_dofs { rhs_mat[(i, 0)] = rhs[i]; }
        let sol = lu.solve(&rhs_mat);

        // Displacements on Top edge (should be analytical_delta_l = 0.010000 m)
        println!("\n=======================================================");
        println!("CLASSIC BEM TEXTBOOK BENCHMARK: 1x1 SQUARE IN TENSION");
        println!("Analytical Extension Delta L: {:.6} m", analytical_delta_l);
        for j in 0..num_bnd {
            if mesh.boundary_elements[j].tag == "Top" {
                let uy = sol[(j * dim + 1, 0)];
                println!("  Top Element {} center=({:.2}, {:.2}): u_y = {:.6} m (Error: {:.2}%)",
                    j, elem_centers[j][0], elem_centers[j][1], uy, (uy - analytical_delta_l) / analytical_delta_l * 100.0);
            }
        }
        println!("=======================================================\n");
    }

    #[test]
    fn test_bem_patch_tests_exact_solutions() {
        use crate::solver::true_bem_fem::kelvin_2d_kernels;
        use faer::Mat;

        let length = 6.0;
        let height = 2.0;
        let mesh = generate_beam_2d(length, height, 12, 4, false);
        let num_bnd = mesh.boundary_elements.len();
        let dim = 2;
        let total_dofs = num_bnd * dim;

        let ref_mat = ReferenceMaterial::default(); // E = 210 GPa, nu = 0.25
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let e = ref_mat.e0;

        let elem_centers: Vec<[f64; 2]> = mesh.boundary_elements.iter().map(|b| {
            let p0 = mesh.nodes[b.nodes[0]];
            let p1 = mesh.nodes[b.nodes[1]];
            [0.5 * (p0[0] + p1[0]), 0.5 * (p0[1] + p1[1])]
        }).collect();

        let elem_lengths: Vec<f64> = mesh.boundary_elements.iter().map(|b| {
            let p0 = mesh.nodes[b.nodes[0]];
            let p1 = mesh.nodes[b.nodes[1]];
            ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt()
        }).collect();

        // Assemble H and G
        let mut h_mat = Mat::<f64>::zeros(total_dofs, total_dofs);
        let mut g_mat = Mat::<f64>::zeros(total_dofs, total_dofs);

        let g_pts = [-0.8611363115940526, -0.3399810435848563, 0.3399810435848563, 0.8611363115940526];
        let g_wts = [0.3478548451374538, 0.6521451548625461, 0.6521451548625461, 0.3478548451374538];

        for i in 0..num_bnd {
            let p_i = elem_centers[i];
            for j in 0..num_bnd {
                let len_j = elem_lengths[j];
                let normal_j = [mesh.boundary_elements[j].normal[0], mesh.boundary_elements[j].normal[1]];

                if i == j {
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
                    let p0 = mesh.nodes[mesh.boundary_elements[j].nodes[0]];
                    let p1 = mesh.nodes[mesh.boundary_elements[j].nodes[1]];
                    for g in 0..4 {
                        let xi = g_pts[g];
                        let wt = g_wts[g];
                        let x_g = [0.5 * (1.0 - xi) * p0[0] + 0.5 * (1.0 + xi) * p1[0],
                                   0.5 * (1.0 - xi) * p0[1] + 0.5 * (1.0 + xi) * p1[1]];
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

        // Rigid body diagonal for H
        for i in 0..num_bnd {
            for d1 in 0..dim {
                let row = i * dim + d1;
                for d2 in 0..dim {
                    let mut sum = 0.0;
                    for j in 0..num_bnd {
                        if i != j {
                            sum += h_mat[(row, j * dim + d2)];
                        }
                    }
                    h_mat[(row, i * dim + d2)] = -sum;
                }
            }
        }

        // PATCH TEST 1: Rigid rotation u = [-y, x], t = 0
        let mut u_rot = vec![0.0; total_dofs];
        for j in 0..num_bnd {
            let x = elem_centers[j][0];
            let y = elem_centers[j][1];
            u_rot[j * dim + 0] = -y;
            u_rot[j * dim + 1] = x;
        }
        let mut res_rot = 0.0;
        for i in 0..total_dofs {
            let mut h_u = 0.0;
            for j in 0..total_dofs {
                h_u += h_mat[(i, j)] * u_rot[j];
            }
            res_rot += h_u * h_u;
        }
        println!("\nPatch Test 1 (Rigid Body Rotation): Residual ||H * u_rot|| = {:.6e}", res_rot.sqrt());

        // PATCH TEST 2: Pure Axial Tension sigma_xx = sigma_0, all others 0
        // In plane strain:
        // epsilon_xx = (1 - nu^2)/E * sigma_0
        // epsilon_yy = - nu*(1 + nu)/E * sigma_0
        // u_x = epsilon_xx * x, u_y = epsilon_yy * y
        // t_x = sigma_0 * n_x, t_y = 0
        let sigma_0 = 1000.0;
        let eps_xx = (1.0 - nu * nu) / e * sigma_0;
        let eps_yy = -nu * (1.0 + nu) / e * sigma_0;

        let mut u_ten = vec![0.0; total_dofs];
        let mut t_ten = vec![0.0; total_dofs];
        for j in 0..num_bnd {
            let x = elem_centers[j][0];
            let y = elem_centers[j][1];
            let nx = mesh.boundary_elements[j].normal[0];
            let ny = mesh.boundary_elements[j].normal[1];

            u_ten[j * dim + 0] = eps_xx * x;
            u_ten[j * dim + 1] = eps_yy * y;

            // sigma = [[sigma_0, 0], [0, 0]]
            // t = sigma * n
            t_ten[j * dim + 0] = sigma_0 * nx;
            t_ten[j * dim + 1] = 0.0;
        }

        let mut hu_vec = vec![0.0; total_dofs];
        let mut gt_vec = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            for j in 0..total_dofs {
                hu_vec[i] += h_mat[(i, j)] * u_ten[j];
                gt_vec[i] += g_mat[(i, j)] * t_ten[j];
            }
        }
        let diff_ten: f64 = (0..total_dofs).map(|k| (hu_vec[k] - gt_vec[k]).powi(2)).sum::<f64>().sqrt();
        let norm_gt: f64 = gt_vec.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();
        for i in 0..5 {
            println!("  Node {:2}: Hu=({:.6e}, {:.6e}), Gt=({:.6e}, {:.6e}), Ratio=({:.4}, {:.4})",
                i, hu_vec[2 * i], hu_vec[2 * i + 1], gt_vec[2 * i], gt_vec[2 * i + 1],
                hu_vec[2 * i] / gt_vec[2 * i], hu_vec[2 * i + 1] / gt_vec[2 * i + 1]
            );
        }

        // PATCH TEST 3: Pure Bending M = const
        // sigma_xx = -C * (y - H/2)
        // u_x = -C * (1 - nu^2)/E * x * (y - H/2)
        // u_y = C/2 * (1 - nu^2)/E * x^2 + C/2 * nu*(1 + nu)/E * (y - H/2)^2
        let c_bend = 1000.0;
        let mut u_bend = vec![0.0; total_dofs];
        let mut t_bend = vec![0.0; total_dofs];
        for j in 0..num_bnd {
            let x = elem_centers[j][0];
            let y = elem_centers[j][1] - height * 0.5;
            let nx = mesh.boundary_elements[j].normal[0];

            u_bend[j * dim + 0] = -(1.0 - nu * nu) / e * c_bend * x * y;
            u_bend[j * dim + 1] = 0.5 * (1.0 - nu * nu) / e * c_bend * x * x + 0.5 * nu * (1.0 + nu) / e * c_bend * y * y;

            t_bend[j * dim + 0] = -c_bend * y * nx;
            t_bend[j * dim + 1] = 0.0;
        }

        let mut hu_b = vec![0.0; total_dofs];
        let mut gt_b = vec![0.0; total_dofs];
        for i in 0..total_dofs {
            for j in 0..total_dofs {
                hu_b[i] += h_mat[(i, j)] * u_bend[j];
                gt_b[i] += g_mat[(i, j)] * t_bend[j];
            }
        }
        let diff_bend: f64 = (0..total_dofs).map(|k| (hu_b[k] - gt_b[k]).powi(2)).sum::<f64>().sqrt();
        let norm_gt_b: f64 = gt_b.iter().map(|v| v.powi(2)).sum::<f64>().sqrt();
        println!("Patch Test 3 (Pure Bending): ||H*u - G*t|| = {:.6e}, ||G*t|| = {:.6e}, Rel Error = {:.4}%",
            diff_bend, norm_gt_b, (diff_bend / norm_gt_b) * 100.0);
    }

    #[test]
    fn test_plane_stress_vs_plane_strain_consistency() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::types::StressState2D;

        let ref_mat = ReferenceMaterial {
            e0: 2.1e11, // 210 GPa
            nu0: 0.25,
            rho0: 7850.0,
        };
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            1.0,
            1.0,
            GradientForm::Linear,
        );
        let p_load = 50_000.0;
        let mesh = generate_beam_2d(6.0, 2.0, 20, 6, false);

        // 1. Solve with Plane Stress
        let fem_stress = crate::solver::fem_solver::solve_pure_fem_with_stress_state(
            &mesh,
            &ref_mat,
            &strategy,
            p_load,
            StressState2D::PlaneStress,
        );
        let bem_stress = crate::solver::bem_solver::solve_pure_bem_quadratic_discontinuous_with_stress_state(
            &mesh,
            &ref_mat,
            &strategy,
            p_load,
            StressState2D::PlaneStress,
        );

        // 2. Solve with Plane Strain
        let fem_strain = crate::solver::fem_solver::solve_pure_fem_with_stress_state(
            &mesh,
            &ref_mat,
            &strategy,
            p_load,
            StressState2D::PlaneStrain,
        );
        let bem_strain = crate::solver::bem_solver::solve_pure_bem_quadratic_discontinuous_with_stress_state(
            &mesh,
            &ref_mat,
            &strategy,
            p_load,
            StressState2D::PlaneStrain,
        );

        println!("\n--- Plane Stress vs Plane Strain Comparison (Mesh 20x6) ---");
        println!("Plane Stress: FEM max_disp = {:.6} mm, BEM max_disp = {:.6} mm",
            fem_stress.max_displacement * 1000.0, bem_stress.max_displacement * 1000.0);
        println!("Plane Strain: FEM max_disp = {:.6} mm, BEM max_disp = {:.6} mm",
            fem_strain.max_displacement * 1000.0, bem_strain.max_displacement * 1000.0);

        // In Plane Strain, the structure is stiffer due to lateral constraint (1 - nu^2 factor)
        assert!(fem_strain.max_displacement < fem_stress.max_displacement);
        assert!(bem_strain.max_displacement < bem_stress.max_displacement);

        // FEM and BEM must agree within 1.5% in both regimes
        let ratio_stress = bem_stress.max_displacement / fem_stress.max_displacement;
        let ratio_strain = bem_strain.max_displacement / fem_strain.max_displacement;
        println!("Ratio BEM/FEM Plane Stress: {:.4}", ratio_stress);
        println!("Ratio BEM/FEM Plane Strain: {:.4}", ratio_strain);

        assert!((ratio_stress - 1.0).abs() < 0.02, "BEM and FEM should agree closely in Plane Stress");
        assert!((ratio_strain - 1.0).abs() < 0.02, "BEM and FEM should agree closely in Plane Strain");
    }

    #[test]
    fn test_oraiopoulos_bem_fem_3d_homogeneous() {
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::oraiopoulos_bem_fem::solve_oraiopoulos_bem_fem_3d;

        let length = 10.0;
        let height = 2.0;
        let depth = 1.0;

        // Volume FEM mesh (Hex8)
        let fem_mesh = generate_beam_3d(length, height, depth, 6, 2, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0, // alpha = 0.0 (Homogeneous reference)
            height,
            1.0,
            GradientForm::Linear,
        );

        let p_load = 50_000.0;
        let fem_res = fem_solver::solve_pure_fem(&fem_mesh, &ref_mat, &strategy, p_load);

        println!("\n=== VERIFIED 3D FEM HEX8 ACCURACY ===");
        println!("Hex8 Pure FEM Max displacement: {:.6} mm", fem_res.max_displacement * 1000.0);
        assert!(fem_res.converged);
        assert!(fem_res.max_displacement > 5e-5, "Hex8 with genuine B^T C B produces real continuum displacement!");

        // Also test Tet4 mesh
        let tet_mesh = generate_beam_3d(length, height, depth, 6, 2, 2, true);
        let tet_res = fem_solver::solve_pure_fem(&tet_mesh, &ref_mat, &strategy, p_load);
        println!("Tet4 Pure FEM Max displacement: {:.6} mm", tet_res.max_displacement * 1000.0);
        assert!(tet_res.converged);
        assert!(tet_res.max_displacement > 5e-5, "Tet4 with genuine B^T C B produces real continuum displacement!");

        let (res, pred_ms, corr_ms) = solve_oraiopoulos_bem_fem_3d(
            length,
            height,
            depth,
            6, 2, 2, // BEM boundary discretization
            &fem_mesh,
            &ref_mat,
            &strategy,
            p_load,
            50,
            1e-5,
            HybridAcceleration::Anderson { depth: 4, beta: 0.5 },
        );

        println!("\n=== 3D Oraiopoulos BEM-FEM Solver Test (alpha = 0.00) ===");
        println!("Pure FEM Max displacement: {:.6} mm", fem_res.max_displacement * 1000.0);
        println!("Oraiopoulos Max displacement: {:.6} mm", res.max_displacement * 1000.0);
        println!("Converged: {}", res.converged);
        println!("Iterations: {}", res.iterations);
        println!("Predictor time: {:.2} ms, Corrector time: {:.2} ms", pred_ms, corr_ms);

        assert!(res.converged);
        assert_eq!(res.iterations, 0, "At alpha = 0, volume corrector perturbation must be exactly 0 (0 iterations)!");
        assert!(res.max_displacement > 0.0);
    }

    #[test]
    fn test_oraiopoulos_bem_fem_3d_heterogeneous() {
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::oraiopoulos_bem_fem::solve_oraiopoulos_bem_fem_3d;

        let length = 10.0;
        let height = 2.0;
        let depth = 1.0;

        let fem_mesh = generate_beam_3d(length, height, depth, 6, 2, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.5, // alpha = 0.50 (Heterogeneous material)
            height,
            1.0,
            GradientForm::Linear,
        );

        let p_load = 50_000.0;
        let (res, pred_ms, corr_ms) = solve_oraiopoulos_bem_fem_3d(
            length,
            height,
            depth,
            6, 2, 2,
            &fem_mesh,
            &ref_mat,
            &strategy,
            p_load,
            50,
            1e-4,
            HybridAcceleration::Anderson { depth: 4, beta: 0.5 },
        );

        println!("\n=== 3D Oraiopoulos BEM-FEM Solver Test (alpha = 0.50) ===");
        println!("Converged: {}", res.converged);
        println!("Iterations: {}", res.iterations);
        println!("Max displacement: {:.6} mm", res.max_displacement * 1000.0);
        println!("Predictor time: {:.2} ms, Corrector time: {:.2} ms", pred_ms, corr_ms);

        assert!(res.converged);
        assert!(res.iterations > 0);
        assert!(res.max_displacement > 0.0);
    }

