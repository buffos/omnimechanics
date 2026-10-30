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
    fn test_execute_and_export_all_mp01_benchmarks() {
        println!(">>> Starting Mini-Paper 1 Comprehensive Static Benchmark Suite <<<");

        let length = 10.0;
        let height = 2.0;
        let nx = 20;
        let ny = 4;
        let mesh = generate_beam_2d(length, height, nx, ny, false); // Quad4
        let load_mag = 1000.0;
        let max_iters = 80;
        let tol = 1e-4;

        // 1. Sweep A: Functional Continuous Gradient
        let alphas = vec![-0.8, -0.5, -0.2, 0.0, 0.2, 0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 3.0];
        let omegas = vec![1.0, 0.85, 0.50];
        let records_a = run_continuous_gradient_sweep(
            &mesh,
            length,
            30.0e9,
            0.20,
            &alphas,
            &omegas,
            load_mag,
            max_iters,
            tol,
        );

        let csv_path_a = Path::new("../docs/benchmarks/mp01_gradient_sweep.csv");
        export_records_to_csv(&records_a, csv_path_a).expect("Failed to export Sweep A CSV");
        println!("Exported {} records to {:?}", records_a.len(), csv_path_a);

        // 2. Sweep B: Rock in Soil Hard Inclusion
        let center = [5.0, 1.0, 0.0];
        let radius = 0.6;
        let e_soil = 100.0e6; // 100 MPa
        let nu_soil = 0.30;
        let hard_contrasts = vec![1.0, 1.5, 2.0, 3.0, 5.0, 10.0, 20.0, 50.0, 100.0, 500.0];
        let records_b = run_rock_in_soil_sweep(
            &mesh,
            center,
            radius,
            e_soil,
            nu_soil,
            &hard_contrasts,
            load_mag,
            max_iters,
            tol,
        );

        let csv_path_b = Path::new("../docs/benchmarks/mp01_rock_inclusion_sweep.csv");
        export_records_to_csv(&records_b, csv_path_b).expect("Failed to export Sweep B CSV");
        println!("Exported {} records to {:?}", records_b.len(), csv_path_b);

        // 3. Sweep C: Soft Inclusion / Cavity
        let e_rock = 30.0e9; // 30 GPa
        let nu_rock = 0.20;
        let soft_contrasts = vec![1.0, 0.8, 0.5, 0.2, 0.1, 0.05, 0.01, 0.001];
        let records_c = run_soft_inclusion_sweep(
            &mesh,
            center,
            radius,
            e_rock,
            nu_rock,
            &soft_contrasts,
            load_mag,
            max_iters,
            tol,
        );

        let csv_path_c = Path::new("../docs/benchmarks/mp01_soft_cavity_sweep.csv");
        export_records_to_csv(&records_c, csv_path_c).expect("Failed to export Sweep C CSV");
        println!("Exported {} records to {:?}", records_c.len(), csv_path_c);

        assert!(!records_a.is_empty());
        assert!(!records_b.is_empty());
        assert!(!records_c.is_empty());
        println!(">>> MP-01 Benchmark Suite Completed Successfully! <<<");
    }
