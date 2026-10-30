with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = "let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);"

replacement = """for &freq in &[1e-3, 0.1, 1.0, 3.0, 5.0, 10.0] {
            let s_val = Complex64::new(freq, 0.0);
            let u_b = crate::solver::dynamic_bem::solve_laplace_pure_bem_nodal_displacements(
                &mesh, &ref_mat, s_val, Complex64::new(50_000.0, 0.0), crate::solver::types::BemElementType::QuadraticDiscontinuous
            );
            println!("Freq s = {:5.2}: BEM tip node 4 uy = {:.6e}", freq, u_b[4*2+1].re);
        }

        let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added frequency sweep test!")
