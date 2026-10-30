use crate::material::strategies::{DirectionalGradientField, InclusionField};
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{Axis, GradientForm, InclusionShape, LocalMaterialProperties, ReferenceMaterial};

#[derive(Debug, Clone)]
pub struct ExplicitInclusionField {
    pub center: [f64; 3],
    pub radius: f64,
    pub e_matrix: f64,
    pub nu_matrix: f64,
    pub e_inclusion: f64,
    pub nu_inclusion: f64,
}

impl MaterialFieldStrategy for ExplicitInclusionField {
    fn evaluate(&self, point: [f64; 3], _reference: &ReferenceMaterial) -> LocalMaterialProperties {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dist = (dx * dx + dy * dy).sqrt();
        let (e, nu) = if dist <= self.radius {
            (self.e_inclusion, self.nu_inclusion)
        } else {
            (self.e_matrix, self.nu_matrix)
        };
        let mu = e / (2.0 * (1.0 + nu));
        let lambda = (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));
        LocalMaterialProperties {
            e,
            nu,
            rho: 2400.0,
            mu,
            lambda,
            secant_modulus: None,
            yield_stress: None,
            scalar_metric: e,
        }
    }
}
use crate::mesh::generators_2d::generate_beam_2d;
use crate::mesh::types::MeshData;
use crate::solver::fem_solver::solve_pure_fem;
use crate::solver::oraiopoulos_bem_fem::solve_decoupled_bem_fem_hybrid;
use crate::solver::types::{HybridAcceleration, SimulationResult, StressState2D};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

fn solve_bem_fem_hybrid_with_relaxation(
    mesh: &MeshData,
    ref_mat: &ReferenceMaterial,
    strat: &dyn MaterialFieldStrategy,
    load_mag: f64,
    max_iters: usize,
    tol: f64,
    omega: f64,
) -> SimulationResult {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in &mesh.nodes {
        if p[0] < min_x { min_x = p[0]; }
        if p[0] > max_x { max_x = p[0]; }
        if p[1] < min_y { min_y = p[1]; }
        if p[1] > max_y { max_y = p[1]; }
    }
    let length = (max_x - min_x).max(1e-3);
    let height = (max_y - min_y).max(1e-3);
    let bnd_nx = (length / 0.5).round().clamp(4.0, 16.0) as usize;
    let bnd_ny = (height / 0.5).round().clamp(2.0, 8.0) as usize;
    let (res, _, _) = solve_decoupled_bem_fem_hybrid(
        length,
        height,
        bnd_nx,
        bnd_ny,
        mesh,
        ref_mat,
        strat,
        load_mag,
        max_iters,
        tol,
        HybridAcceleration::Relaxation(omega),
        StressState2D::PlaneStress,
    );
    res
}

#[derive(Debug, Clone)]
pub struct StaticBenchmarkRecord {
    pub study_name: String,
    pub parameter_label: String,
    pub parameter_value: f64,
    pub contrast_ratio: f64,
    pub method_name: String,
    pub omega: f64,
    pub converged: bool,
    pub iterations: usize,
    pub spectral_radius_est: f64,
    pub final_rel_error: f64,
    pub max_disp_bemfem: f64,
    pub max_disp_pure_fem: f64,
    pub rel_l2_disp_error: f64,
    pub cpu_time_ms: f64,
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

/// Estimates asymptotic convergence rate / spectral radius rho = ||r_{k}|| / ||r_{k-1}||
pub fn estimate_spectral_radius(history: &[crate::solver::types::ConvergenceLog]) -> f64 {
    if history.len() < 2 {
        return 0.0;
    }
    let last = history.last().unwrap().residual_norm;
    let prev = history[history.len() - 2].residual_norm;
    if prev > 1e-14 {
        last / prev
    } else {
        0.0
    }
}

/// Runs Sweep A: Continuous Linear Inhomogeneity Gradient
/// E(x) = E_0 * (1 + alpha * x / L)
pub fn run_continuous_gradient_sweep(
    mesh: &MeshData,
    length: f64,
    e0: f64,
    nu0: f64,
    alphas: &[f64],
    omegas: &[f64],
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<StaticBenchmarkRecord> {
    let mut records = Vec::new();
    let ref_mat = ReferenceMaterial {
        e0,
        nu0,
        rho0: 2400.0,
    };

    for &alpha in alphas {
        let strat = DirectionalGradientField::new(
            Axis::X,
            alpha,
            length,
            1.0,
            GradientForm::Linear,
        );

        // Ground truth: Pure FEM
        let fem_res = solve_pure_fem(mesh, &ref_mat, &strat, load_mag);

        for &omega in omegas {
            let method_label = if (omega - 1.0).abs() < 1e-4 {
                "BEMFEM (Unrelaxed w=1.00)".to_string()
            } else {
                format!("BEMFEM (Relaxed w={:.2})", omega)
            };

            let bemfem_res = solve_bem_fem_hybrid_with_relaxation(
                mesh,
                &ref_mat,
                &strat,
                load_mag,
                max_iters,
                tol,
                omega,
            );

            let rho = estimate_spectral_radius(&bemfem_res.convergence_history);
            let rel_l2 = if bemfem_res.converged {
                compute_relative_l2_error(&bemfem_res.displacements, &fem_res.displacements)
            } else {
                f64::NAN
            };

            let final_err = bemfem_res
                .convergence_history
                .last()
                .map(|h| h.relative_error)
                .unwrap_or(1.0);

            records.push(StaticBenchmarkRecord {
                study_name: "Continuous_Gradient".to_string(),
                parameter_label: "alpha".to_string(),
                parameter_value: alpha,
                contrast_ratio: 1.0 + alpha,
                method_name: method_label,
                omega,
                converged: bemfem_res.converged,
                iterations: bemfem_res.iterations,
                spectral_radius_est: rho,
                final_rel_error: final_err,
                max_disp_bemfem: bemfem_res.max_displacement,
                max_disp_pure_fem: fem_res.max_displacement,
                rel_l2_disp_error: rel_l2,
                cpu_time_ms: bemfem_res.execution_time_ms,
            });
        }
    }

    records
}

/// Runs Sweep B: Hard Inclusion ("Rock in Soil") Challenge
/// Circular inclusion in center of soil beam
pub fn run_rock_in_soil_sweep(
    mesh: &MeshData,
    center: [f64; 3],
    radius: f64,
    e_soil: f64,
    nu_soil: f64,
    contrast_ratios: &[f64],
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<StaticBenchmarkRecord> {
    let mut records = Vec::new();

    for &contrast in contrast_ratios {
        let e_rock = e_soil * contrast;

        let strat = ExplicitInclusionField {
            center,
            radius,
            e_matrix: e_soil,
            nu_matrix: nu_soil,
            e_inclusion: e_rock,
            nu_inclusion: nu_soil,
        };

        // Ground truth: Pure FEM
        let ref_soil = ReferenceMaterial {
            e0: e_soil,
            nu0: nu_soil,
            rho0: 2000.0,
        };
        let fem_res = solve_pure_fem(mesh, &ref_soil, &strat, load_mag);

        // Sub-test 1: BEM Operator using Soil reference (E0 = E_soil) with w=1.0, 0.85, 0.20
        for &omega in &[1.0, 0.85, 0.20] {
            let label = format!("BEMFEM (E0=Soil, w={:.2})", omega);
            let res = solve_bem_fem_hybrid_with_relaxation(
                mesh,
                &ref_soil,
                &strat,
                load_mag,
                max_iters,
                tol,
                omega,
            );
            let rho = estimate_spectral_radius(&res.convergence_history);
            let rel_l2 = if res.converged {
                compute_relative_l2_error(&res.displacements, &fem_res.displacements)
            } else {
                f64::NAN
            };
            let final_err = res
                .convergence_history
                .last()
                .map(|h| h.relative_error)
                .unwrap_or(1.0);

            records.push(StaticBenchmarkRecord {
                study_name: "Rock_In_Soil_Soil_Base".to_string(),
                parameter_label: "E_rock/E_soil".to_string(),
                parameter_value: contrast,
                contrast_ratio: contrast,
                method_name: label,
                omega,
                converged: res.converged,
                iterations: res.iterations,
                spectral_radius_est: rho,
                final_rel_error: final_err,
                max_disp_bemfem: res.max_displacement,
                max_disp_pure_fem: fem_res.max_displacement,
                rel_l2_disp_error: rel_l2,
                cpu_time_ms: res.execution_time_ms,
            });
        }

        // Sub-test 2: BEM Operator using Rock reference (E0 = E_rock) with w=1.0, 0.85
        let ref_rock = ReferenceMaterial {
            e0: e_rock,
            nu0: nu_soil,
            rho0: 2600.0,
        };
        for &omega in &[1.0, 0.85] {
            let label = format!("BEMFEM (E0=Rock, w={:.2})", omega);
            let res = solve_bem_fem_hybrid_with_relaxation(
                mesh,
                &ref_rock,
                &strat,
                load_mag,
                max_iters,
                tol,
                omega,
            );
            let rho = estimate_spectral_radius(&res.convergence_history);
            let rel_l2 = if res.converged {
                compute_relative_l2_error(&res.displacements, &fem_res.displacements)
            } else {
                f64::NAN
            };
            let final_err = res
                .convergence_history
                .last()
                .map(|h| h.relative_error)
                .unwrap_or(1.0);

            records.push(StaticBenchmarkRecord {
                study_name: "Rock_In_Soil_Rock_Base".to_string(),
                parameter_label: "E_rock/E_soil".to_string(),
                parameter_value: contrast,
                contrast_ratio: contrast,
                method_name: label,
                omega,
                converged: res.converged,
                iterations: res.iterations,
                spectral_radius_est: rho,
                final_rel_error: final_err,
                max_disp_bemfem: res.max_displacement,
                max_disp_pure_fem: fem_res.max_displacement,
                rel_l2_disp_error: rel_l2,
                cpu_time_ms: res.execution_time_ms,
            });
        }
    }

    records
}

/// Runs Sweep C: Soft Inclusion / Cavity ("Void in Rock") Challenge
pub fn run_soft_inclusion_sweep(
    mesh: &MeshData,
    center: [f64; 3],
    radius: f64,
    e_rock: f64,
    nu_rock: f64,
    soft_contrasts: &[f64],
    load_mag: f64,
    max_iters: usize,
    tol: f64,
) -> Vec<StaticBenchmarkRecord> {
    let mut records = Vec::new();
    let ref_rock = ReferenceMaterial {
        e0: e_rock,
        nu0: nu_rock,
        rho0: 2600.0,
    };

    for &contrast in soft_contrasts {
        let e_inc = (e_rock * contrast).max(1e3); // Min 1 kPa to prevent singular matrix

        let strat = InclusionField::new(
            InclusionShape::CylinderZ,
            center,
            radius,
            None,
            e_inc,
            nu_rock,
            1200.0,
            false,
        );

        let fem_res = solve_pure_fem(mesh, &ref_rock, &strat, load_mag);

        for &omega in &[1.0f64, 0.85f64] {
            let label = if (omega - 1.0f64).abs() < 1e-4 {
                "BEMFEM (Unrelaxed w=1.00)".to_string()
            } else {
                format!("BEMFEM (Relaxed w={:.2})", omega)
            };

            let res = solve_bem_fem_hybrid_with_relaxation(
                mesh,
                &ref_rock,
                &strat,
                load_mag,
                max_iters,
                tol,
                omega,
            );

            let rho = estimate_spectral_radius(&res.convergence_history);
            let rel_l2 = if res.converged {
                compute_relative_l2_error(&res.displacements, &fem_res.displacements)
            } else {
                f64::NAN
            };
            let final_err = res
                .convergence_history
                .last()
                .map(|h| h.relative_error)
                .unwrap_or(1.0);

            records.push(StaticBenchmarkRecord {
                study_name: "Soft_Inclusion_Cavity".to_string(),
                parameter_label: "E_cavity/E_rock".to_string(),
                parameter_value: contrast,
                contrast_ratio: contrast,
                method_name: label,
                omega,
                converged: res.converged,
                iterations: res.iterations,
                spectral_radius_est: rho,
                final_rel_error: final_err,
                max_disp_bemfem: res.max_displacement,
                max_disp_pure_fem: fem_res.max_displacement,
                rel_l2_disp_error: rel_l2,
                cpu_time_ms: res.execution_time_ms,
            });
        }
    }

    records
}

/// Serializes benchmark records into CSV format
pub fn export_records_to_csv(records: &[StaticBenchmarkRecord], path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    writeln!(
        file,
        "study_name,parameter_label,parameter_value,contrast_ratio,method_name,omega,converged,iterations,spectral_radius_est,final_rel_error,max_disp_bemfem_m,max_disp_pure_fem_m,rel_l2_disp_error,cpu_time_ms"
    )?;

    for r in records {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{},{:.2},{},{},{:.6},{:.6e},{:.6e},{:.6e},{:.6e},{:.3}",
            r.study_name,
            r.parameter_label,
            r.parameter_value,
            r.contrast_ratio,
            r.method_name,
            r.omega,
            r.converged,
            r.iterations,
            r.spectral_radius_est,
            r.final_rel_error,
            r.max_disp_bemfem,
            r.max_disp_pure_fem,
            if r.rel_l2_disp_error.is_nan() { -1.0 } else { r.rel_l2_disp_error },
            r.cpu_time_ms
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "static_contrast_test.rs"]
mod tests;
