use super::*;
use crate::material::strategies::*;
use crate::material::traits::*;
use crate::material::types::*;
use crate::material::ReferenceMaterial;
use crate::mesh::generators_2d::*;
use crate::mesh::generators_3d::*;
use crate::mesh::types::*;
use crate::solver::*;
use std::path::Path;
use std::fs::{create_dir_all, File};
use std::io::Write;


#[test]
    #[ignore = "long-running benchmark suite, run explicitly with cargo test --release -- --ignored"]
    fn test_execute_and_export_all_mp02_scaling_benchmarks() {
        println!(">>> Starting Mini-Paper 2 Computational Complexity & Scaling Benchmark Suite <<<");

        let alpha = 0.20; // Mild gradient (fast, stable 5-6 iters)
        let omega = 1.00;
        let load_mag = 1000.0;
        let max_iters = 50;
        let tol = 1e-4;

        // 1. 2D Progressive Refinement Sweep
        let divisions_2d = vec![
            (10, 2),    // 33 nodes, 66 DOFs
            (20, 4),    // 105 nodes, 210 DOFs
            (40, 8),    // 369 nodes, 738 DOFs
            (80, 16),   // 1,377 nodes, 2,754 DOFs
            (120, 24),  // 3,025 nodes, 6,050 DOFs
            (160, 32),  // 5,313 nodes, 10,626 DOFs
            (200, 40),  // 8,241 nodes, 16,482 DOFs
        ];

        let records_2d = run_2d_scaling_sweep(
            &divisions_2d,
            alpha,
            omega,
            load_mag,
            max_iters,
            tol,
        );

        let csv_path_2d = Path::new("../docs/benchmarks/mp02_complexity_scaling_2d.csv");
        export_scaling_records_to_csv(&records_2d, csv_path_2d)
            .expect("Failed to export MP-02 2D scaling CSV");
        println!("Exported {} 2D records to {:?}", records_2d.len(), csv_path_2d);

        // 2. 3D Progressive Refinement Sweep
        let divisions_3d = vec![
            (6, 2, 2),     // 63 nodes, 189 DOFs
            (10, 3, 3),    // 176 nodes, 528 DOFs
            (16, 4, 4),    // 425 nodes, 1,275 DOFs
            (20, 5, 5),    // 756 nodes, 2,268 DOFs
            (26, 6, 6),    // 1,323 nodes, 3,969 DOFs
            (32, 8, 8),    // 2,673 nodes, 8,019 DOFs
            (40, 10, 10),  // 4,961 nodes, 14,883 DOFs
            (50, 12, 12),  // 8,619 nodes, 25,857 DOFs
            (60, 15, 15),  // 15,616 nodes, 46,848 DOFs
            (70, 18, 18),  // 25,631 nodes, 76,893 DOFs
        ];

        let records_3d = run_3d_scaling_sweep(
            &divisions_3d,
            alpha,
            omega,
            load_mag,
            max_iters,
            tol,
        );

        let csv_path_3d = Path::new("../docs/benchmarks/mp02_complexity_scaling_3d.csv");
        export_scaling_records_to_csv(&records_3d, csv_path_3d)
            .expect("Failed to export MP-02 3D scaling CSV");
        println!("Exported {} 3D records to {:?}", records_3d.len(), csv_path_3d);

        // 3. Multi-Step Reanalysis Sweep (5 load increments in 3D)
        let multistep_divisions = vec![
            (16, 4, 4),    // 1,275 DOFs
            (26, 6, 6),    // 3,969 DOFs
            (32, 8, 8),    // 8,019 DOFs
            (40, 10, 10),  // 14,883 DOFs
            (50, 12, 12),  // 25,857 DOFs
        ];
        let multistep_records = run_multistep_reanalysis_sweep(&multistep_divisions, 5, alpha, omega);
        let csv_path_multi = Path::new("../docs/benchmarks/mp02_multistep_reanalysis.csv");
        export_multistep_records_to_csv(&multistep_records, csv_path_multi)
            .expect("Failed to export MP-02 Multistep CSV");
        println!("Exported {} Multi-Step records to {:?}", multistep_records.len(), csv_path_multi);

        assert!(!records_2d.is_empty());
        assert!(!records_3d.is_empty());
        assert!(!multistep_records.is_empty());
        println!(">>> MP-02 Scaling Benchmark Suite Completed Successfully! <<<");
    }

    #[test]
    fn test_generate_real_paper_table_data() {
        use crate::solver::oraiopoulos_bem_fem::solve_oraiopoulos_bem_fem_3d;

        let length = 10.0;
        let height = 2.0;
        let depth = 2.0;
        let load_mag = 1000.0;
        let ref_mat = ReferenceMaterial {
            e0: 30.0e9,
            nu0: 0.20,
            rho0: 2400.0,
        };

        // Analytical Timoshenko tip deflection:
        // I = W * H^3 / 12 = 2 * 8 / 12 = 1.3333 m^4
        // E = 30 GPa
        // G = E / (2 * (1 + nu)) = 12.5 GPa
        // As = 5/6 * (W * H) = 3.3333 m^2
        // w_bend = P * L^3 / (3 * E * I) = 1000 * 1000 / (3 * 30e9 * 1.3333) = 8.3333e-6 m
        // w_shear = P * L / (G * As) = 1000 * 10 / (12.5e9 * 3.3333) = 0.2400e-6 m
        // w_exact = 8.5733e-6 m (0.008573 mm)
        let w_exact = 8.5733e-6;

        let divisions = [
            (6, 2, 2),
            (10, 3, 3),
            (16, 4, 4),
            (20, 5, 5),
            (26, 6, 6),
            (32, 8, 8),
        ];

        println!("\n==========================================================================================");
        println!("REAL BENCHMARK DATA FOR PAPER 2 TABLE (alpha = 0.00, w_exact = {:.6} mm)", w_exact * 1000.0);
        println!("==========================================================================================");

        let strat_0 = DirectionalGradientField::new(
            Axis::X,
            0.0,
            length,
            1.0,
            GradientForm::Linear,
        );

        for &(nx, ny, nz) in &divisions {
            let mesh = generate_beam_3d(length, height, depth, nx, ny, nz, false);
            let num_nodes = mesh.nodes.len();
            let total_dofs = num_nodes * 3;
            let num_bnd_quads = 2 * (ny * nz + nx * nz + nx * ny);
            let bnd_dofs = num_bnd_quads * 3;
            let tip_idx = find_tip_node_3d(&mesh, length, height, depth);

            // 1. Pure FEM
            let fem_start = std::time::Instant::now();
            let fem_res = solve_pure_fem(&mesh, &ref_mat, &strat_0, load_mag);
            let fem_ms = fem_start.elapsed().as_secs_f64() * 1000.0;
            let w_fem = fem_res.displacements[tip_idx][1].abs();
            let err_fem = ((w_fem - w_exact).abs() / w_exact) * 100.0;

            // 2. Oraiopoulos BEM-FEM
            let (bemfem_res, pred_ms, corr_ms) = solve_oraiopoulos_bem_fem_3d(
                length,
                height,
                depth,
                nx, ny, nz,
                &mesh,
                &ref_mat,
                &strat_0,
                load_mag,
                50,
                1e-5,
                HybridAcceleration::Anderson { depth: 4, beta: 0.5 },
            );
            let w_bemfem = bemfem_res.displacements[tip_idx][1].abs();
            let err_bemfem = ((w_bemfem - w_exact).abs() / w_exact) * 100.0;
            let iters = bemfem_res.iterations;
            let speedup = if bemfem_res.execution_time_ms > 0.0 {
                fem_ms / bemfem_res.execution_time_ms
            } else {
                1.0
            };

            println!(
                "Mesh {:>7} | Hex8={:>4} | DOFs={:>5} | BEM DOFs={:>4} | FEM: {:>6.2} ms, w={:.6} mm, err={:>5.1}% | BEM-FEM: {:>6.2} ms (pred={:>6.2}ms, corr={:>5.2}ms), w={:.6} mm, err={:>5.2}%, it={} | Spd: {:>4.2}x",
                format!("{}x{}x{}", nx, ny, nz),
                mesh.volume_elements.len(),
                total_dofs,
                bnd_dofs,
                fem_ms,
                w_fem * 1000.0,
                err_fem,
                bemfem_res.execution_time_ms,
                pred_ms,
                corr_ms,
                w_bemfem * 1000.0,
                err_bemfem,
                iters,
                speedup
            );
        }
    }
