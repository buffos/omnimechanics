with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::dynamic_solver;
        use crate::solver::types::BemElementType;

        let mesh = generate_beam_2d(4.0, 1.0, 8, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::X,
            0.0,
            4.0,
            1.0,
            GradientForm::Linear,
        );

        let res_fem = dynamic_solver::solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            0.50,
            25,
            32,
            50_000.0,
            0.05,
            Some("HarmonicSine"),
            Some(3.0),
        );

        println!("\\n=== DEBUG TRANSIENT ===");
        println!("Pure FEM max disp = {:.6} m ({:.3} mm)", 
            res_fem.max_overall_displacement, res_fem.max_overall_displacement * 1000.0);

        for &elem_type in &[
            BemElementType::Constant,
            BemElementType::Linear,
            BemElementType::QuadraticDiscontinuous,
        ] {
            let res_bem = dynamic_solver::solve_transient_talbot_bem(
                &mesh,
                &ref_mat,
                &strategy,
                0.50,
                25,
                32,
                50_000.0,
                0.05,
                Some("HarmonicSine"),
                Some(3.0),
                elem_type,
            );
            println!("Pure BEM {:?} max disp = {:.6} m ({:.3} mm)",
                elem_type, res_bem.max_overall_displacement, res_bem.max_overall_displacement * 1000.0);
        }
    }
"""

last_brace = content.rfind('}')
new_content = content[:last_brace] + test_code + '\n}\n'

with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Added debug test successfully!")
