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
    #[ignore]
    pub fn test_execute_and_export_all_mp04_benchmarks() {
        println!(">>> Starting Mini-Paper 4 Dynamic Operator Splitting Benchmark Suite <<<");
        let ref_mat = ReferenceMaterial::default();
        let p_load = 50_000.0;
        let alpha = 0.5; // 50% material gradient

        // 1. Sweep A: Broadband Transient Half-Sine Shock Pulse Comparison
        println!("Running Sweep A: Broadband Transient Half-Sine Shock Pulse (16x4 Quad4)...");
        let mesh_a = generate_beam_2d(10.0, 2.0, 16, 4, false);
        let (records_a, fem_cpu, split_cpu, avg_fem_it, avg_split_it) = run_shock_pulse_sweep(
            &mesh_a,
            &ref_mat,
            alpha,
            0.2,
            40,
            24,
            p_load,
            0.04,
        );
        let csv_path_a = Path::new("../docs/benchmarks/mp04_shock_pulse_fem_vs_splitting.csv");
        export_shock_pulse_to_csv(&records_a, csv_path_a).expect("Failed to export Sweep A CSV");
        println!(
            "Sweep A Completed: Pure FEM CPU = {:.1} ms (avg iters: {:.1}) | Operator Splitting CPU = {:.1} ms (avg iters: {:.1}) | Exported {} records to {:?}",
            fem_cpu, avg_fem_it, split_cpu, avg_split_it, records_a.len(), csv_path_a
        );

        // 2. Sweep B: Convergence across Material Contrast in Complex Laplace Domain
        println!("Running Sweep B: Contrast Convergence in Complex Laplace Domain...");
        let mesh_b = generate_beam_2d(10.0, 2.0, 16, 4, false);
        let contrasts = [-0.7, -0.5, -0.3, 0.0, 0.3, 0.5, 0.8, 1.0, 1.5, 2.0];
        let records_b = run_contrast_convergence_sweep(&mesh_b, &ref_mat, &contrasts, 0.2, p_load);
        let csv_path_b = Path::new("../docs/benchmarks/mp04_contrast_convergence_laplace.csv");
        export_contrast_to_csv(&records_b, csv_path_b).expect("Failed to export Sweep B CSV");
        println!("Sweep B Completed: Exported {} records to {:?}", records_b.len(), csv_path_b);

        // 3. Sweep C: Complexity Scaling vs DOFs
        println!("Running Sweep C: Complexity Scaling vs DOFs...");
        let resolutions = [
            (8, 2, "8x2"),
            (16, 4, "16x4"),
            (32, 8, "32x8"),
            (64, 16, "64x16"),
        ];
        let records_c = run_dofs_scaling_sweep(
            &ref_mat,
            alpha,
            &resolutions,
            0.1,
            10,
            24,
            p_load,
        );
        let csv_path_c = Path::new("../docs/benchmarks/mp04_runtime_scaling_dofs.csv");
        export_scaling_to_csv(&records_c, csv_path_c).expect("Failed to export Sweep C CSV");
        println!("Sweep C Completed: Exported {} records to {:?}", records_c.len(), csv_path_c);

        println!(">>> MP-04 Benchmark Suite Completed Successfully! <<<");
    }
