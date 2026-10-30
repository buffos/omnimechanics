#[cfg(test)]
mod tests {
    use crate::material::strategies::DirectionalGradientField;
    use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
    use crate::mesh::generators_2d::generate_beam_2d;
    use crate::solver::decoupled_hybrid::solve_decoupled_bem_fem_hybrid;
    use crate::solver::types::{HybridAcceleration, StressState2D};

    #[test]
    fn test_anderson_all_alphas_beta_0_65() {
        let length = 6.0;
        let height = 2.0;
        let p_load = 50_000.0;
        let ref_mat = ReferenceMaterial {
            e0: 30.0e9,
            nu0: 0.20,
            rho0: 2400.0,
        };
        let fem_mesh = generate_beam_2d(length, height, 12, 4, false);

        for &alpha in &[0.0, 0.20, 0.60, 1.20] {
            let strategy = DirectionalGradientField::new(Axis::Y, alpha, 2.0, 1.0, GradientForm::Linear);
            let (res, _, _) = solve_decoupled_bem_fem_hybrid(
                length,
                height,
                4,
                2,
                &fem_mesh,
                &ref_mat,
                &strategy,
                p_load,
                50,
                1e-4,
                HybridAcceleration::Anderson { depth: 3, beta: 0.65 },
                StressState2D::PlaneStress,
            );
            println!(
                "alpha = {:.2} => iters = {}, converged = {}, rel_err = {:.2e}, time = {:.2}ms",
                alpha,
                res.iterations,
                res.converged,
                res.convergence_history.last().map(|c| c.relative_error).unwrap_or(1.0),
                res.execution_time_ms
            );
        }
    }

    #[test]
    fn test_3d_parametric_preliminary() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::fem_solver::solve_pure_fem;
        use crate::solver::oraiopoulos_bem_fem::solve_oraiopoulos_bem_fem_3d;
        use crate::solver::types::HybridAcceleration;

        let length = 10.0;
        let height = 2.0;
        let depth = 2.0;
        let p_load = 50_000.0;
        let ref_mat = ReferenceMaterial {
            e0: 30.0e9,
            nu0: 0.20,
            rho0: 2400.0,
        };

        let grids = [(6, 2, 2), (10, 3, 3)];
        for &(nx, ny, nz) in &grids {
            let mesh = generate_beam_3d(length, height, depth, nx, ny, nz, false);
            println!("\n--- 3D Grid {}x{}x{} (Nodes: {}, DOFs: {}) ---", nx, ny, nz, mesh.nodes.len(), mesh.nodes.len() * 3);
            for &alpha in &[0.0] {
                let strat = DirectionalGradientField::new(Axis::X, alpha, length, 1.0, GradientForm::Linear);
                let fem_res = solve_pure_fem(&mesh, &ref_mat, &strat, p_load);
                let (bemfem_res, p_ms, c_ms) = solve_oraiopoulos_bem_fem_3d(
                    length, height, depth, nx, ny, nz,
                    &mesh, &ref_mat, &strat, p_load, 40, 1e-4,
                    HybridAcceleration::Anderson { depth: 3, beta: 0.65 },
                );
                let mut max_tip_fem: f64 = 0.0;
                for (p, u) in mesh.nodes.iter().zip(fem_res.displacements.iter()) {
                    if (p[0] - length).abs() < 1e-4 {
                        max_tip_fem = max_tip_fem.max(u[1].abs());
                    }
                }
                let mut max_tip_bem: f64 = 0.0;
                for (p, u) in mesh.nodes.iter().zip(bemfem_res.displacements.iter()) {
                    if (p[0] - length).abs() < 1e-4 {
                        max_tip_bem = max_tip_bem.max(u[1].abs());
                    }
                }
                println!(
                    "alpha={:.2} | FEM tip_w={:.6} mm (max_w={:.6} mm) | BEM-FEM tip_w={:.6} mm (max_w={:.6} mm)",
                    alpha,
                    max_tip_fem * 1000.0,
                    fem_res.max_displacement * 1000.0,
                    max_tip_bem * 1000.0,
                    bemfem_res.max_displacement * 1000.0,
                );
            }

            let test_bnd = [(6, 2, 2), (10, 3, 3), (16, 4, 4), (20, 5, 5)];
            for &(bx, by, bz) in &test_bnd {
                let pure_bem = crate::solver::oraiopoulos_bem_fem::solve_pure_bem_3d(
                    length, height, depth, bx, by, bz, &mesh, &ref_mat, p_load
                );
                println!("Pure BEM 3D {}x{}x{} max_disp = {:.6} mm", bx, by, bz, pure_bem.max_displacement * 1000.0);
            }
            return;
        }
    }
}
