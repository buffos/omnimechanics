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
    pub fn test_execute_and_export_all_mp03_transient_benchmarks() {
        println!(">>> Starting Mini-Paper 3 Transient Elastodynamics Benchmark Suite <<<");
        let mesh = generate_beam_2d(10.0, 2.0, 16, 4, false); // 16x4 Quad4 beam
        let ref_mat = ReferenceMaterial::default();
        let p_load = 50_000.0;

        // 1. Sweep A: Dispersion and Frequency Response
        println!("Running Sweep A: Dispersion and Frequency Response (Harmonic 3 Hz)...");
        let records_a = run_harmonic_dispersion_sweep(&mesh, &ref_mat, 0.5, 3.0, p_load);
        let csv_path_a = Path::new("../docs/benchmarks/mp03_dispersion_newmark_vs_talbot.csv");
        export_dispersion_to_csv(&records_a, csv_path_a).expect("Failed to export Sweep A CSV");
        println!("Exported {} records to {:?}", records_a.len(), csv_path_a);

        // 2. Sweep B: Shock Pulse and Wavefront Transmission
        println!("Running Sweep B: Shock Pulse and Wavefront Transmission...");
        let records_b = run_shock_pulse_sweep(&mesh, &ref_mat, 0.15, 0.02, p_load);
        let csv_path_b = Path::new("../docs/benchmarks/mp03_shock_pulse_wave_propagation.csv");
        export_pulse_to_csv(&records_b, csv_path_b).expect("Failed to export Sweep B CSV");
        println!("Exported {} records to {:?}", records_b.len(), csv_path_b);

        // 3. Sweep C: Point-in-Time Random Access & Long-Duration Computational Scaling
        println!("Running Sweep C: Long-Duration Scaling & Point-in-Time Random Access...");
        let durations = [0.5, 1.0, 2.0, 5.0];
        let records_c = run_long_duration_scaling_sweep(&mesh, &ref_mat, &durations, 0.005, 32, p_load);
        let csv_path_c = Path::new("../docs/benchmarks/mp03_runtime_talbot_vs_newmark.csv");
        export_long_duration_to_csv(&records_c, csv_path_c).expect("Failed to export Sweep C CSV");
        println!("Exported {} records to {:?}", records_c.len(), csv_path_c);

        println!(">>> MP-03 Transient Elastodynamics Benchmark Suite Completed Successfully! <<<");
    }
