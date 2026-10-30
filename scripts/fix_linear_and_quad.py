with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# First revert the wrong change in linear_2d
wrong_code = """                // Add regularized dynamic minus static difference on self element
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
"""

is_crlf = '\r\n' in text
eol = '\r\n' if is_crlf else '\n'

def norm(s):
    return s.replace('\r\n', '\n').replace('\n', eol)

wrong_code_norm = norm(wrong_code)

if wrong_code_norm in text:
    text = text.replace(wrong_code_norm, "", 1)
    print("Reverted accidental linear_2d change")

# Now apply it ONLY inside solve_laplace_pure_bem_quadratic_2d
quad_marker = "pub fn solve_laplace_pure_bem_quadratic_2d"
quad_pos = text.find(quad_marker)
assert quad_pos != -1, "quad_marker not found"

elem_else = norm("""            } else {
                for (g_idx, &xi_g) in g_pts.iter().enumerate() {""")
target_pos = text.find(elem_else, quad_pos)
assert target_pos != -1, "elem_else not found inside quadratic"

text = text[:target_pos] + wrong_code_norm + text[target_pos:]

with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Applied self-element u_diff strictly to solve_laplace_pure_bem_quadratic_2d!")
