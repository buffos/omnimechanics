with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

is_crlf = '\r\n' in text
eol = '\r\n' if is_crlf else '\n'

def norm(s):
    return s.replace('\r\n', '\n').replace('\n', eol)

bad_rb_in_const = norm("""    // Apply Rigid Body Translation condition to H:
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

""")

assert bad_rb_in_const in text, "bad_rb_in_const not found"
text = text.replace(bad_rb_in_const, "", 1)

# Now find solve_laplace_pure_bem_quadratic_2d and put rigid body translation before let mut a_mat
quad_marker = "pub fn solve_laplace_pure_bem_quadratic_2d"
quad_pos = text.find(quad_marker)
assert quad_pos != -1, "solve_laplace_pure_bem_quadratic_2d not found"

amat_marker = norm("""    let mut a_mat = DMatrix::<Complex64>::zeros(total_bnd_dofs, total_bnd_dofs);
    let mut rhs = vec![Complex64::new(0.0, 0.0); total_bnd_dofs];""")

amat_pos = text.find(amat_marker, quad_pos)
assert amat_pos != -1, "a_mat inside quadratic not found"

new_rb_quad = norm("""    // Apply Rigid Body Translation condition to H:
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

""") + amat_marker

text = text[:amat_pos] + new_rb_quad + text[amat_pos + len(amat_marker):]

with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Moved rigid body translation to solve_laplace_pure_bem_quadratic_2d!")
