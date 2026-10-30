use crate::material::strategies::DirectionalGradientField;
use crate::material::types::{Axis, GradientForm, ReferenceMaterial};
use crate::mesh::generators_2d::generate_beam_2d;
use crate::solver::decoupled_hybrid::solve_decoupled_bem_fem_hybrid;
use crate::solver::fem_solver::solve_pure_fem_with_stress_state;
use crate::solver::types::{HybridAcceleration, StressState2D};

#[derive(Debug, Clone)]
pub struct HybridRunRecord {
    pub alpha: f64,
    pub bnd_desc: String,
    pub bnd_elems: usize,
    pub fem_desc: String,
    pub fem_nx: usize,
    pub fem_ny: usize,
    pub fem_dofs: usize,
    pub w_tip: f64,
    pub err_tip_pct: f64,
    pub iterations: usize,
    pub pred_time_ms: f64,
    pub corr_time_ms: f64,
    pub total_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct PureFemRecord {
    pub alpha: f64,
    pub fem_desc: String,
    pub fem_nx: usize,
    pub fem_ny: usize,
    pub fem_dofs: usize,
    pub w_tip: f64,
    pub err_tip_pct: f64,
    pub solve_time_ms: f64,
}

pub fn run_mini_paper_parametric_campaign() -> String {
    let length = 6.0;
    let height = 2.0;
    let p_load = 50_000.0;
    let stress_state = StressState2D::PlaneStress;

    let ref_mat = ReferenceMaterial {
        e0: 2.1e11, // 210 GPa
        nu0: 0.25,
        rho0: 7850.0,
    };

    let alpha_values = [0.0, 0.20, 0.60, 1.20];
    let bnd_configs = [
        ("Coarse (12 elems)", 4, 2, 12),
        ("Medium (24 elems)", 8, 4, 24),
        ("Fine (48 elems)", 16, 8, 48),
    ];
    let fem_configs = [
        ("6x2", 6, 2),
        ("12x4", 12, 4),
        ("24x8", 24, 8),
        ("48x16", 48, 16),
    ];

    let mut report = String::new();
    report.push_str("# MINI-PAPER BENCHMARK SUITE: DECOUPLED BEM-FEM OPERATOR SPLITTING\n\n");
    report.push_str("Cantilever Beam: $L=6.0\\text{ m}, H=2.0\\text{ m}, P=50\\text{ kN}$ (Shear Load)\n");
    report.push_str("Formulation: 2D Plane Stress, Linear Directional Gradient $E(y) = E_0 [1 + \\alpha (y/H)]$\n");
    report.push_str("Ground Truth: Overkill Standalone FEM $96\\times 32$ (3,072 Quad4 elements, 6,402 DOFs)\n\n");

    for &alpha in &alpha_values {
        eprintln!("\n>>> Running Mini-Paper Parametric Suite for alpha = {:.2}...", alpha);
        let strategy = DirectionalGradientField::new(
            Axis::Y,
            alpha,
            height,
            1.0,
            GradientForm::Linear,
        );

        // 1. Compute Overkill Ground Truth FEM (96x32)
        let mesh_ref = generate_beam_2d(length, height, 96, 32, false);
        let res_ref = solve_pure_fem_with_stress_state(&mesh_ref, &ref_mat, &strategy, p_load, stress_state);

        // Find tip center node in reference mesh
        let mut ref_tip_idx = 0;
        let mut min_d = 1e9;
        for (idx, p) in mesh_ref.nodes.iter().enumerate() {
            let d = ((p[0] - length).powi(2) + (p[1] - height * 0.5).powi(2)).sqrt();
            if d < min_d {
                min_d = d;
                ref_tip_idx = idx;
            }
        }
        let w_ref = res_ref.displacements[ref_tip_idx][1].abs();

        report.push_str(&format!(
            "## Inhomogeneity $\\alpha = {:.2}$ (Contrast ratio: {:.2}x, Ground Truth $w_{{\\text{{tip}}}} = {:.6}\\text{{ mm}}$, solve time: {:.2}\\text{{ ms}})\n\n",
            alpha, 1.0 + alpha, w_ref * 1000.0, res_ref.execution_time_ms
        ));

        // 2. Compute Standalone Pure FEM for each coarse corrector mesh
        let mut pure_fem_records = Vec::new();
        for &(fem_label, nx, ny) in &fem_configs {
            let mesh = generate_beam_2d(length, height, nx, ny, false);
            let res_fem = solve_pure_fem_with_stress_state(&mesh, &ref_mat, &strategy, p_load, stress_state);
            let mut tip_idx = 0;
            let mut d_min = 1e9;
            for (idx, p) in mesh.nodes.iter().enumerate() {
                let d = ((p[0] - length).powi(2) + (p[1] - height * 0.5).powi(2)).sqrt();
                if d < d_min {
                    d_min = d;
                    tip_idx = idx;
                }
            }
            let w_fem = res_fem.displacements[tip_idx][1].abs();
            let err_pct = ((w_fem - w_ref) / w_ref).abs() * 100.0;
            pure_fem_records.push(PureFemRecord {
                alpha,
                fem_desc: fem_label.to_string(),
                fem_nx: nx,
                fem_ny: ny,
                fem_dofs: mesh.nodes.len() * 2,
                w_tip: w_fem,
                err_tip_pct: err_pct,
                solve_time_ms: res_fem.execution_time_ms,
            });
        }

        // 3. Compute Decoupled Hybrid BEM-FEM runs (3 BEM x 4 FEM = 12 combinations)
        let mut hybrid_grid = Vec::new();

        for &(bnd_label, b_nx, b_ny, b_total) in &bnd_configs {
            let mut row_records = Vec::new();

            for &(fem_label, f_nx, f_ny) in &fem_configs {
                let fem_mesh = generate_beam_2d(length, height, f_nx, f_ny, false);
                let (res_hyb, t_pred, t_corr) = solve_decoupled_bem_fem_hybrid(
                    length,
                    height,
                    b_nx,
                    b_ny,
                    &fem_mesh,
                    &ref_mat,
                    &strategy,
                    p_load,
                    50,
                    1e-4,
                    HybridAcceleration::Anderson { depth: 3, beta: 0.70 },
                    stress_state,
                );

                let mut tip_idx = 0;
                let mut d_min = 1e9;
                for (idx, p) in fem_mesh.nodes.iter().enumerate() {
                    let d = ((p[0] - length).powi(2) + (p[1] - height * 0.5).powi(2)).sqrt();
                    if d < d_min {
                        d_min = d;
                        tip_idx = idx;
                    }
                }
                let w_hyb = res_hyb.displacements[tip_idx][1].abs();
                let err_pct = ((w_hyb - w_ref) / w_ref).abs() * 100.0;

                row_records.push(HybridRunRecord {
                    alpha,
                    bnd_desc: bnd_label.to_string(),
                    bnd_elems: b_total,
                    fem_desc: fem_label.to_string(),
                    fem_nx: f_nx,
                    fem_ny: f_ny,
                    fem_dofs: fem_mesh.nodes.len() * 2,
                    w_tip: w_hyb,
                    err_tip_pct: err_pct,
                    iterations: res_hyb.iterations,
                    pred_time_ms: t_pred,
                    corr_time_ms: t_corr,
                    total_time_ms: res_hyb.execution_time_ms,
                });
            }
            hybrid_grid.push((bnd_label, row_records));
        }

        // Table Output: Error (e%), Iterations (iter), Total Time (ms)
        report.push_str("| BEM Boundary ($N_\\Gamma$) | FEM $6\\times 2$ (42 DOFs) | FEM $12\\times 4$ (130 DOFs) | FEM $24\\times 8$ (450 DOFs) | FEM $48\\times 16$ (1666 DOFs) |\n");
        report.push_str("| :--- | :---: | :---: | :---: | :---: |\n");

        for (bnd_label, row) in &hybrid_grid {
            report.push_str(&format!("| **{}** | ", bnd_label));
            for cell in row {
                report.push_str(&format!(
                    "**{:.2}%** ({}, {:.1}ms) | ",
                    cell.err_tip_pct, cell.iterations, cell.total_time_ms
                ));
            }
            report.push_str("\n");
        }

        // Pure FEM comparison row
        report.push_str("| *Pure Standalone FEM* | ");
        for fem in &pure_fem_records {
            report.push_str(&format!(
                "*{:.2}%* (1, {:.1}ms) | ",
                fem.err_tip_pct, fem.solve_time_ms
            ));
        }
        report.push_str("\n\n");
    }

    report
}

#[cfg(test)]
#[path = "mini_paper_suite_test.rs"]
mod tests;
