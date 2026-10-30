with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    text = f.read()

target = """        println!("TRANSIENT STEP FEM max_disp = {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("TRANSIENT STEP BEM max_disp = {:.6} mm", res_bem.max_overall_displacement * 1000.0);"""

replacement = """        println!("TRANSIENT STEP FEM max_disp = {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("TRANSIENT STEP BEM max_disp = {:.6} mm", res_bem.max_overall_displacement * 1000.0);

        let talbot_pts = crate::solver::talbot::generate_talbot_nodes(16, 1.0);
        for (idx, pt) in talbot_pts.iter().enumerate().take(5) {
            let s = pt.s;
            let pulse_factor = Complex64::new(1.0, 0.0) / s;
            let load_factor = Complex64::new(50_000.0, 0.0) * pulse_factor;
            let u_bem = crate::solver::dynamic_bem::solve_laplace_pure_bem_nodal_displacements(
                &mesh, &ref_mat, s, load_factor, crate::solver::types::BemElementType::QuadraticDiscontinuous
            );
            println!("Talbot pt {}: s = {:?}, BEM node 4: ({:?}, {:?}), node 9: ({:?}, {:?})",
                idx, s, u_bem[4*2], u_bem[4*2+1], u_bem[9*2], u_bem[9*2+1]);
        }"""

assert target in text, "target not found"
text = text.replace(target, replacement, 1)

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("Added Talbot point inspection loop!")
