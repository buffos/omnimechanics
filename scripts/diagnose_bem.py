with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::types::StressState2D;
        use num_complex::Complex64;

        let mesh = generate_beam_2d(4.0, 1.0, 4, 1, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::X,
            0.0,
            4.0,
            1.0,
            GradientForm::Linear,
        );

        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let s = Complex64::new(0.02 * 2.0 * std::f64::consts::PI * 3.0, 2.0 * std::f64::consts::PI * 3.0);

        let (h_dyn, g_dyn) = crate::solver::dynamic_bem::assemble_dynamic_bem_matrices(&mesh, &ref_mat, s);

        println!("Mesh num bnd elems = {}", mesh.boundary_elements.len());
        println!("Dynamic H[(0,0)] = {:?}", h_dyn[(0, 0)]);
        println!("Dynamic G[(0,0)] = {:?}", g_dyn[(0, 0)]);
        println!("Dynamic H[(0,2)] = {:?}", h_dyn[(0, 2)]);
        println!("Dynamic G[(0,2)] = {:?}", g_dyn[(0, 2)]);

        // Check sum of rows of H_dyn:
        for row in 0..4 {
            let mut sum_h = Complex64::new(0.0, 0.0);
            for col in 0..mesh.boundary_elements.len() {
                sum_h += h_dyn[(row, col * 2 + (row % 2))];
            }
            println!("Row {} sum(H_col) = {:?}", row, sum_h);
        }
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Diagnose test written")
