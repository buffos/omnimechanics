with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::types::BemElementType;
        use num_complex::Complex64;

        let mesh = generate_beam_2d(4.0, 1.0, 8, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let s = Complex64::new(0.02 * 2.0 * std::f64::consts::PI * 3.0, 2.0 * std::f64::consts::PI * 3.0);
        let load_factor = Complex64::new(50_000.0, 0.0);

        println!("--- RUNNING CONSTANT 2D ---");
        let u_const = crate::solver::dynamic_bem::solve_laplace_pure_bem_constant_2d(&mesh, &ref_mat, s, load_factor);
        let max_u_const = u_const.iter().map(|c| c.norm()).fold(0.0, f64::max);
        println!("Constant max u = {:.6e}", max_u_const);

        println!("--- RUNNING QUADRATIC 2D ---");
        let u_quad = crate::solver::dynamic_bem::solve_laplace_pure_bem_quadratic_2d(&mesh, &ref_mat, s, load_factor);
        let max_u_quad = u_quad.iter().map(|c| c.norm()).fold(0.0, f64::max);
        println!("Quadratic max u = {:.6e}", max_u_quad);
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Debug test written")
