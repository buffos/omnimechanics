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
    pub fn test_execute_and_export_all_mp05_half_space_benchmarks() {
        println!(">>> Starting Mini-Paper 5: Inhomogeneous Elastic Half-Space Benchmark Suite <<<");
        let ref_mat = ReferenceMaterial::default();
        let total_time = 0.20;
        let time_steps = 25;
        let talbot_nodes = 24;
        let footing_load = 100_000.0;
        let footing_width = 4.0;
        let pulse_dur = 0.05;
        let alpha = 0.50; // 50% increase in soil stiffness with depth

        // 1. Establish High-Resolution Ground Truth Baseline (80x40 mesh, 3321 nodes, 6642 DOFs)
        println!("Establishing Ground Truth Baseline (80x40 Overkill Mesh)...");
        let mesh_baseline = generate_beam_2d(40.0, 20.0, 80, 40, false);
        let (times, baseline_disp, baseline_cpu, _) = solve_half_space_pure_fem(
            &mesh_baseline,
            &ref_mat,
            alpha,
            total_time,
            time_steps,
            talbot_nodes,
            footing_load,
            footing_width,
            pulse_dur,
        );
        println!(
            "Baseline Established: 6642 DOFs, Peak Disp = {:.6} mm, Runtime = {:.1} ms",
            baseline_disp.iter().cloned().fold(0.0, f64::max),
            baseline_cpu
        );

        // 2. Discretization Sweep: Pure FEM vs BEM-FEM Operator Splitting
        let resolutions = [
            (10, 5, "10x5"),
            (20, 10, "20x10"),
            (40, 20, "40x20"),
            (60, 30, "60x30"),
        ];

        let mut disc_records = Vec::new();
        let mut traj_coarse_fem = Vec::new();
        let mut traj_fine_fem = Vec::new();
        let mut traj_coarse_split = Vec::new();
        let mut traj_fine_split = Vec::new();

        for (idx, &(nx, ny, label)) in resolutions.iter().enumerate() {
            println!("Testing Mesh Resolution: {}...", label);
            let mesh = generate_beam_2d(40.0, 20.0, nx, ny, false);
            let num_nodes = mesh.nodes.len();
            let total_dofs = num_nodes * 2;

            // Pure FEM
            let (_, fem_disp, fem_cpu, fem_iters) = solve_half_space_pure_fem(
                &mesh,
                &ref_mat,
                alpha,
                total_time,
                time_steps,
                talbot_nodes,
                footing_load,
                footing_width,
                pulse_dur,
            );
            let fem_err = compute_l2_relative_error(&fem_disp, &baseline_disp);
            let avg_fem_it = fem_iters.iter().sum::<usize>() as f64 / fem_iters.len() as f64;

            // BEM-FEM Operator Splitting
            let (_, split_disp, split_cpu, split_iters) = solve_half_space_operator_splitting(
                &mesh,
                &ref_mat,
                alpha,
                total_time,
                time_steps,
                talbot_nodes,
                footing_load,
                footing_width,
                pulse_dur,
                0.85,
            );
            let split_err = compute_l2_relative_error(&split_disp, &baseline_disp);
            let avg_split_it = split_iters.iter().sum::<usize>() as f64 / split_iters.len() as f64;

            println!(
                "  {} ({} DOFs): Pure FEM err = {:.2}%, CPU = {:.1} ms | Split err = {:.2}%, CPU = {:.1} ms (iters: {:.1})",
                label, total_dofs, fem_err, fem_cpu, split_err, split_cpu, avg_split_it
            );

            disc_records.push(DiscretizationAccuracyRecord {
                resolution: label.to_string(),
                num_nodes,
                total_dofs,
                fem_error_pct: fem_err,
                fem_cpu_ms: fem_cpu,
                splitting_error_pct: split_err,
                splitting_cpu_ms: split_cpu,
                avg_fem_iters: avg_fem_it,
                avg_splitting_iters: avg_split_it,
            });

            if idx == 0 {
                traj_coarse_fem = fem_disp;
                traj_coarse_split = split_disp;
            } else if idx == resolutions.len() - 1 {
                traj_fine_fem = fem_disp;
                traj_fine_split = split_disp;
            }
        }

        // Export Discretization Accuracy CSV
        let disc_path = Path::new("../docs/benchmarks/mp05_half_space_discretization_accuracy_ms.csv");
        if let Some(parent) = disc_path.parent() {
            create_dir_all(parent).unwrap();
        }
        let mut disc_file = File::create(disc_path).unwrap();
        writeln!(disc_file, "resolution,nodes,dofs,fem_error_pct,fem_cpu_ms,split_error_pct,split_cpu_ms,avg_fem_iters,avg_split_iters").unwrap();
        for r in &disc_records {
            writeln!(
                disc_file,
                "{},{},{},{:.4},{:.2},{:.4},{:.2},{:.1},{:.1}",
                r.resolution, r.num_nodes, r.total_dofs, r.fem_error_pct, r.fem_cpu_ms, r.splitting_error_pct, r.splitting_cpu_ms, r.avg_fem_iters, r.avg_splitting_iters
            ).unwrap();
        }
        println!("Exported Discretization CSV to {:?}", disc_path);

        // Export Trajectory Comparison CSV
        let traj_path = Path::new("../docs/benchmarks/mp05_half_space_baseline_comparison.csv");
        let mut traj_file = File::create(traj_path).unwrap();
        writeln!(traj_file, "time_s,baseline_disp_mm,fem_coarse_disp_mm,fem_fine_disp_mm,split_coarse_disp_mm,split_fine_disp_mm").unwrap();
        for i in 0..times.len() {
            writeln!(
                traj_file,
                "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                times[i], baseline_disp[i], traj_coarse_fem[i], traj_fine_fem[i], traj_coarse_split[i], traj_fine_split[i]
            ).unwrap();
        }
        println!("Exported Trajectory CSV to {:?}", traj_path);

        // 3. Contrast Sensitivity Sweep on 20x10 Mesh
        let contrasts = [-0.5, -0.3, 0.0, 0.3, 0.5, 1.0, 1.5];
        let mesh_c = generate_beam_2d(40.0, 20.0, 20, 10, false);
        let mut contrast_records = Vec::new();

        println!("Running Contrast Sensitivity Sweep on 20x10 Mesh...");
        for &a in &contrasts {
            let (_, fem_d, fem_ms, _) = solve_half_space_pure_fem(
                &mesh_c,
                &ref_mat,
                a,
                total_time,
                time_steps,
                talbot_nodes,
                footing_load,
                footing_width,
                pulse_dur,
            );
            let (_, split_d, split_ms, split_it) = solve_half_space_operator_splitting(
                &mesh_c,
                &ref_mat,
                a,
                total_time,
                time_steps,
                talbot_nodes,
                footing_load,
                footing_width,
                pulse_dur,
                0.85,
            );

            let diff_err = compute_l2_relative_error(&split_d, &fem_d);
            let avg_it = split_it.iter().sum::<usize>() as f64 / split_it.len() as f64;

            println!("  alpha = {:.2}: diff between methods = {:.4}%, iters = {:.1}", a, diff_err, avg_it);

            contrast_records.push(ContrastSensitivityRecord {
                alpha: a,
                fem_error_pct: 0.0,
                splitting_error_pct: diff_err,
                splitting_iters: avg_it.round() as usize,
                fem_cpu_ms: fem_ms,
                splitting_cpu_ms: split_ms,
            });
        }

        let cont_path = Path::new("../docs/benchmarks/mp05_half_space_contrast_sweep.csv");
        let mut cont_file = File::create(cont_path).unwrap();
        writeln!(cont_file, "alpha,methods_diff_pct,splitting_iters,fem_cpu_ms,splitting_cpu_ms").unwrap();
        for r in &contrast_records {
            writeln!(
                cont_file,
                "{:.2},{:.4},{},{:.2},{:.2}",
                r.alpha, r.splitting_error_pct, r.splitting_iters, r.fem_cpu_ms, r.splitting_cpu_ms
            ).unwrap();
        }
        println!("Exported Contrast Sweep CSV to {:?}", cont_path);

        println!(">>> MP-05 Benchmark Suite Completed Successfully! <<<");
    }
