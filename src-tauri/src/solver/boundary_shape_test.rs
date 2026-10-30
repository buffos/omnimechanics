use super::*;

use crate::mesh::generators_2d::generate_beam_2d;

    #[test]
    fn test_boundary_shape_matrix_2d() {
        // Simple 2D beam: length = 4.0, height = 2.0, 2x1 elements
        let mesh = generate_beam_2d(4.0, 2.0, 2, 1, false);
        let s_mat = BoundaryShapeMatrix::assemble(&mesh);

        assert_eq!(s_mat.dimension, 2);
        assert!(s_mat.num_bnd_dofs > 0);

        // Test traction to force with uniform unit traction
        let uniform_t = vec![1.0; s_mat.num_bnd_dofs];
        let forces = s_mat.tractions_to_forces(&uniform_t);
        assert_eq!(forces.len(), s_mat.num_bnd_dofs);

        // Total perimeter in 2D = 2 * (4 + 2) = 12.0
        // Sum of horizontal and vertical forces should equal total perimeter
        let total_fx: f64 = forces.iter().step_by(2).sum();
        let total_fy: f64 = forces.iter().skip(1).step_by(2).sum();
        assert!((total_fx - 12.0).abs() < 1e-4);
        assert!((total_fy - 12.0).abs() < 1e-4);

        // Test roundtrip: forces_to_tractions(tractions_to_forces(t)) == t
        let recovered_t = s_mat.forces_to_tractions(&forces, 100, 1e-8);
        for (orig, rec) in uniform_t.iter().zip(&recovered_t) {
            assert!((orig - rec).abs() < 1e-4);
        }
    }
