with open('src-tauri/src/solver/tests.rs', 'r', encoding='utf-8') as f:
    content = f.read()

test_code = """
    #[test]
    fn test_debug_beam_transient_fem_vs_bem() {
        use crate::material::types::ReferenceMaterial;
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::bem_solver::integrate_poly_ln;
        use num_complex::Complex64;
        use nalgebra::{DMatrix, DVector};

        let mesh = generate_beam_2d(4.0, 1.0, 4, 1, false);
        let ref_mat = ReferenceMaterial::default();
        let mu = ref_mat.mu0();
        let nu = ref_mat.nu0;
        let s = Complex64::new(0.02 * 2.0 * std::f64::consts::PI * 3.0, 2.0 * std::f64::consts::PI * 3.0);

        let num_bnd_elems = mesh.boundary_elements.len();
        let num_elem_nodes = 3;
        let total_bnd_nodes = num_bnd_elems * num_elem_nodes;
        let dim = 2;
        let total_bnd_dofs = total_bnd_nodes * dim;

        let elem_lengths: Vec<f64> = mesh
            .boundary_elements
            .iter()
            .map(|bnd| {
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt().max(1e-5)
            })
            .collect();

        let right_area: f64 = mesh
            .boundary_elements
            .iter()
            .enumerate()
            .filter(|(_, b)| b.tag == "Right")
            .map(|(j, _)| elem_lengths[j])
            .sum::<f64>()
            .max(1e-4);

        let traction_y = -50_000.0 / right_area;
        println!("traction_y = {}", traction_y);

        // Let's compare G_mat and H_mat elements
        // In static:
        // What is H_mat and G_mat?
    }
"""

start_idx = content.find("fn test_debug_beam_transient_fem_vs_bem()")
if start_idx != -1:
    prev_idx = content.rfind("#[test]", 0, start_idx)
    new_content = content[:prev_idx] + test_code.strip() + "\n}\n"
    with open('src-tauri/src/solver/tests.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)
    print("Debug test written")
