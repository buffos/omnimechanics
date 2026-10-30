use crate::material::strategies::DirectionalGradientField;
use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
use crate::mesh::generators_2d::generate_beam_2d;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::{
    assemble_heterogeneous_dynamic_stiffness_and_mass,
    assemble_reference_dynamic_stiffness_and_mass,
};
use crate::solver::hybrid_solver::{
    solve_laplace_dynamic_operator_splitting, solve_laplace_pure_fem,
    solve_transient_dynamic_operator_splitting_talbot, solve_transient_pure_fem_talbot,
};
use crate::solver::talbot::generate_talbot_nodes;
use num_complex::Complex64;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ShockPulseBenchmarkRecord {
    pub time_s: f64,
    pub fem_tip_disp_mm: f64,
    pub splitting_tip_disp_mm: f64,
    pub rel_diff_pct: f64,
    pub fem_max_vm_mpa: f64,
    pub splitting_max_vm_mpa: f64,
}

#[derive(Debug, Clone)]
pub struct ContrastConvergenceRecord {
    pub alpha: f64,
    pub contrast_ratio_e_max_e_min: f64,
    pub operator_splitting_iters: usize,
    pub contraction_factor: f64,
    pub final_relative_residual: f64,
    pub fem_gmres_iters: usize,
    pub speedup_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct DofsScalingRecord {
    pub mesh_resolution: String,
    pub num_nodes: usize,
    pub total_dofs: usize,
    pub pure_fem_cpu_ms: f64,
    pub operator_splitting_cpu_ms: f64,
    pub speedup: f64,
    pub avg_fem_gmres_iters: f64,
    pub avg_splitting_iters: f64,
}

/// Runs Sweep A: Broadband Transient Half-Sine Shock Pulse Comparison
pub fn run_shock_pulse_sweep(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    alpha: f64,
    total_time: f64,
    time_steps: usize,
    num_talbot_nodes: usize,
    load_mag: f64,
    pulse_duration: f64,
) -> (Vec<ShockPulseBenchmarkRecord>, f64, f64, f64, f64) {
    let ly = mesh.nodes.iter().map(|p| p[1]).fold(0.0, f64::max).max(1.0);
    let strategy = DirectionalGradientField::new(
        Axis::Y,
        alpha,
        ly,
        1.0,
        GradientForm::Linear,
    );

    // 1. Pure FEM + Talbot
    let start_fem = std::time::Instant::now();
    let (res_fem, fem_iters) = solve_transient_pure_fem_talbot(
        mesh,
        ref_mat,
        &strategy,
        total_time,
        time_steps,
        num_talbot_nodes,
        load_mag,
        pulse_duration,
        Some("HalfSine"),
        None,
        100,
        1e-7,
    );
    let fem_cpu = start_fem.elapsed().as_secs_f64() * 1000.0;

    // 2. Variational Dynamic Operator Splitting + Talbot
    let start_split = std::time::Instant::now();
    let (res_split, split_iters) = solve_transient_dynamic_operator_splitting_talbot(
        mesh,
        ref_mat,
        &strategy,
        total_time,
        time_steps,
        num_talbot_nodes,
        load_mag,
        pulse_duration,
        Some("HalfSine"),
        None,
        30,
        1e-5,
        0.85,
    );
    let split_cpu = start_split.elapsed().as_secs_f64() * 1000.0;

    let avg_fem_it = fem_iters.iter().sum::<usize>() as f64 / fem_iters.len() as f64;
    let avg_split_it = split_iters.iter().sum::<usize>() as f64 / split_iters.len() as f64;

    let mut records = Vec::new();
    let n_frames = res_fem.frames.len().min(res_split.frames.len());

    for i in 0..n_frames {
        let f_fem = &res_fem.frames[i];
        let f_split = &res_split.frames[i];

        let d_fem = f_fem.max_displacement * 1000.0;
        let d_split = f_split.max_displacement * 1000.0;

        let diff = (d_fem - d_split).abs();
        let rel_diff = diff / d_fem.max(1e-8) * 100.0;

        let vm_fem = f_fem.von_mises_stresses.iter().cloned().fold(0.0, f64::max);
        let vm_split = f_split.von_mises_stresses.iter().cloned().fold(0.0, f64::max);

        records.push(ShockPulseBenchmarkRecord {
            time_s: f_fem.time,
            fem_tip_disp_mm: d_fem,
            splitting_tip_disp_mm: d_split,
            rel_diff_pct: rel_diff,
            fem_max_vm_mpa: vm_fem,
            splitting_max_vm_mpa: vm_split,
        });
    }

    (records, fem_cpu, split_cpu, avg_fem_it, avg_split_it)
}

/// Runs Sweep B: Convergence across Material Contrast in the Complex Laplace Domain
pub fn run_contrast_convergence_sweep(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    contrasts: &[f64],
    total_time: f64,
    load_mag: f64,
) -> Vec<ContrastConvergenceRecord> {
    let dim = mesh.dimension;
    let num_nodes = mesh.nodes.len();
    let total_dofs = num_nodes * dim;
    let ly = mesh.nodes.iter().map(|p| p[1]).fold(0.0, f64::max).max(1.0);

    // Assemble homogeneous reference operator A_0(s)
    let (k0_mat, m0_mat, fixed_nodes) = assemble_reference_dynamic_stiffness_and_mass(mesh, ref_mat);

    // Generate representative Talbot nodes: pick the dominant midpoint node
    let talbot_nodes = generate_talbot_nodes(32, total_time);
    let mid_node = &talbot_nodes[talbot_nodes.len() / 2];
    let s = mid_node.s;

    // Load transform
    let t_pulse = 0.04;
    let omega_p = std::f64::consts::PI / t_pulse;
    let pulse_factor = (Complex64::new(omega_p, 0.0) * (Complex64::new(1.0, 0.0) + (-s * t_pulse).exp()))
        / (s * s + Complex64::new(omega_p * omega_p, 0.0));

    // Boundary load
    let mut loaded_nodes = Vec::new();
    for b in &mesh.boundary_elements {
        if b.tag == "Right" {
            for &n in &b.nodes {
                if !loaded_nodes.contains(&n) {
                    loaded_nodes.push(n);
                }
            }
        }
    }
    let mut f_s = vec![Complex64::new(0.0, 0.0); total_dofs];
    if !loaded_nodes.is_empty() {
        let force_per_node = -load_mag / loaded_nodes.len() as f64;
        for &node_idx in &loaded_nodes {
            f_s[node_idx * dim + 1] = Complex64::new(force_per_node, 0.0) * pulse_factor;
        }
    }

    let mut records = Vec::new();

    for &alpha in contrasts {
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            alpha,
            ly,
            1.0,
            GradientForm::Linear,
        );

        let (k_hetero, m_hetero, _) = assemble_heterogeneous_dynamic_stiffness_and_mass(mesh, ref_mat, &strategy);

        // 1. Pure FEM solve
        let start_fem = std::time::Instant::now();
        let (_u_fem, fem_iters, _fem_res) = solve_laplace_pure_fem(
            &k_hetero,
            &m_hetero,
            &fixed_nodes,
            dim,
            s,
            &f_s,
            100,
            1e-7,
        );
        let cpu_fem = start_fem.elapsed().as_secs_f64() * 1000.0;

        // 2. Dynamic Operator Splitting solve
        let start_split = std::time::Instant::now();
        let (_u_split, split_iters, split_res) = solve_laplace_dynamic_operator_splitting(
            &k0_mat,
            &m0_mat,
            &k_hetero,
            &m_hetero,
            &fixed_nodes,
            dim,
            s,
            &f_s,
            40,
            1e-5,
            0.85,
        );
        let cpu_split = start_split.elapsed().as_secs_f64() * 1000.0;

        let contrast_ratio = if alpha >= 0.0 {
            1.0 + alpha
        } else {
            1.0 / (1.0 + alpha).max(0.01)
        };

        // Estimate contraction factor rho ~ (rel_res)^(1 / iters)
        let contraction = if split_iters > 1 {
            split_res.powf(1.0 / split_iters as f64).clamp(0.0, 0.99)
        } else {
            0.0
        };

        let speedup = cpu_fem / cpu_split.max(0.01);

        records.push(ContrastConvergenceRecord {
            alpha,
            contrast_ratio_e_max_e_min: contrast_ratio,
            operator_splitting_iters: split_iters,
            contraction_factor: contraction,
            final_relative_residual: split_res,
            fem_gmres_iters: fem_iters,
            speedup_ratio: speedup,
        });
    }

    records
}

/// Runs Sweep C: Computational Complexity and Wall-Clock Scaling vs Problem Size (DOFs)
pub fn run_dofs_scaling_sweep(
    ref_mat: &ReferenceMaterial,
    alpha: f64,
    resolutions: &[(usize, usize, &str)],
    total_time: f64,
    time_steps: usize,
    num_talbot_nodes: usize,
    load_mag: f64,
) -> Vec<DofsScalingRecord> {
    let mut records = Vec::new();

    for &(nx, ny, label) in resolutions {
        let mesh = generate_beam_2d(10.0, 2.0, nx, ny, false);
        let num_nodes = mesh.nodes.len();
        let total_dofs = num_nodes * mesh.dimension;

        let strategy = DirectionalGradientField::new(
            Axis::Y,
            alpha,
            2.0,
            1.0,
            GradientForm::Linear,
        );

        // 1. Pure FEM + Talbot
        let start_fem = std::time::Instant::now();
        let (_res_fem, fem_iters) = solve_transient_pure_fem_talbot(
            &mesh,
            ref_mat,
            &strategy,
            total_time,
            time_steps,
            num_talbot_nodes,
            load_mag,
            0.04,
            Some("HalfSine"),
            None,
            100,
            1e-7,
        );
        let cpu_fem = start_fem.elapsed().as_secs_f64() * 1000.0;

        // 2. Variational Dynamic Operator Splitting + Talbot
        let start_split = std::time::Instant::now();
        let (_res_split, split_iters) = solve_transient_dynamic_operator_splitting_talbot(
            &mesh,
            ref_mat,
            &strategy,
            total_time,
            time_steps,
            num_talbot_nodes,
            load_mag,
            0.04,
            Some("HalfSine"),
            None,
            30,
            1e-5,
            0.85,
        );
        let cpu_split = start_split.elapsed().as_secs_f64() * 1000.0;

        let avg_fem_it = fem_iters.iter().sum::<usize>() as f64 / fem_iters.len() as f64;
        let avg_split_it = split_iters.iter().sum::<usize>() as f64 / split_iters.len() as f64;
        let speedup = cpu_fem / cpu_split.max(0.01);

        records.push(DofsScalingRecord {
            mesh_resolution: label.to_string(),
            num_nodes,
            total_dofs,
            pure_fem_cpu_ms: cpu_fem,
            operator_splitting_cpu_ms: cpu_split,
            speedup,
            avg_fem_gmres_iters: avg_fem_it,
            avg_splitting_iters: avg_split_it,
        });
    }

    records
}

/// CSV Export helper for Sweep A
pub fn export_shock_pulse_to_csv(records: &[ShockPulseBenchmarkRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(file, "time_s,fem_tip_disp_mm,splitting_tip_disp_mm,rel_diff_pct,fem_max_vm_mpa,splitting_max_vm_mpa")?;
    for r in records {
        writeln!(
            file,
            "{:.6},{:.6},{:.6},{:.6},{:.4},{:.4}",
            r.time_s, r.fem_tip_disp_mm, r.splitting_tip_disp_mm, r.rel_diff_pct, r.fem_max_vm_mpa, r.splitting_max_vm_mpa
        )?;
    }
    Ok(())
}

/// CSV Export helper for Sweep B
pub fn export_contrast_to_csv(records: &[ContrastConvergenceRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(file, "alpha,contrast_ratio,operator_splitting_iters,contraction_factor,final_relative_residual,fem_gmres_iters,speedup_ratio")?;
    for r in records {
        writeln!(
            file,
            "{:.2},{:.4},{},{:.4},{:.4e},{},{:.2}",
            r.alpha, r.contrast_ratio_e_max_e_min, r.operator_splitting_iters, r.contraction_factor, r.final_relative_residual, r.fem_gmres_iters, r.speedup_ratio
        )?;
    }
    Ok(())
}

/// CSV Export helper for Sweep C
pub fn export_scaling_to_csv(records: &[DofsScalingRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(file, "resolution,num_nodes,total_dofs,pure_fem_cpu_ms,operator_splitting_cpu_ms,speedup,avg_fem_gmres_iters,avg_splitting_iters")?;
    for r in records {
        writeln!(
            file,
            "{},{},{},{:.2},{:.2},{:.2},{:.1},{:.1}",
            r.mesh_resolution, r.num_nodes, r.total_dofs, r.pure_fem_cpu_ms, r.operator_splitting_cpu_ms, r.speedup, r.avg_fem_gmres_iters, r.avg_splitting_iters
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "dynamic_operator_splitting_test.rs"]
mod tests;
