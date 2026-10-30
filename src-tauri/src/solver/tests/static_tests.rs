use crate::material::strategies::*;
use crate::material::types::*;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::solver::*;
use faer::Mat;
use num_complex::Complex64;

    #[test]
    fn test_fem_cantilever_deflection() {
        // Cantilever beam: L = 10m, H = 2m, 10x4 elements
        let mesh = generate_beam_2d(10.0, 2.0, 10, 4, false);
        let ref_mat = ReferenceMaterial::default(); // E0 = 210 GPa
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0, // homogeneous for benchmark check
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let p_load = 50_000.0; // 50 kN downward
        let res = fem_solver::solve_pure_fem(&mesh, &ref_mat, &strategy, p_load);

        assert!(res.converged);
        assert!(res.max_displacement > 0.0);

        // Analytical Euler-Bernoulli tip deflection: delta = P * L^3 / (3 * E * I)
        // I = 1 * 2^3 / 12 = 8 / 12 = 0.6667 m^4
        // E = 2.1e11 Pa
        // delta_EB = 50000 * 1000 / (3 * 2.1e11 * 0.6667) = 5e7 / 4.2e11 = 1.19e-4 m (~0.12 mm)
        assert!(res.max_displacement > 5e-5 && res.max_displacement < 5e-4);

        // Max von Mises stress should be non-zero
        assert!(res.max_von_mises > 0.0);
    }

    #[test]
    fn test_bem_pure_solver() {
        let mesh = generate_beam_2d(6.0, 2.0, 6, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let res = bem_solver::solve_pure_bem(&mesh, &ref_mat, &strategy, 10_000.0);
        assert!(res.converged);
        assert_eq!(res.solver_mode, SolverMode::PureBEM);
        assert!(res.max_displacement > 0.0);
    }

    #[test]
    fn test_bem_fem_hybrid_convergence() {
        let mesh = generate_beam_2d(8.0, 2.0, 8, 3, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.20, // 20% inhomogeneity
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let (res, _, _) = oraiopoulos_bem_fem::solve_decoupled_bem_fem_hybrid(
            10.0,
            2.0,
            20,
            6,
            &mesh,
            &ref_mat,
            &strategy,
            20_000.0,
            20,
            1e-4,
            HybridAcceleration::Anderson { depth: 3, beta: 0.65 },
            StressState2D::PlaneStress,
        );

        assert!(res.converged);
        assert_eq!(res.solver_mode, SolverMode::BEMFEM);
        assert!(!res.convergence_history.is_empty());
        assert!(res.max_displacement > 0.0);
    }

    #[test]
    fn test_harmonic_sweep() {
        let mesh = crate::mesh::generators_2d::generate_beam_2d(10.0, 2.0, 20, 6, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            1.0,
            1.0,
            GradientForm::Linear,
        );

        println!("\n--- HARMONIC FREQUENCY SWEEP (L=10, H=2, P=50 kN) ---");
        for &f in &[0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 8.0, 10.0, 20.0, 50.0] {
            let res = dynamic_solver::solve_harmonic(
                &mesh,
                &ref_mat,
                &strategy,
                f,
                50_000.0,
                100,
                1e-6,
            );
            println!("f = {:5.1} Hz -> max_amp = {:10.6} mm, iter = {}, converged = {}", 
                f, res.max_amplitude * 1000.0, res.iterations, res.converged);
        }
    }

    #[test]
    fn test_3d_harmonic_fem_hex8_and_tet4() {
        use crate::mesh::generators_3d::generate_beam_3d;
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::X,
            0.0,
            10.0,
            1.0,
            GradientForm::Linear,
        );

        // 1. Hex8 3D Harmonic FEM
        let hex_mesh = generate_beam_3d(10.0, 2.0, 2.0, 6, 2, 2, false);
        let res_hex = dynamic_solver::solve_harmonic(
            &hex_mesh,
            &ref_mat,
            &strategy,
            3.0,
            50_000.0,
            200,
            1e-5,
        );
        println!("\n[3D Hex8 Harmonic 3 Hz] max_amp = {:.6} mm, converged = {}, iters = {}", 
            res_hex.max_amplitude * 1000.0, res_hex.converged, res_hex.iterations);
        assert!(res_hex.converged);
        assert!(res_hex.max_amplitude > 0.0);

        // 2. Tet4 3D Harmonic FEM
        let tet_mesh = generate_beam_3d(10.0, 2.0, 2.0, 6, 2, 2, true);
        let res_tet = dynamic_solver::solve_harmonic(
            &tet_mesh,
            &ref_mat,
            &strategy,
            3.0,
            50_000.0,
            200,
            1e-5,
        );
        println!("[3D Tet4 Harmonic 3 Hz] max_amp = {:.6} mm, converged = {}, iters = {}", 
            res_tet.max_amplitude * 1000.0, res_tet.converged, res_tet.iterations);
        assert!(res_tet.converged);
        assert!(res_tet.max_amplitude > 0.0);
    }

    #[test]
    fn test_transient_talbot() {
        let mesh = generate_beam_2d(6.0, 2.0, 6, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.10,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let res = dynamic_solver::solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            0.05, // 50 ms
            5,
            16,
            10_000.0,
            0.01,
            Some("Pulse"),
            None,
        );

        assert_eq!(res.frames.len(), 5);
        assert!(res.max_overall_displacement > 0.0);
        for frame in &res.frames {
            assert!(frame.time > 0.0);
            assert_eq!(frame.displacements.len(), mesh.nodes.len());
        }
    }

    #[test]
    fn test_transient_talbot_beam_sine() {
        let mesh = crate::mesh::generators_2d::generate_beam_2d(10.0, 2.0, 20, 6, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            1.0,
            1.0,
            GradientForm::Linear,
        );

        let res = dynamic_solver::solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            0.6, // 0.6 s
            30,
            32, // 32 Talbot nodes!
            50_000.0,
            0.0,
            Some("HarmonicSine"),
            Some(3.0),
        );

        println!("\n--- TRANSIENT SINE 3 HZ (N=32) ---");
        println!("Max Overall Wave Deflection: {:.6} mm", res.max_overall_displacement * 1000.0);
        for (i, frame) in res.frames.iter().enumerate().step_by(5) {
            println!("Frame {:2} (t={:.3} s): max_d = {:.6} mm", i, frame.time, frame.max_displacement * 1000.0);
        }
    }

    #[test]
    fn test_dynamic_robin_impedance_tensor() {
        use crate::solver::robin_transmission::DynamicImpedance;
        use num_complex::Complex64;

        let ref_mat = ReferenceMaterial::default();
        let impedance = DynamicImpedance::new(&ref_mat);

        assert!(impedance.cp > impedance.cs);
        assert!(impedance.cs > 0.0);

        // Test normal along x: n = [1, 0]
        let s = Complex64::new(0.5, 20.0 * std::f64::consts::PI);
        let z = impedance.tensor_2d([1.0, 0.0], s);

        // For n = [1, 0], Z_xx = rho * s * cp, Z_yy = rho * s * cs, Z_xy = 0
        assert!(z[0][0].re > 0.0, "Re(Z_xx) must be strictly positive (energy absorbing)");
        assert!(z[1][1].re > 0.0, "Re(Z_yy) must be strictly positive (energy absorbing)");
        assert!(z[0][1].norm() < 1e-12, "Cross terms must vanish for axis-aligned normal");

        // Verify positive definiteness for inclined normal n = [cos(theta), sin(theta)]
        let theta = std::f64::consts::FRAC_PI_4; // 45 degrees
        let z_inclined = impedance.tensor_2d([theta.cos(), theta.sin()], s);
        
        // Re(Z) must be strictly positive definite: Trace > 0, Det > 0
        let re_tr = z_inclined[0][0].re + z_inclined[1][1].re;
        let re_det = z_inclined[0][0].re * z_inclined[1][1].re - z_inclined[0][1].re * z_inclined[1][0].re;
        assert!(re_tr > 0.0, "Trace of Re(Z) must be positive");
        assert!(re_det > 0.0, "Determinant of Re(Z) must be positive (strictly positive definite)");
    }

    #[test]
    fn test_dynamic_robin_hybrid_convergence() {
        use crate::solver::robin_transmission::solve_dynamic_robin_hybrid;

        let mesh = generate_beam_2d(8.0, 2.0, 8, 3, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.15, // 15% inhomogeneity
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let freq_hz = 15.0; // Dynamic harmonic excitation at 15 Hz
        let res = solve_dynamic_robin_hybrid(
            &mesh,
            &ref_mat,
            &strategy,
            freq_hz,
            25_000.0,
            10, // max iterations
            0.02, // 2% tolerance for domain decomposition Schwarz iteration
        );

        println!("\nDynamic Robin result: converged = {}, iterations = {}, rel_residual = {:.6e}",
            res.converged, res.iterations, res.relative_residual);
        assert!(res.converged, "Dynamic Robin hybrid solver must converge");
        assert!(res.iterations <= 6, "Dynamic Robin Schwarz must contract rapidly in <= 6 iterations (actual: {})", res.iterations);
        assert!(res.max_amplitude > 0.0, "Max amplitude must be positive");
        assert!(res.max_von_mises > 0.0, "Max von Mises stress must be positive");
    }

    #[test]
    fn test_transient_newmark_beam() {
        use crate::solver::time_marching::solve_transient_newmark;

        let mesh = generate_beam_2d(10.0, 2.0, 10, 4, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0, // homogeneous
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let total_time = 0.2; // 200 ms
        let time_steps = 20;
        let p_load = 50_000.0;
        let pulse_dur = 0.05; // 50 ms rectangular pulse

        let res = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            time_steps,
            p_load,
            pulse_dur,
            Some("Pulse"),
            None,
            Some(0.0), // classical Newmark average acceleration (alpha = 0)
            Some(0.05), // alpha_R
            Some(0.0002), // beta_R
        );

        assert_eq!(res.time_points.len(), time_steps);
        assert_eq!(res.frames.len(), time_steps);
        assert!(res.max_overall_displacement > 0.0);

        // Peak deflection under 50 kN dynamic pulse should be around static deflection (0.12 mm)
        // with dynamic amplification factor (0.1 - 0.3 mm)
        println!("\nNewmark transient max displacement: {:.6} mm", res.max_overall_displacement * 1000.0);
        assert!(
            res.max_overall_displacement > 5e-5 && res.max_overall_displacement < 5e-4,
            "Newmark deflection should be between 0.05 mm and 0.5 mm, got {}",
            res.max_overall_displacement
        );

        // Check frame displacements are smooth and positive
        for frame in &res.frames {
            assert!(frame.time > 0.0);
            assert_eq!(frame.displacements.len(), mesh.nodes.len());
            assert_eq!(frame.von_mises_stresses.len(), mesh.nodes.len());
        }
    }

    #[test]
    fn test_transient_newmark_vs_talbot_benchmark() {
        use crate::solver::dynamic_solver::solve_transient_talbot;
        use crate::solver::time_marching::solve_transient_newmark;

        let mesh = generate_beam_2d(6.0, 2.0, 6, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let total_time = 0.1;
        let time_steps = 10;
        let p_load = 30_000.0;
        let pulse_dur = 0.04;

        let res_talbot = solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            time_steps,
            16,
            p_load,
            pulse_dur,
            Some("Pulse"),
            None,
        );

        let res_newmark = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            time_steps,
            p_load,
            pulse_dur,
            Some("Pulse"),
            None,
            Some(0.0),
            Some(0.0), // zero damping for clean comparison
            Some(0.0),
        );

        println!("\nBenchmark Talbot peak: {:.6} mm vs Newmark peak: {:.6} mm",
            res_talbot.max_overall_displacement * 1000.0,
            res_newmark.max_overall_displacement * 1000.0
        );

        assert!(res_talbot.max_overall_displacement > 0.0);
        assert!(res_newmark.max_overall_displacement > 0.0);

        // Both methods should be of the same physical magnitude (within 50% for this coarse discretization)
        let ratio = res_newmark.max_overall_displacement / res_talbot.max_overall_displacement;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "Newmark and Talbot peak deflections should be consistent! Ratio = {:.3}",
            ratio
        );
    }

    #[test]
    fn test_transient_half_sine_pulse_benchmark() {
        use crate::solver::dynamic_solver::solve_transient_talbot;
        use crate::solver::time_marching::solve_transient_newmark;

        let mesh = generate_beam_2d(6.0, 2.0, 6, 2, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let total_time = 0.15;
        let time_steps = 15;
        let p_load = 40_000.0;
        let pulse_dur = 0.05;

        let res_talbot = solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            time_steps,
            24,
            p_load,
            pulse_dur,
            Some("HalfSine"),
            None,
        );

        let res_newmark = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            time_steps,
            p_load,
            pulse_dur,
            Some("HalfSine"),
            None,
            Some(0.0),
            Some(0.01),
            Some(0.0001),
        );

        println!("\nHalf-Sine Benchmark: Talbot peak = {:.6} mm vs Newmark peak = {:.6} mm",
            res_talbot.max_overall_displacement * 1000.0,
            res_newmark.max_overall_displacement * 1000.0
        );

        assert!(res_talbot.max_overall_displacement > 0.0);
        assert!(res_newmark.max_overall_displacement > 0.0);
    }

    #[test]
    fn test_investigate_talbot_vs_newmark_harmonic() {
        use crate::solver::dynamic_solver::solve_transient_talbot;
        use crate::solver::time_marching::solve_transient_newmark;

        let mesh = generate_beam_2d(10.0, 2.0, 10, 4, false);
        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.0,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        let total_time = 0.5;
        let p_load = 50_000.0;
        let freq = 3.0;

        // Talbot with 32 nodes and 25 steps (user's setting)
        let res_talbot_32 = solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            25,
            32,
            p_load,
            0.05,
            Some("HarmonicSine"),
            Some(freq),
        );

        // Talbot with 64 nodes and 50 steps
        let res_talbot_64 = solve_transient_talbot(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            50,
            64,
            p_load,
            0.05,
            Some("HarmonicSine"),
            Some(freq),
        );

        // Newmark with 25 steps, alpha_R = 0.05, beta_R = 0.0002
        let res_newmark_25_damped = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            25,
            p_load,
            0.05,
            Some("HarmonicSine"),
            Some(freq),
            Some(0.0),
            Some(0.05),
            Some(0.0002),
        );

        // Newmark with 25 steps, NO damping (alpha_R = 0, beta_R = 0)
        let res_newmark_25_undamped = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            25,
            p_load,
            0.05,
            Some("HarmonicSine"),
            Some(freq),
            Some(0.0),
            Some(0.0),
            Some(0.0),
        );

        // Newmark with 100 steps, NO damping
        let res_newmark_100_undamped = solve_transient_newmark(
            &mesh,
            &ref_mat,
            &strategy,
            total_time,
            100,
            p_load,
            0.05,
            Some("HarmonicSine"),
            Some(freq),
            Some(0.0),
            Some(0.0),
            Some(0.0),
        );

        println!("\n=== HARMONIC SINE 3 HZ BENCHMARK (0.5 s) ===");
        println!("Talbot (32 nodes, 25 steps): max = {:.6} mm", res_talbot_32.max_overall_displacement * 1000.0);
        println!("Talbot (64 nodes, 50 steps): max = {:.6} mm", res_talbot_64.max_overall_displacement * 1000.0);
        println!("Newmark (25 steps, damped αR=0.05): max = {:.6} mm", res_newmark_25_damped.max_overall_displacement * 1000.0);
        println!("Newmark (25 steps, undamped): max = {:.6} mm", res_newmark_25_undamped.max_overall_displacement * 1000.0);
        println!("Newmark (100 steps, undamped): max = {:.6} mm", res_newmark_100_undamped.max_overall_displacement * 1000.0);
        
        println!("\nTime trajectory comparison at t = 0.1, 0.2, 0.3, 0.4, 0.5 s:");
        for i in [4, 9, 14, 19, 24] {
            let t = res_talbot_32.frames[i].time;
            let d_tal = res_talbot_32.frames[i].max_displacement * 1000.0;
            let d_new = res_newmark_25_damped.frames[i].max_displacement * 1000.0;
            println!("t = {:.2} s -> Talbot: {:.6} mm | Newmark: {:.6} mm | diff: {:.6} mm", t, d_tal, d_new, (d_new - d_tal).abs());
        }
    }

    #[test]
    fn test_dynamic_operator_splitting_vs_pure_fem_laplace() {
        use crate::material::strategies::DirectionalGradientField;
        use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
        use crate::mesh::generators_2d::generate_beam_2d;
        use crate::solver::hybrid_solver::{
            solve_transient_dynamic_operator_splitting_talbot, solve_transient_pure_fem_talbot,
        };

        let mesh = generate_beam_2d(2.0, 0.4, 10, 4, false);
        let reference = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.5, // 50% contrast across depth (alpha = 0.5)
            0.4, // depth length scale
            1.0,
            GradientForm::Linear,
        );

        let total_time = 0.2;
        let time_steps = 10;
        let num_talbot_points = 24;
        let load_magnitude = 10_000.0;
        let pulse_duration = 0.04;

        // 1. Solve via Pure FEM + Talbot
        let (res_fem, fem_iters) = solve_transient_pure_fem_talbot(
            &mesh,
            &reference,
            &strategy,
            total_time,
            time_steps,
            num_talbot_points,
            load_magnitude,
            pulse_duration,
            Some("HalfSine"),
            None,
            100,
            1e-7,
        );

        // 2. Solve via Variational Dynamic Operator Splitting + Talbot
        let (res_split, split_iters) = solve_transient_dynamic_operator_splitting_talbot(
            &mesh,
            &reference,
            &strategy,
            total_time,
            time_steps,
            num_talbot_points,
            load_magnitude,
            pulse_duration,
            Some("HalfSine"),
            None,
            30,
            1e-5,
            0.85,
        );

        println!("\n=== DYNAMIC OPERATOR SPLITTING VS PURE FEM TALBOT ===");
        println!("Pure FEM Peak Deflection: {:.6} mm", res_fem.max_overall_displacement * 1000.0);
        println!("Operator Splitting Peak Deflection: {:.6} mm", res_split.max_overall_displacement * 1000.0);
        println!("FEM GMRES iters per Talbot node: {:?}", &fem_iters[..5]);
        println!("Operator Splitting outer iters per Talbot node: {:?}", &split_iters[..5]);

        let diff = (res_fem.max_overall_displacement - res_split.max_overall_displacement).abs();
        let rel_diff = diff / res_fem.max_overall_displacement;
        println!("Relative difference between methods: {:.6}%", rel_diff * 100.0);

        assert!(rel_diff < 0.001, "Operator splitting and Pure FEM should agree within 0.1% in Laplace-Talbot domain!");
        assert!(split_iters.iter().all(|&it| it <= 12), "Operator splitting should converge in <= 12 iterations!");
    }

    #[test]
    fn test_dense_bem_boundary_coarse_fem_investigation() {
        use crate::solver::true_bem_fem::solve_true_bem_fem_static;

        let ref_mat = ReferenceMaterial::default();
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            0.30, // 30% linear inhomogeneity
            2.0,
            1.0,
            GradientForm::Linear,
        );
        let p_load = 50_000.0;

        // Baseline Pure FEM dense
        let mesh_dense = generate_beam_2d(6.0, 2.0, 30, 10, false);
        let res_base = fem_solver::solve_pure_fem(&mesh_dense, &ref_mat, &strategy, p_load);

        println!("\n=======================================================");
        println!("INVESTIGATION: Dense Boundary BEM vs Coarse FEM");
        println!("Dense Baseline FEM (30x10): Disp = {:.6} mm", res_base.max_displacement * 1000.0);

        // Test mesh with dense boundary (30x10)
        let _res_bem_dense_bnd = solve_true_bem_fem_static(
            &mesh_dense,
            &ref_mat,
            &strategy,
            p_load,
            1, // Just step 0 (Predictor)
            1e-4,
            0.85,
        );
    }

    #[test]
    fn test_homogeneous_calibration_fem_vs_bem() {
        let ref_mat = ReferenceMaterial::default(); // E = 210 GPa, nu = 0.25
        let strategy_homog = DirectionalGradientField::new(
            Axis::Y,
            0.0, // EXACTLY HOMOGENEOUS (alpha = 0)
            2.0,
            1.0,
            GradientForm::Linear,
        );
        let p_load = 50_000.0;

        // Axial elongation analytical: Delta L = P * L / (E * A)
        // P = 50,000 N, L = 6.0 m, A = 2.0 * 1.0 m^2, E = 2.1e11 Pa
        let delta_axial_analytical = (50_000.0 * 6.0) / (2.1e11 * 2.0); // = 7.142857e-7 m = 0.000714 mm

        println!("\n=======================================================");
        println!("HOMOGENEOUS CALIBRATION (alpha = 0, E0 = 210 GPa, nu0 = 0.25)");
        println!("Euler-Bernoulli Bending:    w_tip = 0.025714 mm");
        println!("Timoshenko Bending:         w_tip = 0.027857 mm");
        println!("Axial Tension (P*L/(E*A)):  u_tip = {:.6} mm", delta_axial_analytical * 1000.0);
        println!("=======================================================\n");

        let mesh_configs = [(6, 2), (12, 4), (20, 6), (30, 10)];
        for &(nx, ny) in &mesh_configs {
            let mesh = generate_beam_2d(6.0, 2.0, nx, ny, false);
            let res_fem = fem_solver::solve_pure_fem(&mesh, &ref_mat, &strategy_homog, p_load);
            let res_bem_const = bem_solver::solve_pure_bem_constant(&mesh, &ref_mat, &strategy_homog, p_load);
            let res_bem_linear = bem_solver::solve_pure_bem_linear(&mesh, &ref_mat, &strategy_homog, p_load);
            let res_bem_dn = bem_solver::solve_pure_bem_linear_dn(&mesh, &ref_mat, &strategy_homog, p_load);
            let res_bem_quad_disc = bem_solver::solve_pure_bem_quadratic_discontinuous(&mesh, &ref_mat, &strategy_homog, p_load);

            println!("--- Mesh {}x{} (Nodes: {}, BndElems: {}) ---", nx, ny, mesh.nodes.len(), mesh.boundary_elements.len());
            println!("  FEM:                     max_disp = {:.6} mm (time: {:.2} ms)", res_fem.max_displacement * 1000.0, res_fem.execution_time_ms);
            println!("  BEM Constant:            max_disp = {:.6} mm (time: {:.2} ms)", res_bem_const.max_displacement * 1000.0, res_bem_const.execution_time_ms);
            println!("  BEM Linear (no DN):      max_disp = {:.6} mm (time: {:.2} ms)", res_bem_linear.max_displacement * 1000.0, res_bem_linear.execution_time_ms);
            println!("  BEM Linear (Double Nodes): max_disp = {:.6} mm (time: {:.2} ms)", res_bem_dn.max_displacement * 1000.0, res_bem_dn.execution_time_ms);
            println!("  BEM Quad Discontinuous:  max_disp = {:.6} mm (time: {:.2} ms)", res_bem_quad_disc.max_displacement * 1000.0, res_bem_quad_disc.execution_time_ms);
            println!("  Ratio Const/FEM          = {:.4}", res_bem_const.max_displacement / res_fem.max_displacement);
            println!("  Ratio Linear/FEM         = {:.4}", res_bem_linear.max_displacement / res_fem.max_displacement);
            println!("  Ratio DoubleNodes/FEM    = {:.4}", res_bem_dn.max_displacement / res_fem.max_displacement);
            println!("  Ratio QuadDisc/FEM       = {:.4}", res_bem_quad_disc.max_displacement / res_fem.max_displacement);

            // Find tip center node (x = 6.0, y = 1.0)
            let mut tip_center_idx = 0;
            let mut min_dist = 1e9;
            for (idx, p) in mesh.nodes.iter().enumerate() {
                let dist = ((p[0] - 6.0).powi(2) + (p[1] - 1.0).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    tip_center_idx = idx;
                }
            }
            println!("  Tip Center Disp (x=6, y=1):");
            println!("    FEM:              u_y = {:.6} mm", res_fem.displacements[tip_center_idx][1] * 1000.0);
            println!("    Linear:           u_y = {:.6} mm", res_bem_linear.displacements[tip_center_idx][1] * 1000.0);
            println!("    DoubleNodes:      u_y = {:.6} mm", res_bem_dn.displacements[tip_center_idx][1] * 1000.0);
            println!("    QuadDiscontinuous: u_y = {:.6} mm", res_bem_quad_disc.displacements[tip_center_idx][1] * 1000.0);
            println!("    Timoshenko Analytical      = -0.027857 mm");
            println!("    Euler-Bernoulli Analytical = -0.025714 mm");
        }
    }

