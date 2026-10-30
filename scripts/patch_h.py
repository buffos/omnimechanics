with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

is_crlf = '\r\n' in text
eol = '\r\n' if is_crlf else '\n'

def norm(s):
    return s.replace('\r\n', '\n').replace('\n', eol)

# 1. factor_h_sing
old_factor = norm("""    let factor_g_sing = 1.0 / (8.0 * PI * mu * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;""")

new_factor = norm("""    let factor_g_sing = 1.0 / (8.0 * PI * mu * (1.0 - nu));
    let factor_h_sing = (1.0 - 2.0 * nu) / (4.0 * PI * (1.0 - nu));
    let kappa = 3.0 - 4.0 * nu;""")

assert old_factor in text, "old_factor not found"
text = text.replace(old_factor, new_factor, 1)

# 2. int_cauchy in solve_laplace_pure_bem_quadratic_2d
old_self = norm("""                    if alpha == local_i {
                        for d in 0..dim {
                            h_mat[(i * dim + d, col_node * dim + d)] = Complex64::new(0.5, 0.0);
                        }
                    }

                    let c = shape_coeffs[alpha];""")

new_self = norm("""                    if alpha == local_i {
                        for d in 0..dim {
                            h_mat[(i * dim + d, col_node * dim + d)] = Complex64::new(0.5, 0.0);
                        }
                    } else {
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

                    let c = shape_coeffs[alpha];""")

assert old_self in text, "old_self not found"
text = text.replace(old_self, new_self, 1)

# 3. Rigid body translation
old_rb = norm("""    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];""")

new_rb = norm("""    // Apply Rigid Body Translation condition to H:
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
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];""")

assert old_rb in text, "old_rb not found"
text = text.replace(old_rb, new_rb, 1)

with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Successfully applied factor_h_sing, int_cauchy, and rigid body translation!")
