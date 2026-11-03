use nalgebra::{DMatrix, DVector, SymmetricEigen};
use std::collections::HashMap;

use super::element_stiffness::{
    condense_releases, element_transformation_matrix, local_frame_stiffness,
};
use super::linear_solver::{assemble_linear_system, AssembledSystem};
use super::types::*;

/// Assembles the global lumped mass matrix (3N x 3N) and partitions it into M_ff (Nf x Nf)
pub fn assemble_mass_matrix(
    model: &StructuralModel,
    sys: &AssembledSystem,
    load_cases_mass: Option<&[(&str, f64)]>,
) -> (DMatrix<f64>, DMatrix<f64>) {
    let mut node_idx_map: HashMap<&str, usize> = HashMap::new();
    for (i, node) in model.nodes.iter().enumerate() {
        node_idx_map.insert(node.id.as_str(), i);
    }

    let total_dofs = sys.total_dofs;
    let mut m_global = DMatrix::<f64>::zeros(total_dofs, total_dofs);
    let g = if model.gravity_g > 0.0 { model.gravity_g } else { 9.81 };

    // 1. Explicit lumped masses from model
    for lm in &model.lumped_masses {
        if let Some(&ni) = node_idx_map.get(lm.node_id.as_str()) {
            m_global[(ni * 3, ni * 3)] += lm.mass_x_t;
            m_global[(ni * 3 + 1, ni * 3 + 1)] += lm.mass_y_t;
            m_global[(ni * 3 + 2, ni * 3 + 2)] += lm.inertia_theta_tm2;
        }
    }

    // 2. Element self-weight masses (distributed half-and-half to end nodes)
    let sec_map: HashMap<&str, &Section> = model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &Material> = model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    for el in &model.elements {
        match el {
            Element::FrameMember {
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            }
            | Element::TrussBar {
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            }
            | Element::Cable {
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } => {
                if let (Some(&ni), Some(&nj)) = (
                    node_idx_map.get(node_i.as_str()),
                    node_idx_map.get(node_j.as_str()),
                ) {
                    let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
                    let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
                    let l = (xj - xi).hypot(yj - yi).max(1e-4);

                    if let (Some(sec), Some(mat)) = (sec_map.get(section_id.as_str()), mat_map.get(material_id.as_str())) {
                        let total_weight_kn = mat.density_gamma * sec.area_a * l;
                        let total_mass_t = total_weight_kn / g;
                        let half_mass_t = 0.5 * total_mass_t;

                        m_global[(ni * 3, ni * 3)] += half_mass_t;
                        m_global[(ni * 3 + 1, ni * 3 + 1)] += half_mass_t;
                        m_global[(nj * 3, nj * 3)] += half_mass_t;
                        m_global[(nj * 3 + 1, nj * 3 + 1)] += half_mass_t;
                    }
                }
            }
            Element::Spring { .. } => {}
        }
    }

    // 3. Load case mass rule (e.g. G + psi2 * Q)
    if let Some(lc_masses) = load_cases_mass {
        let lc_map: HashMap<&str, &LoadCase> =
            model.load_cases.iter().map(|lc| (lc.id.as_str(), lc)).collect();

        for &(lc_id, factor) in lc_masses {
            if let Some(lc) = lc_map.get(lc_id) {
                for load in &lc.loads {
                    if let Load::NodalLoad { node_id, fy, .. } = load {
                        if let Some(&ni) = node_idx_map.get(node_id.as_str()) {
                            let vertical_load_kn = fy.abs() * factor;
                            let mass_t = vertical_load_kn / g;
                            m_global[(ni * 3, ni * 3)] += mass_t;
                            m_global[(ni * 3 + 1, ni * 3 + 1)] += mass_t;
                        }
                    }
                }
            }
        }
    }

    // Ensure rotational DOFs have a small regularization mass if massless to avoid singularity
    for ni in 0..model.nodes.len() {
        let m_trans = m_global[(ni * 3, ni * 3)].max(m_global[(ni * 3 + 1, ni * 3 + 1)]);
        if m_global[(ni * 3 + 2, ni * 3 + 2)] < 1e-7 && m_trans > 1e-6 {
            // Representative radius of gyration ~ 0.5 m => I_rot = m * r^2 * 1e-4
            m_global[(ni * 3 + 2, ni * 3 + 2)] = (m_trans * 0.25 * 1e-3).max(1e-6);
        }
    }

    // Partition into M_ff
    let nf = sys.free_dof_indices.len();
    let mut m_ff = DMatrix::<f64>::zeros(nf, nf);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            m_ff[(i, j)] = m_global[(fi, fj)];
        }
    }

    (m_global, m_ff)
}

/// Evaluates EN 1998-1 Elastic / Design Spectrum Acceleration Sd(T) in m/s^2 (or g-fractions)
pub fn evaluate_en1998_design_spectrum(
    t: f64,
    ag_m_s2: f64,
    ground_type: &str,
    q: f64,
    beta: f64,
) -> f64 {
    let (s, tb, tc, td) = match ground_type.to_uppercase().as_str() {
        "A" => (1.0, 0.15, 0.4, 2.0),
        "B" => (1.2, 0.15, 0.5, 2.0),
        "C" => (1.15, 0.20, 0.6, 2.0),
        "D" => (1.35, 0.20, 0.8, 2.0),
        "E" => (1.40, 0.15, 0.5, 2.0),
        _ => (1.0, 0.15, 0.4, 2.0),
    };

    let q_eff = q.max(1.0);
    let beta_lower = beta.max(0.2);

    if t < 0.0 {
        return 0.0;
    }

    if t <= tb {
        ag_m_s2 * s * (2.0 / 3.0 + (t / tb) * (2.5 / q_eff - 2.0 / 3.0))
    } else if t <= tc {
        ag_m_s2 * s * (2.5 / q_eff)
    } else if t <= td {
        let val = ag_m_s2 * s * (2.5 / q_eff) * (tc / t);
        val.max(beta_lower * ag_m_s2)
    } else {
        let val = ag_m_s2 * s * (2.5 / q_eff) * ((tc * td) / (t * t));
        val.max(beta_lower * ag_m_s2)
    }
}

/// Computes modal properties and extracts vibration modes up to cumulative mass >= target_ratio (default 0.90)
pub fn solve_modal_analysis(
    model: &StructuralModel,
    target_mass_ratio: f64,
) -> Result<DynamicAnalysisData, CanonicalError> {
    let sys = assemble_linear_system(model)?;
    let nf = sys.free_dof_indices.len();
    if nf == 0 {
        return Err(CanonicalError::new(
            "NO_FREE_DOFS_FOR_MODAL",
            "Δεν υπάρχουν ελεύθεροι βαθμοί ελευθερίας για ιδιομορφική ανάλυση.",
            "No free degrees of freedom available for modal analysis.",
        ));
    }

    let (m_global, m_ff) = assemble_mass_matrix(model, &sys, None);

    // Total translational masses
    let mut total_mass_x = 0.0;
    let mut total_mass_y = 0.0;
    for i in 0..model.nodes.len() {
        total_mass_x += m_global[(i * 3, i * 3)];
        total_mass_y += m_global[(i * 3 + 1, i * 3 + 1)];
    }
    if total_mass_x < 1e-9 {
        total_mass_x = 1.0;
    }
    if total_mass_y < 1e-9 {
        total_mass_y = 1.0;
    }

    // Symmetrized generalized eigenvalue problem:
    // (K_ff - omega^2 * M_ff) * phi = 0
    // M_ff is diagonal (or diagonally dominant lumped). Form M^(-1/2) * K_ff * M^(-1/2)
    let mut inv_sqrt_m = DMatrix::<f64>::zeros(nf, nf);
    for i in 0..nf {
        let m_ii = m_ff[(i, i)].max(1e-8);
        inv_sqrt_m[(i, i)] = 1.0 / m_ii.sqrt();
    }

    let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_ff[(i, j)] = sys.k_global[(fi, fj)];
        }
    }

    let k_tilde = &inv_sqrt_m * &k_ff * &inv_sqrt_m;
    let eigen = SymmetricEigen::new(k_tilde);

    let mut eigen_pairs: Vec<(f64, DVector<f64>)> = Vec::with_capacity(nf);
    for (i, &lambda) in eigen.eigenvalues.iter().enumerate() {
        if lambda > 1e-6 {
            // Eigenvector phi_f = M^(-1/2) * v_i
            let v_col = eigen.eigenvectors.column(i);
            let phi_f = &inv_sqrt_m * &v_col;
            eigen_pairs.push((lambda, phi_f));
        }
    }

    // Sort by ascending eigenvalue (lowest frequency first)
    eigen_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Influence vectors for free DOFs
    let mut rx_f = DVector::<f64>::zeros(nf);
    let mut ry_f = DVector::<f64>::zeros(nf);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        if fi % 3 == 0 {
            rx_f[i] = 1.0;
        } else if fi % 3 == 1 {
            ry_f[i] = 1.0;
        }
    }

    let mut modes: Vec<VibrationMode> = Vec::new();
    let mut cum_ratio_x = 0.0;
    let mut cum_ratio_y = 0.0;

    for (idx, (lambda, phi_f)) in eigen_pairs.iter().enumerate() {
        let omega = lambda.sqrt();
        let freq_hz = omega / (2.0 * std::f64::consts::PI);
        let period_s = if freq_hz > 1e-9 { 1.0 / freq_hz } else { 0.0 };

        // Generalized mass: phi_f^T * M_ff * phi_f
        let gen_mass = phi_f.dot(&(&m_ff * phi_f)).max(1e-9);

        // Participation factors
        let l_x = phi_f.dot(&(&m_ff * &rx_f));
        let l_y = phi_f.dot(&(&m_ff * &ry_f));

        let gamma_x = l_x / gen_mass;
        let gamma_y = l_y / gen_mass;

        // Modal effective masses
        let modal_mass_x = (l_x * l_x) / gen_mass;
        let modal_mass_y = (l_y * l_y) / gen_mass;

        let eff_ratio_x = modal_mass_x / total_mass_x;
        let eff_ratio_y = modal_mass_y / total_mass_y;

        cum_ratio_x += eff_ratio_x;
        cum_ratio_y += eff_ratio_y;

        // Full nodal displacement representation
        let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
        for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
            u_full[fi] = phi_f[i];
        }

        // Normalize mode shape by maximum translation
        let mut max_disp = 1.0f64;
        for i in 0..model.nodes.len() {
            let d = u_full[i * 3].hypot(u_full[i * 3 + 1]);
            if d > max_disp {
                max_disp = d;
            }
        }
        let scale = 1.0 / max_disp;

        let mut node_displacements = Vec::with_capacity(model.nodes.len());
        for (i, node) in model.nodes.iter().enumerate() {
            node_displacements.push(NodeDisplacement {
                node_id: node.id.clone(),
                dux: u_full[i * 3] * scale,
                duy: u_full[i * 3 + 1] * scale,
                dtheta: u_full[i * 3 + 2] * scale,
            });
        }

        modes.push(VibrationMode {
            mode_index: idx + 1,
            omega_rad_s: omega,
            frequency_hz: freq_hz,
            period_s,
            gamma_x,
            gamma_y,
            modal_mass_x_t: modal_mass_x,
            modal_mass_y_t: modal_mass_y,
            effective_mass_ratio_x: eff_ratio_x,
            effective_mass_ratio_y: eff_ratio_y,
            cumulative_mass_ratio_x: cum_ratio_x,
            cumulative_mass_ratio_y: cum_ratio_y,
            node_displacements,
        });

        // Break if we've satisfied the cumulative target effective mass ratio
        if cum_ratio_x >= target_mass_ratio && (idx + 1) >= 3 {
            break;
        }
    }

    Ok(DynamicAnalysisData {
        modes,
        base_shear_x_kn: 0.0,
        base_shear_y_kn: 0.0,
        combination_rule: None,
    })
}

/// Solves Equivalent Static Lateral Force method according to EN 1998-1 §4.3.3.2
pub fn solve_lateral_force(
    model: &StructuralModel,
    spectrum_ag_m_s2: f64,
    ground_type: &str,
    q: f64,
) -> Result<ResultSet, CanonicalError> {
    let sys = assemble_linear_system(model)?;
    let modal_data = solve_modal_analysis(model, 0.90)?;

    if modal_data.modes.is_empty() {
        return Err(CanonicalError::new(
            "NO_MODES_FOR_LATERAL_FORCE",
            "Δεν εντοπίστηκαν ιδιομορφές για την μέθοδο πλευρικών φορτίων.",
            "No vibration modes identified for lateral force method.",
        ));
    }

    let t1 = modal_data.modes[0].period_s;
    let (m_global, _) = assemble_mass_matrix(model, &sys, None);

    // Total active mass
    let mut total_mass = 0.0;
    for i in 0..model.nodes.len() {
        total_mass += m_global[(i * 3, i * 3)];
    }
    if total_mass < 1e-4 {
        total_mass = 1.0;
    }

    // Spectral acceleration Sd(T1)
    let sd = evaluate_en1998_design_spectrum(t1, spectrum_ag_m_s2, ground_type, q, 0.2);

    // Correction factor lambda = 0.85 if T1 <= 2*TC and > 2 storeys, else 1.0
    let lambda = 0.85;
    // Base shear Fb = Sd(T1) * m * lambda
    let fb_kn = sd * total_mass * lambda;

    // Height distribution: F_i = F_b * (z_i * m_i) / sum(z_j * m_j)
    let min_y = model.nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);

    let mut num_sum = 0.0;
    let mut zi_mi_factors = Vec::with_capacity(model.nodes.len());
    for (i, node) in model.nodes.iter().enumerate() {
        let zi = (node.y - min_y).max(0.0);
        let mi = m_global[(i * 3, i * 3)];
        let prod = zi * mi;
        num_sum += prod;
        zi_mi_factors.push(prod);
    }
    if num_sum < 1e-6 {
        num_sum = 1.0;
    }

    let mut f_lat = DVector::<f64>::zeros(sys.total_dofs);
    for (i, prod) in zi_mi_factors.iter().enumerate() {
        let f_i = fb_kn * (prod / num_sum);
        f_lat[i * 3] += f_i;
    }

    // Solve static system: K_ff * u_f = F_lat_f
    let mut f_lat_f = DVector::<f64>::zeros(sys.free_dof_indices.len());
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        f_lat_f[i] = f_lat[fi];
    }

    let nf = sys.free_dof_indices.len();
    let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_ff[(i, j)] = sys.k_global[(fi, fj)];
        }
    }

    let u_f = k_ff.clone().cholesky().map(|chol| chol.solve(&f_lat_f)).or_else(|| {
        let svd = k_ff.svd(true, true);
        svd.solve(&f_lat_f, 1e-8).ok()
    }).ok_or_else(|| {
        CanonicalError::new(
            "LATERAL_FORCE_SOLVE_FAILED",
            "Αποτυχία επίλυσης συστήματος πλευρικών φορτίων.",
            "Failed to solve lateral force linear system.",
        )
    })?;

    let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        u_full[fi] = u_f[i];
    }

    let r_full = &sys.k_global * &u_full - &f_lat;

    // Nodal displacements
    let mut nodal_displacements = Vec::with_capacity(model.nodes.len());
    for (i, node) in model.nodes.iter().enumerate() {
        nodal_displacements.push(NodalDisplacementResult {
            node_id: node.id.clone(),
            ux_m: u_full[i * 3],
            uy_m: u_full[i * 3 + 1],
            theta_rad: u_full[i * 3 + 2],
        });
    }

    // Support reactions
    let mut support_reactions = Vec::new();
    let node_idx_map: HashMap<&str, usize> = model.nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    for sup in &model.supports {
        if let Some(&ni) = node_idx_map.get(sup.node_id.as_str()) {
            support_reactions.push(SupportReactionResult {
                support_id: sup.id.clone(),
                node_id: sup.node_id.clone(),
                rx_kn: r_full[ni * 3],
                ry_kn: r_full[ni * 3 + 1],
                mz_knm: r_full[ni * 3 + 2],
            });
        }
    }

    // Member diagrams
    let sec_map: HashMap<&str, &Section> = model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &Material> = model.materials.iter().map(|m| (m.id.as_str(), m)).collect();
    let mut member_results = Vec::new();

    for el in &model.elements {
        if let Element::FrameMember {
            id,
            node_i,
            node_j,
            section_id,
            material_id,
            release_i,
            release_j,
            offset_i,
            offset_j,
        } = el
        {
            let ni = node_idx_map[node_i.as_str()];
            let nj = node_idx_map[node_j.as_str()];
            let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
            let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
            let dx = xj - xi;
            let dy = yj - yi;
            let l_nom = dx.hypot(dy).max(1e-6);
            let cos_a = dx / l_nom;
            let sin_a = dy / l_nom;
            let l = l_nom;

            let sec = sec_map[section_id.as_str()];
            let mat = mat_map[material_id.as_str()];

            let u_el_global = DVector::from_column_slice(&[
                u_full[ni * 3],
                u_full[ni * 3 + 1],
                u_full[ni * 3 + 2],
                u_full[nj * 3],
                u_full[nj * 3 + 1],
                u_full[nj * 3 + 2],
            ]);

            let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
            let u_loc = &t * &u_el_global;

            let k_loc = local_frame_stiffness(mat.e_gpa * 1e6, sec.area_a, sec.inertia_i, l);
            let (k_cond, _) = condense_releases(&k_loc, &[0.0; 6], release_i, release_j);
            let f_loc = &k_cond * &u_loc;

            let mr = super::diagrams::evaluate_member_diagram(
                id,
                l,
                mat.e_gpa * 1e6,
                sec.area_a,
                sec.inertia_i,
                cos_a,
                sin_a,
                &[u_loc[0], u_loc[1], u_loc[2], u_loc[3], u_loc[4], u_loc[5]],
                &[f_loc[0], f_loc[1], f_loc[2], f_loc[3], f_loc[4], f_loc[5]],
                &[],
                false,
                mat.density_gamma,
            );
            member_results.push(mr);
        }
    }

    Ok(ResultSet {
        revision: model.revision,
        target_id: "LATERAL_FORCE".to_string(),
        target_kind: "Seismic".to_string(),
        analysis_kind: AnalysisKind::LateralForce,
        superposition_valid: false,
        nodal_displacements,
        support_reactions,
        member_results,
        cable_results: Vec::new(),
        buckling_modes: Vec::new(),
        dynamic_data: Some(DynamicAnalysisData {
            modes: modal_data.modes,
            base_shear_x_kn: fb_kn,
            base_shear_y_kn: 0.0,
            combination_rule: Some("LateralForce".to_string()),
        }),
    })
}

/// Der Kiureghian CQC correlation coefficient rho_ij
pub fn der_kiureghian_correlation(omega_i: f64, omega_j: f64, damping_ratio: f64) -> f64 {
    let xi = damping_ratio.clamp(0.01, 0.20);
    let r = (omega_j / omega_i.max(1e-9)).clamp(1e-4, 1e4);

    let num = 8.0 * xi * xi * (1.0 + r) * r.powf(1.5);
    let den = (1.0 - r * r).powi(2) + 4.0 * xi * xi * r * (1.0 + r).powi(2);
    (num / den.max(1e-12)).clamp(0.0, 1.0)
}

/// Solves Modal Response Spectrum Analysis (SRSS and CQC)
pub fn solve_response_spectrum(
    model: &StructuralModel,
    spectrum_ag_m_s2: f64,
    ground_type: &str,
    q: f64,
    damping_ratio: f64,
    use_cqc: bool,
) -> Result<ResultSet, CanonicalError> {
    let modal_data = solve_modal_analysis(model, 0.90)?;
    let modes = &modal_data.modes;
    let num_modes = modes.len();
    if num_modes == 0 {
        return Err(CanonicalError::new(
            "NO_MODES_FOR_RSA",
            "Δεν εντοπίστηκαν ιδιομορφές για ανάλυση φάσματος απόκρισης.",
            "No modes found for response spectrum analysis.",
        ));
    }

    // Compute peak response for each mode k
    // u_k = Gamma_x,k * (Sd(Tk) / omega_k^2) * phi_k
    let num_nodes = model.nodes.len();
    let mut modal_displacements = Vec::with_capacity(num_modes);
    let mut modal_base_shears_x = Vec::with_capacity(num_modes);

    for m in modes {
        let sd = evaluate_en1998_design_spectrum(m.period_s, spectrum_ag_m_s2, ground_type, q, 0.2);
        let omega2 = m.omega_rad_s.powi(2).max(1e-4);
        let spectral_disp = sd / omega2;

        let mut node_u = Vec::with_capacity(num_nodes);
        for nd in &m.node_displacements {
            node_u.push(NodalDisplacementResult {
                node_id: nd.node_id.clone(),
                ux_m: m.gamma_x * spectral_disp * nd.dux,
                uy_m: m.gamma_x * spectral_disp * nd.duy,
                theta_rad: m.gamma_x * spectral_disp * nd.dtheta,
            });
        }

        // Peak modal base shear: V_k = m_eff,x,k * Sd(Tk)
        let vk = m.modal_mass_x_t * sd;
        modal_base_shears_x.push(vk);
        modal_displacements.push(node_u);
    }

    // Modal combinations (SRSS or CQC)
    let mut total_nodal_displacements = Vec::with_capacity(num_nodes);

    for ni in 0..num_nodes {
        let node_id = model.nodes[ni].id.clone();
        let mut ux_comb2 = 0.0;
        let mut uy_comb2 = 0.0;
        let mut th_comb2 = 0.0;

        for i in 0..num_modes {
            for j in 0..num_modes {
                let rho = if use_cqc {
                    der_kiureghian_correlation(modes[i].omega_rad_s, modes[j].omega_rad_s, damping_ratio)
                } else if i == j {
                    1.0
                } else {
                    0.0
                };

                let uxi = modal_displacements[i][ni].ux_m;
                let uxj = modal_displacements[j][ni].ux_m;
                ux_comb2 += uxi * rho * uxj;

                let uyi = modal_displacements[i][ni].uy_m;
                let uyj = modal_displacements[j][ni].uy_m;
                uy_comb2 += uyi * rho * uyj;

                let thi = modal_displacements[i][ni].theta_rad;
                let thj = modal_displacements[j][ni].theta_rad;
                th_comb2 += thi * rho * thj;
            }
        }

        total_nodal_displacements.push(NodalDisplacementResult {
            node_id,
            ux_m: ux_comb2.max(0.0).sqrt(),
            uy_m: uy_comb2.max(0.0).sqrt(),
            theta_rad: th_comb2.max(0.0).sqrt(),
        });
    }

    // Combine base shear
    let mut base_shear_comb2 = 0.0;
    for i in 0..num_modes {
        for j in 0..num_modes {
            let rho = if use_cqc {
                der_kiureghian_correlation(modes[i].omega_rad_s, modes[j].omega_rad_s, damping_ratio)
            } else if i == j {
                1.0
            } else {
                0.0
            };
            base_shear_comb2 += modal_base_shears_x[i] * rho * modal_base_shears_x[j];
        }
    }
    let total_base_shear_x = base_shear_comb2.max(0.0).sqrt();

    // Support reactions from base shear
    let mut support_reactions = Vec::new();
    let num_sups = model.supports.len().max(1) as f64;
    for sup in &model.supports {
        support_reactions.push(SupportReactionResult {
            support_id: sup.id.clone(),
            node_id: sup.node_id.clone(),
            rx_kn: total_base_shear_x / num_sups,
            ry_kn: 0.0,
            mz_knm: 0.0,
        });
    }

    Ok(ResultSet {
        revision: model.revision,
        target_id: if use_cqc { "RSA_CQC".to_string() } else { "RSA_SRSS".to_string() },
        target_kind: "ResponseSpectrum".to_string(),
        analysis_kind: AnalysisKind::ResponseSpectrum,
        superposition_valid: false,
        nodal_displacements: total_nodal_displacements,
        support_reactions,
        member_results: Vec::new(),
        cable_results: Vec::new(),
        buckling_modes: Vec::new(),
        dynamic_data: Some(DynamicAnalysisData {
            modes: modal_data.modes,
            base_shear_x_kn: total_base_shear_x,
            base_shear_y_kn: 0.0,
            combination_rule: Some(if use_cqc { "CQC".to_string() } else { "SRSS".to_string() }),
        }),
    })
}
