with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

is_crlf = '\r\n' in text
eol = '\r\n' if is_crlf else '\n'

def norm(s):
    return s.replace('\r\n', '\n').replace('\n', eol)

target = norm("""                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g_sing * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_mat[(i * dim + d1, col_node * dim + d2)] += Complex64::new(g_val, 0.0);
                        }
                    }
                }
            } else {""")

replacement = norm("""                    for d1 in 0..dim {
                        for d2 in 0..dim {
                            let delta = if d1 == d2 { 1.0 } else { 0.0 };
                            let g_val = factor_g_sing * (kappa * delta * int_ln_r + tau[d1] * tau[d2] * int_tau);
                            g_mat[(i * dim + d1, col_node * dim + d2)] += Complex64::new(g_val, 0.0);
                        }
                    }
                }

                // Add regularized dynamic minus static difference on self element
                for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                    let w_g = g_wts[g_idx];
                    let n0_geom = 0.5 * (1.0 - xi_g);
                    let n1_geom = 0.5 * (1.0 + xi_g);
                    let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];

                    let rx = x_g[0] - p_i[0];
                    let ry = x_g[1] - p_i[1];
                    let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                    let dr = [rx / r, ry / r];

                    let u_star_dyn = dynamic_kelvin_u(r, dr, s, reference);
                    let ln_inv_r = (1.0 / r).ln();
                    let u_star_stat = [
                        [factor_g_sing * (kappa * ln_inv_r + dr[0] * dr[0]), factor_g_sing * (dr[0] * dr[1])],
                        [factor_g_sing * (dr[1] * dr[0]), factor_g_sing * (kappa * ln_inv_r + dr[1] * dr[1])],
                    ];

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
                                let u_diff = u_star_dyn[d1][d2] - Complex64::new(u_star_stat[d1][d2], 0.0);
                                g_mat[(i * dim + d1, col_node * dim + d2)] += u_diff * Complex64::new(n_alpha * d_gamma, 0.0);
                            }
                        }
                    }
                }
            } else {""")

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added self-element u_diff to dynamic_bem.rs!")
