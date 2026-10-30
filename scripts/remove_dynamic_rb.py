with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

is_crlf = '\r\n' in text
eol = '\r\n' if is_crlf else '\n'

def norm(s):
    return s.replace('\r\n', '\n').replace('\n', eol)

bad_rb_quad = norm("""    // Apply Rigid Body Translation condition to H:
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

assert bad_rb_quad in text, "bad_rb_quad not found"
text = text.replace(bad_rb_quad, "", 1)

with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Removed dynamic rigid body overwrite from dynamic_bem.rs!")
