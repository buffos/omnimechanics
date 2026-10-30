with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::{dynamic_solver, dynamic_bem, fem_solver, bem_solver};
        use crate::solver::types::{BemElementType, StressState2D};
        use num_complex::Complex64;

        let ref_mat = ReferenceMaterial::default();
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let s = Complex64::new(0.02 * 2.0 * std::f64::consts::PI * 3.0, 2.0 * std::f64::consts::PI * 3.0);
        let r = 1.0;
        let dr = [1.0, 0.0];
        let normal = [1.0, 0.0];

        let u_dyn = dynamic_bem::dynamic_kelvin_u(r, dr, s, &ref_mat);
        let t_dyn = dynamic_bem::dynamic_kelvin_t(r, dr, normal, s, &ref_mat);

        let (u_stat, t_stat) = crate::solver::true_bem_fem::kelvin_2d_kernels([1.0, 0.0], [0.0, 0.0], normal, mu, nu);

        println!("r=1.0:");
        println!("  Dynamic U_xx = {:?}", u_dyn[0][0]);
        println!("  Static  U_xx = {}", u_stat[0][0]);
        println!("  Ratio   U_xx = {}", u_dyn[0][0].re / u_stat[0][0]);
        println!("  Dynamic T_xx = {:?}", t_dyn[0][0]);
        println!("  Static  T_xx = {}", t_stat[0][0]);
        println!("  Ratio   T_xx = {}", t_dyn[0][0].re / t_stat[0][0]);
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Updated debug test successfully")
