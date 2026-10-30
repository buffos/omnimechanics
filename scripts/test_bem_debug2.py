with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::mesh::generators_2d::generate_beam_2d;
        use num_complex::Complex64;

        let mesh = generate_beam_2d(4.0, 1.0, 8, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let s = Complex64::new(0.02 * 2.0 * std::f64::consts::PI * 3.0, 2.0 * std::f64::consts::PI * 3.0);
        let load_factor = Complex64::new(50_000.0, 0.0);

        // Let's inspect A_mat condition and solution in static vs dynamic
        let res_stat = crate::solver::bem_solver::solve_pure_bem_quadratic_discontinuous_with_stress_state(
            &mesh, &ref_mat, &crate::material::strategies::DirectionalGradientField::new(
                crate::material::types::Axis::X, 0.0, 4.0, 1.0, crate::material::types::GradientForm::Linear),
            50_000.0, crate::solver::types::StressState2D::PlaneStress);
        println!("Static max disp = {:.6e} m ({:.3} mm)", res_stat.max_displacement, res_stat.max_displacement * 1000.0);
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Debug test written")
