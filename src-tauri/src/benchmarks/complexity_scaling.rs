use crate::material::strategies::DirectionalGradientField;
use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
use crate::mesh::generators_2d::generate_beam_2d;
use crate::mesh::generators_3d::generate_beam_3d;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::solve_pure_fem;
use crate::solver::oraiopoulos_bem_fem::{solve_decoupled_bem_fem_hybrid, solve_oraiopoulos_bem_fem_3d};
use crate::solver::types::{HybridAcceleration, StressState2D};
use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ComplexityBenchmarkRecord {
    pub dimension: usize,
    pub mesh_label: String,
    pub num_nodes: usize,
    pub num_elements: usize,
    pub total_dofs: usize,
    pub boundary_nodes: usize,
    pub volume_nodes: usize,
    pub boundary_to_volume_ratio: f64,
    pub pure_fem_total_ms: f64,
    pub bemfem_total_ms: f64,
    pub bemfem_iterations: usize,
    pub speedup_ratio: f64,
    pub pure_fem_nnz_triplets: usize,
    pub estimated_ram_mb: f64,
    pub rel_l2_displacement_error: f64,
}

/// Computes the relative L2 displacement error norm between BEM-FEM and Pure FEM
pub fn compute_relative_l2_error(u_test: &[[f64; 3]], u_ref: &[[f64; 3]]) -> f64 {
    if u_test.len() != u_ref.len() || u_test.is_empty() {
        return 0.0;
    }
    let mut diff_sq_sum = 0.0;
    let mut ref_sq_sum = 0.0;
    for (t, r) in u_test.iter().zip(u_ref.iter()) {
        for d in 0..3 {
            let diff = t[d] - r[d];
            diff_sq_sum += diff * diff;
            ref_sq_sum += r[d] * r[d];
        }
    }
    let ref_norm = ref_sq_sum.sqrt();
    if ref_norm < 1e-14 {
        0.0
    } else {
        diff_sq_sum.sqrt() / ref_norm
    }
}

/// Counts distinct boundary nodes from mesh boundary elements
pub fn count_boundary_nodes(mesh: &MeshData) -> usize {
    let mut bnd_set = HashSet::new();
    for bnd in &mesh.boundary_elements {
        for &node in &bnd.nodes {
            bnd_set.insert(node);
        }
    }
    bnd_set.len()
}

/// Estimates sparse matrix non-zero count for regular grids
pub fn estimate_sparse_nnz(mesh: &MeshData) -> usize {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    // In 2D Quad4 each node is connected to ~9 nodes -> ~9 * dim^2 non-zeros per row block
    // In 3D Hex8 each node is connected to ~27 nodes -> ~27 * dim^2 non-zeros per row block
    let neighbors = if dim == 2 { 9 } else { 27 };
    num_nodes * neighbors * dim * dim
}

/// Runs progressive 2D h-refinement scaling benchmark
pub fn run_2d_scaling_sweep(
    divisions: &[(usize, usize)],
    alpha: f64,
    omega: f64,
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<ComplexityBenchmarkRecord> {
    let mut records = Vec::new();
    let length = 10.0;
    let height = 2.0;
    let ref_mat = ReferenceMaterial {
        e0: 30.0e9,
        nu0: 0.20,
        rho0: 2400.0,
    };
    let strat = DirectionalGradientField::new(
        Axis::X,
        alpha,
        length,
        1.0,
        GradientForm::Linear,
    );

    for &(nx, ny) in divisions {
        let mesh = generate_beam_2d(length, height, nx, ny, false); // Quad4
        let num_nodes = mesh.nodes.len();
        let num_elems = mesh.volume_elements.len();
        let total_dofs = num_nodes * 2;
        let bnd_nodes = count_boundary_nodes(&mesh);
        let vol_nodes = num_nodes.saturating_sub(bnd_nodes);
        let bnd_ratio = bnd_nodes as f64 / num_nodes.max(1) as f64;
        let nnz = estimate_sparse_nnz(&mesh);
        // RAM in MB: 8 bytes for f64 values + 8 bytes for row indices + 8 bytes col ptrs
        let ram_mb = (nnz * 16 + total_dofs * 8) as f64 / (1024.0 * 1024.0);

        // 1. Pure FEM Solve
        let fem_res = solve_pure_fem(&mesh, &ref_mat, &strat, load_mag);
        let fem_time = fem_res.execution_time_ms;

        // 2. BEM-FEM Hybrid Solve
        let bnd_nx = (length / 0.5).round().clamp(4.0, 16.0) as usize;
        let bnd_ny = (height / 0.5).round().clamp(2.0, 8.0) as usize;
        let (bemfem_res, _, _) = solve_decoupled_bem_fem_hybrid(
            length,
            height,
            bnd_nx,
            bnd_ny,
            &mesh,
            &ref_mat,
            &strat,
            load_mag,
            max_iters,
            tol,
            HybridAcceleration::Relaxation(omega),
            StressState2D::PlaneStress,
        );
        let bemfem_time = bemfem_res.execution_time_ms;
        let iters = bemfem_res.iterations;

        let speedup = if bemfem_time > 0.0 {
            fem_time / bemfem_time
        } else {
            1.0
        };

        let rel_l2 = compute_relative_l2_error(&bemfem_res.displacements, &fem_res.displacements);

        println!(
            "  [2D] {:>7} | N={:>5}, DOFs={:>6} | Bnd/Vol={:>6.3} | FEM: {:>8.2} ms | BEM-FEM: {:>8.2} ms (it={:>2}) | Spdup: {:>5.2}x | L2: {:>9.2e}",
            format!("{}x{}", nx, ny),
            num_nodes,
            total_dofs,
            bnd_ratio,
            fem_time,
            bemfem_time,
            iters,
            speedup,
            rel_l2
        );

        records.push(ComplexityBenchmarkRecord {
            dimension: 2,
            mesh_label: format!("{}x{}", nx, ny),
            num_nodes,
            num_elements: num_elems,
            total_dofs,
            boundary_nodes: bnd_nodes,
            volume_nodes: vol_nodes,
            boundary_to_volume_ratio: bnd_ratio,
            pure_fem_total_ms: fem_time,
            bemfem_total_ms: bemfem_time,
            bemfem_iterations: iters,
            speedup_ratio: speedup,
            pure_fem_nnz_triplets: nnz,
            estimated_ram_mb: ram_mb,
            rel_l2_displacement_error: rel_l2,
        });
    }

    records
}

#[derive(Debug, Clone)]
pub struct MultistepBenchmarkRecord {
    pub mesh_label: String,
    pub total_dofs: usize,
    pub num_steps: usize,
    pub pure_fem_total_ms: f64,
    pub bemfem_total_ms: f64,
    pub speedup_ratio: f64,
}

/// Runs progressive 3D h-refinement scaling benchmark
pub fn run_3d_scaling_sweep(
    divisions: &[(usize, usize, usize)],
    alpha: f64,
    omega: f64,
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<ComplexityBenchmarkRecord> {
    let mut records = Vec::new();
    let length = 10.0;
    let height = 2.0;
    let depth = 2.0;
    let ref_mat = ReferenceMaterial {
        e0: 30.0e9,
        nu0: 0.20,
        rho0: 2400.0,
    };
    let strat = DirectionalGradientField::new(
        Axis::X,
        alpha,
        length,
        1.0,
        GradientForm::Linear,
    );

    for &(nx, ny, nz) in divisions {
        let mesh = generate_beam_3d(length, height, depth, nx, ny, nz, false); // Hex8
        let num_nodes = mesh.nodes.len();
        let num_elems = mesh.volume_elements.len();
        let total_dofs = num_nodes * 3;
        let bnd_nodes = count_boundary_nodes(&mesh);
        let vol_nodes = num_nodes.saturating_sub(bnd_nodes);
        let bnd_ratio = bnd_nodes as f64 / num_nodes.max(1) as f64;
        let nnz = estimate_sparse_nnz(&mesh);
        let ram_mb = (nnz * 16 + total_dofs * 8) as f64 / (1024.0 * 1024.0);

        let fem_res = solve_pure_fem(&mesh, &ref_mat, &strat, load_mag);
        let fem_time = fem_res.execution_time_ms;

        let (bemfem_res, _pred_ms, _corr_ms) = solve_oraiopoulos_bem_fem_3d(
            length,
            height,
            depth,
            nx,
            ny,
            nz,
            &mesh,
            &ref_mat,
            &strat,
            load_mag,
            max_iters,
            tol,
            HybridAcceleration::Relaxation(omega),
        );
        let bemfem_time = bemfem_res.execution_time_ms;
        let iters = bemfem_res.iterations;

        let speedup = if bemfem_time > 0.0 {
            fem_time / bemfem_time
        } else {
            1.0
        };

        let rel_l2 = compute_relative_l2_error(&bemfem_res.displacements, &fem_res.displacements);

        println!(
            "  [3D] {:>9} | N={:>5}, DOFs={:>6} | Bnd/Vol={:>6.3} | FEM: {:>8.2} ms | BEM-FEM: {:>8.2} ms (it={:>2}) | Spdup: {:>5.2}x | L2: {:>9.2e}",
            format!("{}x{}x{}", nx, ny, nz),
            num_nodes,
            total_dofs,
            bnd_ratio,
            fem_time,
            bemfem_time,
            iters,
            speedup,
            rel_l2
        );

        records.push(ComplexityBenchmarkRecord {
            dimension: 3,
            mesh_label: format!("{}x{}x{}", nx, ny, nz),
            num_nodes,
            num_elements: num_elems,
            total_dofs,
            boundary_nodes: bnd_nodes,
            volume_nodes: vol_nodes,
            boundary_to_volume_ratio: bnd_ratio,
            pure_fem_total_ms: fem_time,
            bemfem_total_ms: bemfem_time,
            bemfem_iterations: iters,
            speedup_ratio: speedup,
            pure_fem_nnz_triplets: nnz,
            estimated_ram_mb: ram_mb,
            rel_l2_displacement_error: rel_l2,
        });
    }

    records
}

/// Serializes scaling benchmark records into CSV format
pub fn export_scaling_records_to_csv(
    records: &[ComplexityBenchmarkRecord],
    path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "dimension,mesh_label,num_nodes,num_elements,total_dofs,boundary_nodes,volume_nodes,boundary_to_volume_ratio,pure_fem_total_ms,bemfem_total_ms,bemfem_iterations,speedup_ratio,pure_fem_nnz_triplets,estimated_ram_mb,rel_l2_displacement_error"
    )?;

    for r in records {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{:.4},{:.3},{:.3},{},{:.4},{},{:.3},{:.6e}",
            r.dimension,
            r.mesh_label,
            r.num_nodes,
            r.num_elements,
            r.total_dofs,
            r.boundary_nodes,
            r.volume_nodes,
            r.boundary_to_volume_ratio,
            r.pure_fem_total_ms,
            r.bemfem_total_ms,
            r.bemfem_iterations,
            r.speedup_ratio,
            r.pure_fem_nnz_triplets,
            r.estimated_ram_mb,
            r.rel_l2_displacement_error
        )?;
    }
    Ok(())
}

/// Runs multi-load step / reanalysis benchmark demonstrating one-time K0 preconditioning advantage
pub fn run_multistep_reanalysis_sweep(
    divisions: &[(usize, usize, usize)],
    num_steps: usize,
    alpha: f64,
    omega: f64,
) -> Vec<MultistepBenchmarkRecord> {
    let mut records = Vec::new();
    let length = 10.0;
    let height = 2.0;
    let depth = 2.0;
    let ref_mat = ReferenceMaterial {
        e0: 30.0e9,
        nu0: 0.20,
        rho0: 2400.0,
    };
    let strat = DirectionalGradientField::new(
        Axis::X,
        alpha,
        length,
        1.0,
        GradientForm::Linear,
    );

    for &(nx, ny, nz) in divisions {
        let mesh = generate_beam_3d(length, height, depth, nx, ny, nz, false);
        let total_dofs = mesh.nodes.len() * 3;

        // 1. Pure FEM over N load steps
        let fem_start = std::time::Instant::now();
        for step in 1..=num_steps {
            let load = 1000.0 * (step as f64 / num_steps as f64);
            let _ = solve_pure_fem(&mesh, &ref_mat, &strat, load);
        }
        let fem_total_ms = fem_start.elapsed().as_secs_f64() * 1000.0;

        // 2. BEM-FEM over N load steps
        let bemfem_start = std::time::Instant::now();
        for step in 1..=num_steps {
            let load = 1000.0 * (step as f64 / num_steps as f64);
            let _ = solve_oraiopoulos_bem_fem_3d(
                length,
                height,
                depth,
                nx,
                ny,
                nz,
                &mesh,
                &ref_mat,
                &strat,
                load,
                50,
                1e-4,
                HybridAcceleration::Relaxation(omega),
            );
        }
        let bemfem_total_ms = bemfem_start.elapsed().as_secs_f64() * 1000.0;

        let speedup = if bemfem_total_ms > 0.0 {
            fem_total_ms / bemfem_total_ms
        } else {
            1.0
        };

        println!(
            "  [Multi-Step 3D] {:>9} | DOFs={:>6} | Steps={} | Pure FEM: {:>8.2} ms | BEM-FEM: {:>8.2} ms | Speedup: {:>5.2}x",
            format!("{}x{}x{}", nx, ny, nz),
            total_dofs,
            num_steps,
            fem_total_ms,
            bemfem_total_ms,
            speedup
        );

        records.push(MultistepBenchmarkRecord {
            mesh_label: format!("{}x{}x{}", nx, ny, nz),
            total_dofs,
            num_steps,
            pure_fem_total_ms: fem_total_ms,
            bemfem_total_ms: bemfem_total_ms,
            speedup_ratio: speedup,
        });
    }

    records
}

pub fn export_multistep_records_to_csv(
    records: &[MultistepBenchmarkRecord],
    path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "mesh_label,total_dofs,num_steps,pure_fem_total_ms,bemfem_total_ms,speedup_ratio"
    )?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{:.3},{:.3},{:.4}",
            r.mesh_label,
            r.total_dofs,
            r.num_steps,
            r.pure_fem_total_ms,
            r.bemfem_total_ms,
            r.speedup_ratio
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Contrast3DRecord {
    pub alpha: f64,
    pub contrast_ratio: f64,
    pub mesh_label: String,
    pub num_nodes: usize,
    pub total_dofs: usize,
    pub overkill_w_tip: f64,
    pub pure_fem_ms: f64,
    pub pure_fem_w_tip: f64,
    pub pure_fem_err_pct: f64,
    pub bemfem_ms: f64,
    pub bemfem_iters: usize,
    pub bemfem_w_tip: f64,
    pub bemfem_err_pct: f64,
    pub speedup_ratio: f64,
    pub rel_l2_diff: f64,
}

pub fn find_tip_node_3d(mesh: &MeshData, length: f64, height: f64, depth: f64) -> usize {
    let target = [length, height * 0.5, depth * 0.5];
    let mut best_idx = 0;
    let mut min_dist_sq = f64::MAX;
    for (idx, p) in mesh.nodes.iter().enumerate() {
        let dx = p[0] - target[0];
        let dy = p[1] - target[1];
        let dz = p[2] - target[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            best_idx = idx;
        }
    }
    best_idx
}

pub fn run_3d_contrast_sweep(
    alphas: &[f64],
    divisions: &[(usize, usize, usize)],
    overkill_div: (usize, usize, usize),
    acceleration: HybridAcceleration,
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<Contrast3DRecord> {
    let mut records = Vec::new();
    let length = 10.0;
    let height = 2.0;
    let depth = 2.0;
    let ref_mat = ReferenceMaterial {
        e0: 30.0e9,
        nu0: 0.20,
        rho0: 2400.0,
    };

    for &alpha in alphas {
        println!("\n>>> Running 3D Contrast Sweep for alpha = {:.2} (Contrast ratio: {:.2}x)...", alpha, 1.0 + alpha);
        let strat = DirectionalGradientField::new(
            Axis::X,
            alpha,
            length,
            1.0,
            GradientForm::Linear,
        );

        // 1. Solve Overkill 3D Ground Truth
        let (ox, oy, oz) = overkill_div;
        let mesh_overkill = generate_beam_3d(length, height, depth, ox, oy, oz, false);
        let overkill_tip_idx = find_tip_node_3d(&mesh_overkill, length, height, depth);
        let overkill_res = solve_pure_fem(&mesh_overkill, &ref_mat, &strat, load_mag);
        let w_ref = overkill_res.displacements[overkill_tip_idx][1].abs();
        let overkill_dofs = mesh_overkill.nodes.len() * 3;
        println!("  Overkill Mesh ({}x{}x{}, DOFs={}): w_ref = {:.6} mm, time = {:.2} ms", 
            ox, oy, oz, overkill_dofs, w_ref * 1000.0, overkill_res.execution_time_ms);

        // 2. Evaluate for each resolution
        for &(nx, ny, nz) in divisions {
            let mesh = generate_beam_3d(length, height, depth, nx, ny, nz, false);
            let num_nodes = mesh.nodes.len();
            let total_dofs = num_nodes * 3;
            let tip_idx = find_tip_node_3d(&mesh, length, height, depth);

            // Pure FEM
            let fem_res = solve_pure_fem(&mesh, &ref_mat, &strat, load_mag);
            let fem_w_tip = fem_res.displacements[tip_idx][1].abs();
            let fem_err = if w_ref > 1e-14 { ((fem_w_tip - w_ref).abs() / w_ref) * 100.0 } else { 0.0 };

            // BEM-FEM Hybrid (Oraiopoulos 3D Decoupled BEM Predictor + Volume Corrector)
            let (bemfem_res, _pred_ms, _corr_ms) = solve_oraiopoulos_bem_fem_3d(
                length,
                height,
                depth,
                nx,
                ny,
                nz,
                &mesh,
                &ref_mat,
                &strat,
                load_mag,
                max_iters,
                tol,
                acceleration,
            );
            let bemfem_w_tip = bemfem_res.displacements[tip_idx][1].abs();
            let bemfem_err = if w_ref > 1e-14 { ((bemfem_w_tip - w_ref).abs() / w_ref) * 100.0 } else { 0.0 };
            let speedup = if bemfem_res.execution_time_ms > 0.0 {
                fem_res.execution_time_ms / bemfem_res.execution_time_ms
            } else {
                1.0
            };
            let rel_l2 = compute_relative_l2_error(&bemfem_res.displacements, &fem_res.displacements);

            println!(
                "  [{:>7}, DOFs={:>5}] FEM: {:>6.2} ms (err={:>5.2}%) | BEM-FEM: {:>6.2} ms, it={:>2} (err={:>5.2}%) | Spd: {:>4.2}x | L2: {:>8.2e}",
                format!("{}x{}x{}", nx, ny, nz),
                total_dofs,
                fem_res.execution_time_ms,
                fem_err,
                bemfem_res.execution_time_ms,
                bemfem_res.iterations,
                bemfem_err,
                speedup,
                rel_l2
            );

            records.push(Contrast3DRecord {
                alpha,
                contrast_ratio: 1.0 + alpha,
                mesh_label: format!("{}x{}x{}", nx, ny, nz),
                num_nodes,
                total_dofs,
                overkill_w_tip: w_ref * 1000.0,
                pure_fem_ms: fem_res.execution_time_ms,
                pure_fem_w_tip: fem_w_tip * 1000.0,
                pure_fem_err_pct: fem_err,
                bemfem_ms: bemfem_res.execution_time_ms,
                bemfem_iters: bemfem_res.iterations,
                bemfem_w_tip: bemfem_w_tip * 1000.0,
                bemfem_err_pct: bemfem_err,
                speedup_ratio: speedup,
                rel_l2_diff: rel_l2,
            });
        }
    }

    records
}

pub fn export_3d_contrast_records_to_csv(
    records: &[Contrast3DRecord],
    path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "alpha,contrast_ratio,mesh_label,num_nodes,total_dofs,overkill_w_tip_mm,pure_fem_ms,pure_fem_w_tip_mm,pure_fem_err_pct,bemfem_ms,bemfem_iters,bemfem_w_tip_mm,bemfem_err_pct,speedup_ratio,rel_l2_diff"
    )?;

    for r in records {
        writeln!(
            file,
            "{:.2},{:.2},{},{},{},{:.6},{:.2},{:.6},{:.2},{:.2},{},{:.6},{:.2},{:.4},{:.6e}",
            r.alpha,
            r.contrast_ratio,
            r.mesh_label,
            r.num_nodes,
            r.total_dofs,
            r.overkill_w_tip,
            r.pure_fem_ms,
            r.pure_fem_w_tip,
            r.pure_fem_err_pct,
            r.bemfem_ms,
            r.bemfem_iters,
            r.bemfem_w_tip,
            r.bemfem_err_pct,
            r.speedup_ratio,
            r.rel_l2_diff
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "complexity_scaling_test.rs"]
mod tests;
