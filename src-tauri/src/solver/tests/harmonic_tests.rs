use crate::material::strategies::*;
use crate::material::types::*;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::solver::*;
use faer::Mat;
use num_complex::Complex64;

    #[test]
    fn test_harmonic_pure_bem_2d_and_3d() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::mesh::types::ElementType;
        use crate::solver::dynamic_bem::{solve_harmonic_pure_bem_2d, solve_harmonic_pure_bem_3d};

        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::X,
            0.0,
            4.0,
            1.0,
            GradientForm::Linear,
        );

        // 1. 2D Harmonic Pure BEM (Quad4 volume mesh with Line2 boundary elements)
        let mesh_2d = generate_beam_2d(4.0, 1.0, 8, 2, false);
        let res_2d = solve_harmonic_pure_bem_2d(&mesh_2d, &ref_mat, &strategy, 5.0, 50_000.0);
        let res_2d_quad = crate::solver::dynamic_bem::solve_harmonic_pure_bem_quadratic_2d(&mesh_2d, &ref_mat, &strategy, 5.0, 50_000.0);
        let res_fem_2d = crate::solver::dynamic_solver::solve_harmonic(&mesh_2d, &ref_mat, &strategy, 5.0, 50_000.0, 200, 1e-5);
        println!("\n=== 2D Harmonic Comparison (5 Hz) ===");
        println!("FEM Max amplitude: {:.6} mm", res_fem_2d.max_amplitude * 1000.0);
        println!("BEM Const Max amplitude: {:.6} mm", res_2d.max_amplitude * 1000.0);
        println!("BEM Quad Max amplitude: {:.6} mm", res_2d_quad.max_amplitude * 1000.0);
        println!("Execution time: {:.2} ms", res_2d.execution_time_ms);
        assert!(res_2d.converged);
        assert!(res_2d.max_amplitude > 0.0);

        // 2. 3D Harmonic Pure BEM (Hex8 volume mesh with Quad4 boundary elements)
        let mesh_3d = generate_beam_3d(3.0, 1.0, 1.0, 4, 2, 2, false);
        let res_3d = solve_harmonic_pure_bem_3d(&mesh_3d, &ref_mat, &strategy, 5.0, 50_000.0);
        let res_3d_tri = crate::solver::dynamic_bem::solve_harmonic_pure_bem_tri3_3d(&mesh_3d, &ref_mat, &strategy, 5.0, 50_000.0);
        let res_fem_3d = crate::solver::dynamic_solver::solve_harmonic(&mesh_3d, &ref_mat, &strategy, 5.0, 50_000.0, 200, 1e-5);
        println!("\n=== 3D Comparison (Static & 5 Hz Dynamic) ===");
        println!("FEM Harmonic Max amp: {:.6} mm", res_fem_3d.max_amplitude * 1000.0);
        println!("BEM Quad4 Max amp: {:.6} mm", res_3d.max_amplitude * 1000.0);
        println!("BEM Tri3 Max amp: {:.6} mm", res_3d_tri.max_amplitude * 1000.0);
        println!("Execution time: {:.2} ms", res_3d.execution_time_ms);
        assert!(res_3d.converged);
        assert!(res_3d.max_amplitude > 0.0);

        // 3. 3D Transient Talbot Comparison
        let res_trans_fem = crate::solver::dynamic_solver::solve_transient_talbot(
            &mesh_3d, &ref_mat, &strategy, 0.05, 5, 8, 50_000.0, 0.01, Some("Pulse"), None,
        );
        let res_trans_bem_tri = crate::solver::dynamic_solver::solve_transient_talbot_bem(
            &mesh_3d, &ref_mat, &strategy, 0.05, 5, 8, 50_000.0, 0.01, Some("Pulse"), None,
            crate::solver::types::BemElementType::Tri3,
        );
        let res_trans_bem_quad = crate::solver::dynamic_solver::solve_transient_talbot_bem(
            &mesh_3d, &ref_mat, &strategy, 0.05, 5, 8, 50_000.0, 0.01, Some("Pulse"), None,
            crate::solver::types::BemElementType::Quad4,
        );
        println!("\n=== 3D Transient Talbot Comparison (t=0.05s) ===");
        println!("FEM Transient Max disp: {:.6} mm", res_trans_fem.max_overall_displacement * 1000.0);
        println!("BEM Tri3 Transient Max disp: {:.6} mm", res_trans_bem_tri.max_overall_displacement * 1000.0);
        println!("BEM Quad4 Transient Max disp: {:.6} mm", res_trans_bem_quad.max_overall_displacement * 1000.0);
        assert!(res_trans_bem_tri.max_overall_displacement > 0.0);
        assert!(res_trans_bem_quad.max_overall_displacement > 0.0);
    }

    #[test]
    fn test_harmonic_all_modes_and_elements() {
        use crate::material::types::{Axis, GradientForm, MaterialStrategyConfig, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::types::{HarmonicSimulationRequest, HarmonicSolverMode};
        use crate::solver::run_harmonic_simulation;

        let ref_mat = ReferenceMaterial::default();
        let strategy = MaterialStrategyConfig::DirectionalGradient {
            axis: Axis::X,
            alpha: 0.0,
            length_scale: 4.0,
            power: 1.0,
            form: GradientForm::Linear,
        };

        // 1. Pure FEM 2D Tri3
        let mesh_2d_tri = generate_beam_2d(4.0, 1.0, 6, 2, true);
        let req_fem_tri = HarmonicSimulationRequest {
            mesh: mesh_2d_tri,
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_fem_tri = run_harmonic_simulation(req_fem_tri);
        assert!(res_fem_tri.converged);
        assert!(res_fem_tri.max_amplitude > 0.0);

        // 2. Pure FEM 2D Quad4
        let mesh_2d_quad = generate_beam_2d(4.0, 1.0, 6, 2, false);
        let req_fem_quad = HarmonicSimulationRequest {
            mesh: mesh_2d_quad.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_fem_quad = run_harmonic_simulation(req_fem_quad);
        assert!(res_fem_quad.converged);
        assert!(res_fem_quad.max_amplitude > 0.0);

        // 3. Pure BEM 2D
        let req_bem_2d = HarmonicSimulationRequest {
            mesh: mesh_2d_quad.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureBEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_bem_2d = run_harmonic_simulation(req_bem_2d);
        assert!(res_bem_2d.converged);
        assert!(res_bem_2d.max_amplitude > 0.0);

        // 4. Robin SSI 2D
        let req_robin_2d = HarmonicSimulationRequest {
            mesh: mesh_2d_quad,
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::Robin),
            bem_element_type: None,
            stress_state: None,
        };
        let res_robin_2d = run_harmonic_simulation(req_robin_2d);
        println!("Robin 2D converged: {}, iters: {}, rel_res: {:.6e}", res_robin_2d.converged, res_robin_2d.iterations, res_robin_2d.relative_residual);
        assert!(res_robin_2d.converged);
        assert!(res_robin_2d.max_amplitude > 0.0);

        // 5. Pure FEM 3D Hex8
        let mesh_3d_hex = generate_beam_3d(3.0, 1.0, 1.0, 3, 2, 2, false);
        let req_fem_hex = HarmonicSimulationRequest {
            mesh: mesh_3d_hex.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_fem_hex = run_harmonic_simulation(req_fem_hex);
        assert!(res_fem_hex.converged);
        assert!(res_fem_hex.max_amplitude > 0.0);

        // 6. Pure FEM 3D Tet4
        let mesh_3d_tet = generate_beam_3d(3.0, 1.0, 1.0, 2, 1, 1, true);
        let req_fem_tet = HarmonicSimulationRequest {
            mesh: mesh_3d_tet,
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 60,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_fem_tet = run_harmonic_simulation(req_fem_tet);
        assert!(res_fem_tet.converged);
        assert!(res_fem_tet.max_amplitude > 0.0);

        // 7. Pure BEM 3D
        let req_bem_3d = HarmonicSimulationRequest {
            mesh: mesh_3d_hex.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureBEM),
            bem_element_type: None,
            stress_state: None,
        };
        let res_bem_3d = run_harmonic_simulation(req_bem_3d);
        assert!(res_bem_3d.converged);
        assert!(res_bem_3d.max_amplitude > 0.0);

        // 8. Robin SSI 3D
        let req_robin_3d = HarmonicSimulationRequest {
            mesh: mesh_3d_hex,
            reference_material: ref_mat.clone(),
            strategy: strategy.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::Robin),
            bem_element_type: None,
            stress_state: None,
        };
        let res_robin_3d = run_harmonic_simulation(req_robin_3d);
        assert!(res_robin_3d.converged);
        assert!(res_robin_3d.max_amplitude > 0.0);

        println!("\n=== ALL 8 HARMONIC COMBINATIONS (2D/3D FEM Tri3/Quad4/Tet4/Hex8, BEM 2D/3D, Robin 2D/3D) PASSED ===");
    }

    #[test]
    fn test_all_elements_pure_bem_and_fem_static_and_harmonic() {
        use crate::material::types::{Axis, GradientForm, MaterialStrategyConfig, ReferenceMaterial};
        use crate::material::strategies::DirectionalGradientField;
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::mesh::generators_3d::generate_beam_3d;
        use crate::solver::types::{BemElementType, HarmonicSimulationRequest, HarmonicSolverMode, SimulationRequest, SolverMode};
        use crate::solver::{run_simulation, run_harmonic_simulation};

        let ref_mat = ReferenceMaterial::default();
        let strategy_cfg = MaterialStrategyConfig::DirectionalGradient {
            axis: Axis::X,
            alpha: 0.0,
            length_scale: 4.0,
            power: 1.0,
            form: GradientForm::Linear,
        };
        let strategy = DirectionalGradientField::new(
            Axis::X,
            0.0,
            4.0,
            1.0,
            GradientForm::Linear,
        );

        let mesh_2d_tri = generate_beam_2d(4.0, 1.0, 4, 2, true);
        let mesh_2d_quad = generate_beam_2d(4.0, 1.0, 4, 2, false);
        let mesh_3d_tet = generate_beam_3d(3.0, 1.0, 1.0, 2, 1, 1, true);
        let mesh_3d_hex = generate_beam_3d(3.0, 1.0, 1.0, 2, 1, 1, false);

        // -------------------------------------------------------------
        // 1. FEM: Static & Harmonic on all 4 elements (Tri3, Quad4, Tet4, Hex8)
        // -------------------------------------------------------------
        // 1.1 Tri3 FEM (Static & Harmonic)
        let res_fem_s_tri = crate::solver::fem_solver::solve_pure_fem(&mesh_2d_tri, &ref_mat, &strategy, 50_000.0);
        assert!(res_fem_s_tri.converged && res_fem_s_tri.max_displacement > 0.0);
        let res_fem_h_tri = run_harmonic_simulation(HarmonicSimulationRequest {
            mesh: mesh_2d_tri,
            reference_material: ref_mat.clone(),
            strategy: strategy_cfg.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        });
        assert!(res_fem_h_tri.converged && res_fem_h_tri.max_amplitude > 0.0);

        // 1.2 Quad4 FEM (Static & Harmonic)
        let res_fem_s_quad = crate::solver::fem_solver::solve_pure_fem(&mesh_2d_quad, &ref_mat, &strategy, 50_000.0);
        assert!(res_fem_s_quad.converged && res_fem_s_quad.max_displacement > 0.0);
        let res_fem_h_quad = run_harmonic_simulation(HarmonicSimulationRequest {
            mesh: mesh_2d_quad.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy_cfg.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        });
        assert!(res_fem_h_quad.converged && res_fem_h_quad.max_amplitude > 0.0);

        // 1.3 Tet4 FEM (Static & Harmonic)
        let res_fem_s_tet = crate::solver::fem_solver::solve_pure_fem(&mesh_3d_tet, &ref_mat, &strategy, 50_000.0);
        assert!(res_fem_s_tet.converged && res_fem_s_tet.max_displacement > 0.0);
        let res_fem_h_tet = run_harmonic_simulation(HarmonicSimulationRequest {
            mesh: mesh_3d_tet.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy_cfg.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        });
        assert!(res_fem_h_tet.converged && res_fem_h_tet.max_amplitude > 0.0);

        // 1.4 Hex8 FEM (Static & Harmonic)
        let res_fem_s_hex = crate::solver::fem_solver::solve_pure_fem(&mesh_3d_hex, &ref_mat, &strategy, 50_000.0);
        assert!(res_fem_s_hex.converged && res_fem_s_hex.max_displacement > 0.0);
        let res_fem_h_hex = run_harmonic_simulation(HarmonicSimulationRequest {
            mesh: mesh_3d_hex.clone(),
            reference_material: ref_mat.clone(),
            strategy: strategy_cfg.clone(),
            frequency_hz: 5.0,
            load_magnitude: 50_000.0,
            max_gmres_iter: 50,
            tolerance: 1e-4,
            use_robin_coupling: None,
            solver_mode: Some(HarmonicSolverMode::PureFEM),
            bem_element_type: None,
            stress_state: None,
        });
        assert!(res_fem_h_hex.converged && res_fem_h_hex.max_amplitude > 0.0);

        // -------------------------------------------------------------
        // 2. Pure BEM: 2D Elements (5 Types) - Static and Harmonic
        // -------------------------------------------------------------
        let bem_2d_types = [
            BemElementType::Constant,
            BemElementType::Linear,
            BemElementType::LinearDoubleNodes,
            BemElementType::Quadratic,
            BemElementType::QuadraticDiscontinuous,
        ];

        for &elem_type in &bem_2d_types {
            // Static 2D Pure BEM
            let res_bem_s = run_simulation(SimulationRequest {
                mesh: mesh_2d_quad.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                solver_mode: SolverMode::PureBEM,
                load_magnitude: 50_000.0,
                max_iterations: 1,
                tolerance: 1e-4,
                hybrid_acceleration: None,
                bem_element_type: Some(elem_type),
                stress_state: None,
            });
            assert!(res_bem_s.converged, "Static 2D Pure BEM failed for {:?}", elem_type);
            assert!(res_bem_s.max_displacement > 0.0, "Displacement must be positive for {:?}", elem_type);

            // Harmonic 2D Pure BEM
            let res_bem_h = run_harmonic_simulation(HarmonicSimulationRequest {
                mesh: mesh_2d_quad.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                frequency_hz: 5.0,
                load_magnitude: 50_000.0,
                max_gmres_iter: 50,
                tolerance: 1e-4,
                use_robin_coupling: None,
                solver_mode: Some(HarmonicSolverMode::PureBEM),
                bem_element_type: Some(elem_type),
                stress_state: None,
            });
            assert!(res_bem_h.converged, "Harmonic 2D Pure BEM failed for {:?}", elem_type);
            assert!(res_bem_h.max_amplitude > 0.0, "Amplitude must be positive for {:?}", elem_type);
        }

        // -------------------------------------------------------------
        // 3. Pure BEM: 3D Elements (Quad4 & Tri3) - Static and Harmonic
        // -------------------------------------------------------------
        let bem_3d_types = [
            BemElementType::Quad4,
            BemElementType::Tri3,
        ];

        for &elem_type in &bem_3d_types {
            // Static 3D Pure BEM
            let res_bem_s_3d = run_simulation(SimulationRequest {
                mesh: mesh_3d_hex.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                solver_mode: SolverMode::PureBEM,
                load_magnitude: 50_000.0,
                max_iterations: 1,
                tolerance: 1e-4,
                hybrid_acceleration: None,
                bem_element_type: Some(elem_type),
                stress_state: None,
            });
            assert!(res_bem_s_3d.converged, "Static 3D Pure BEM failed for {:?}", elem_type);
            assert!(res_bem_s_3d.max_displacement > 0.0, "Displacement must be positive for 3D {:?}", elem_type);

            // Harmonic 3D Pure BEM
            let res_bem_h_3d = run_harmonic_simulation(HarmonicSimulationRequest {
                mesh: mesh_3d_hex.clone(),
                reference_material: ref_mat.clone(),
                strategy: strategy_cfg.clone(),
                frequency_hz: 5.0,
                load_magnitude: 50_000.0,
                max_gmres_iter: 50,
                tolerance: 1e-4,
                use_robin_coupling: None,
                solver_mode: Some(HarmonicSolverMode::PureBEM),
                bem_element_type: Some(elem_type),
                stress_state: None,
            });
            assert!(res_bem_h_3d.converged, "Harmonic 3D Pure BEM failed for {:?}", elem_type);
            assert!(res_bem_h_3d.max_amplitude > 0.0, "Amplitude must be positive for 3D {:?}", elem_type);
        }

        println!("\n=========================================================================");
        println!(">>> 100% COMPLETE MATRIX VERIFICATION: ALL YES (ΝΑΙ)");
        println!("  - Pure FEM Static & Harmonic: Tri3, Quad4, Tet4, Hex8 -> ALL PASSED");
        println!("  - Pure BEM Static & Harmonic: Constant, Linear, Linear DN, Quadratic, Quadratic Disc -> ALL PASSED");
        println!("  - Pure BEM 3D Static & Harmonic: Quad4 Surface, Tri3 Surface -> ALL PASSED");
        println!("=========================================================================\n");
    }

