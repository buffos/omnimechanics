with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::mesh::generators_2d::generate_beam_2d;
        use num_complex::Complex64;
        use faer::Mat;
        use std::f64::consts::PI;

        let mesh = generate_beam_2d(4.0, 1.0, 4, 1, false);
        let ref_mat = ReferenceMaterial::default();
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let s = Complex64::new(0.001, 0.001); // almost zero frequency

        let num_bnd = mesh.boundary_elements.len();
        let dim = 2;
        let total_dofs = num_bnd * dim;

        // Static BEM constant
        let (h_dyn, g_dyn) = crate::solver::dynamic_bem::assemble_dynamic_bem_matrices(&mesh, &ref_mat, s);

        let elem_lengths: Vec<f64> = mesh
            .boundary_elements
            .iter()
            .map(|bnd| {
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-5)
            })
            .collect();

        let elem_centers: Vec<[f64; 2]> = mesh
            .boundary_elements
            .iter()
            .map(|bnd| {
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                [0.5 * (p0[0] + p1[0]), 0.5 * (p0[1] + p1[1])]
            })
            .collect();

        let mut h_stat = Mat::<f64>::zeros(total_dofs, total_dofs);
        let mut g_stat = Mat::<f64>::zeros(total_dofs, total_dofs);

        for i in 0..num_bnd {
            let p_i = elem_centers[i];
            for j in 0..num_bnd {
                let len_j = elem_lengths[j];
                let normal_j = [mesh.boundary_elements[j].normal[0], mesh.boundary_elements[j].normal[1]];
                let tau = [-normal_j[1], normal_j[0]];

                if i == j {
                    let int_ln = len_j * (1.0 - (0.5 * len_j).ln());
                    let factor = 1.0 / (8.0 * PI * mu * (1.0 - nu));
                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            g_stat[(i * dim + d1, j * dim + d2)] = factor * ((3.0 - 4.0 * nu) * int_ln * delta + len_j * tau[d1] * tau[d2]);
                        }
                    }
                } else {
                    let p0 = mesh.nodes[mesh.boundary_elements[j].nodes[0]];
                    let p1 = mesh.nodes[mesh.boundary_elements[j].nodes[1]];
                    let g_pts = [-0.5773502691896257, 0.5773502691896257];
                    let g_wts = [1.0, 1.0];
                    for g in 0..2 {
                        let xi = g_pts[g];
                        let wt = g_wts[g];
                        let n0 = 0.5 * (1.0 - xi);
                        let n1 = 0.5 * (1.0 + xi);
                        let x_g = [n0 * p0[0] + n1 * p1[0], n0 * p0[1] + n1 * p1[1]];
                        let d_gamma = 0.5 * len_j * wt;
                        let (u_star, t_star) = crate::solver::true_bem_fem::kelvin_2d_kernels(x_g, p_i, normal_j, mu, nu);
                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                h_stat[(i * dim + d1, j * dim + d2)] += t_star[d1][d2] * d_gamma;
                                g_stat[(i * dim + d1, j * dim + d2)] += u_star[d1][d2] * d_gamma;
                            }
                        }
                    }
                }
            }
        }

        // Apply rigid body to h_stat:
        for i in 0..num_bnd {
            for d1 in 0..dim {
                let row = i * dim + d1;
                for d2 in 0..dim {
                    let mut sum = 0.0;
                    for j in 0..num_bnd {
                        if i != j {
                            sum += h_stat[(row, j * dim + d2)];
                        }
                    }
                    h_stat[(row, i * dim + d2)] = -sum;
                }
            }
        }

        println!("--- MATRIX COMPARISON ---");
        println!("G_stat[(0, 0)] = {:.6e}, G_dyn[(0, 0)] = {:.6e}", g_stat[(0, 0)], g_dyn[(0, 0)].re);
        println!("G_stat[(0, 2)] = {:.6e}, G_dyn[(0, 2)] = {:.6e}", g_stat[(0, 2)], g_dyn[(0, 2)].re);
        println!("H_stat[(0, 0)] = {:.6f}, H_dyn[(0, 0)] = {:.6f}", h_stat[(0, 0)], h_dyn[(0, 0)].re);
        println!("H_stat[(0, 2)] = {:.6f}, H_dyn[(0, 2)] = {:.6f}", h_stat[(0, 2)], h_dyn[(0, 2)].re);
        println!("H_stat[(0, 4)] = {:.6f}, H_dyn[(0, 4)] = {:.6f}", h_stat[(0, 4)], h_dyn[(0, 4)].re);
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Compare test written")
