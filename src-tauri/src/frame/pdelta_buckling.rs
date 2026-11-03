use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use super::element_stiffness::{
    condense_releases, element_transformation_matrix, local_frame_stiffness,
};
use super::linear_solver::{assemble_linear_system, assemble_load_vector, solve_linear_static};
use super::types::{
    AnalysisKind, BucklingMode, CanonicalError, Combination, Element, NodeDisplacement, ResultSet,
    StructuralModel,
};

/// Assembles 6x6 local geometric stiffness matrix k_G for Euler-Bernoulli beam under axial force N
/// N > 0 in tension, N < 0 in compression
pub fn local_geometric_stiffness(n_axial: f64, l: f64) -> DMatrix<f64> {
    let mut kg = DMatrix::<f64>::zeros(6, 6);
    let n_l = n_axial / l;

    kg[(1, 1)] = 1.2 * n_l;
    kg[(1, 2)] = 0.1 * l * n_l;
    kg[(1, 4)] = -1.2 * n_l;
    kg[(1, 5)] = 0.1 * l * n_l;

    kg[(2, 1)] = 0.1 * l * n_l;
    kg[(2, 2)] = (2.0 / 15.0) * l * l * n_l;
    kg[(2, 4)] = -0.1 * l * n_l;
    kg[(2, 5)] = -(1.0 / 30.0) * l * l * n_l;

    kg[(4, 1)] = -1.2 * n_l;
    kg[(4, 2)] = -0.1 * l * n_l;
    kg[(4, 4)] = 1.2 * n_l;
    kg[(4, 5)] = -0.1 * l * n_l;

    kg[(5, 1)] = 0.1 * l * n_l;
    kg[(5, 2)] = -(1.0 / 30.0) * l * l * n_l;
    kg[(5, 4)] = -0.1 * l * n_l;
    kg[(5, 5)] = (2.0 / 15.0) * l * l * n_l;

    kg
}

fn get_target_load_vector(
    model: &StructuralModel,
    sys: &super::linear_solver::AssembledSystem,
    combinations: &[Combination],
    target_id: &str,
) -> DVector<f64> {
    if let Some(comb) = combinations.iter().find(|c| c.id == target_id) {
        let mut f_comb = DVector::<f64>::zeros(sys.total_dofs);
        for factor_entry in &comb.factors {
            let f_lc = assemble_load_vector(model, sys, &factor_entry.load_case_id);
            f_comb += factor_entry.factor * f_lc;
        }
        f_comb
    } else {
        assemble_load_vector(model, sys, target_id)
    }
}

/// Solves 2nd-order P-Delta analysis per combination
pub fn solve_second_order_pdelta(
    model: &StructuralModel,
    combinations: &[Combination],
) -> Result<Vec<ResultSet>, CanonicalError> {
    let base_res_list = solve_linear_static(model, combinations)?;
    let sys = assemble_linear_system(model)?;
    let nf = sys.free_dof_indices.len();
    let ns = sys.restrained_dof_indices.len();

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }
    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    let mut pdelta_results = Vec::new();

    for mut res in base_res_list {
        let target_id = res.target_id.clone();
        let max_iter = 50;
        let tol = 1e-5;

        // Current displacement vector u_full
        let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
        for d in &res.nodal_displacements {
            let ni = node_idx_map[d.node_id.as_str()];
            u_full[ni * 3] = d.ux_m;
            u_full[ni * 3 + 1] = d.uy_m;
            u_full[ni * 3 + 2] = d.theta_rad;
        }

        let f_global = get_target_load_vector(model, &sys, combinations, &target_id);
        let mut f_f = DVector::<f64>::zeros(nf);
        for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
            f_f[i] = f_global[fi];
        }

        let mut converged = false;
        let mut last_k_t_global = DMatrix::<f64>::zeros(sys.total_dofs, sys.total_dofs);

        for _iter in 0..max_iter {
            // 1. Calculate axial force N in each member from current u_full
            let mut k_g_global = DMatrix::<f64>::zeros(sys.total_dofs, sys.total_dofs);

            for el in &model.elements {
                if let Element::FrameMember {
                    node_i,
                    node_j,
                    section_id,
                    material_id,
                    release_i,
                    release_j,
                    offset_i,
                    offset_j,
                    ..
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

                    let (dxi, _) = match offset_i.frame {
                        super::types::OffsetFrame::Local => (offset_i.dx, offset_i.dy),
                        super::types::OffsetFrame::Global => (
                            offset_i.dx * cos_a + offset_i.dy * sin_a,
                            -offset_i.dx * sin_a + offset_i.dy * cos_a,
                        ),
                    };
                    let (dxj, _) = match offset_j.frame {
                        super::types::OffsetFrame::Local => (offset_j.dx, offset_j.dy),
                        super::types::OffsetFrame::Global => (
                            offset_j.dx * cos_a + offset_j.dy * sin_a,
                            -offset_j.dx * sin_a + offset_j.dy * cos_a,
                        ),
                    };
                    let l_eff = (l_nom - dxi + dxj).max(1e-4);

                    let sec = sec_map[section_id.as_str()];
                    let mat = mat_map[material_id.as_str()];
                    let ea = mat.e_gpa * 1e6 * sec.area_a;

                    let dofs = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
                    let mut u_el = DVector::<f64>::zeros(6);
                    for (a, &d) in dofs.iter().enumerate() {
                        u_el[a] = u_full[d];
                    }
                    let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
                    let u_loc = &t * &u_el;

                    let delta_u_axial = u_loc[3] - u_loc[0];
                    let n_axial = (ea / l_eff) * delta_u_axial;

                    let k_g_loc = local_geometric_stiffness(n_axial, l_eff);
                    let (k_g_cond, _) = condense_releases(&k_g_loc, &[0.0; 6], release_i, release_j);
                    let k_g_elem_global = t.transpose() * &k_g_cond * &t;

                    for (a, &da) in dofs.iter().enumerate() {
                        for (b, &db) in dofs.iter().enumerate() {
                            k_g_global[(da, db)] += k_g_elem_global[(a, b)];
                        }
                    }
                }
            }

            // Total tangent stiffness: K_T = K_E + K_G
            let k_t_global = &sys.k_global + &k_g_global;
            last_k_t_global = k_t_global.clone();

            let mut k_t_ff = DMatrix::<f64>::zeros(nf, nf);
            let mut k_t_fs = DMatrix::<f64>::zeros(nf, ns);
            for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
                for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
                    k_t_ff[(i, j)] = k_t_global[(fi, fj)];
                }
                for (j, &sj) in sys.restrained_dof_indices.iter().enumerate() {
                    k_t_fs[(i, j)] = k_t_global[(fi, sj)];
                }
            }

            // Check positive definiteness
            let cholesky = k_t_ff.cholesky().ok_or_else(|| {
                CanonicalError::new(
                    "PDELTA_BUCKLING_DIVERGENCE",
                    "Ο φορέας απώλεσε την ευστάθειά του υπό τα εφαρμοζόμενα φορτία (λυγισμός 2ης τάξης P-Δ).",
                    "Structure buckled under applied compressive loads (P-Delta divergence).",
                )
            })?;

            let mut u_s = DVector::<f64>::zeros(ns);
            for (si, &dof) in sys.restrained_dof_indices.iter().enumerate() {
                u_s[si] = u_full[dof];
            }

            let rhs = &f_f - &k_t_fs * &u_s;
            let u_f_new = cholesky.solve(&rhs);

            let mut diff_norm = 0.0f64;
            let mut total_norm = 0.0f64;
            for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
                let diff = u_f_new[i] - u_full[fi];
                diff_norm += diff * diff;
                total_norm += u_f_new[i] * u_f_new[i];
                u_full[fi] = u_f_new[i];
            }

            let rel_err = diff_norm.sqrt() / total_norm.sqrt().max(1e-9);
            if rel_err < tol {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(CanonicalError::new(
                "NONLINEAR_NOT_CONVERGED",
                "Η ανάλυση 2ης τάξης P-Δ δεν συνέκλινε.",
                "Second-order P-Delta analysis failed to converge.",
            ));
        }

        // Update results
        res.analysis_kind = AnalysisKind::SecondOrderPDelta;
        res.superposition_valid = false;
        for (i, node) in model.nodes.iter().enumerate() {
            res.nodal_displacements[i] = super::types::NodalDisplacementResult {
                node_id: node.id.clone(),
                ux_m: u_full[i * 3],
                uy_m: u_full[i * 3 + 1],
                theta_rad: u_full[i * 3 + 2],
            };
        }

        // Compute 2nd-order reactions: R = K_T * u - F_applied
        let r_full = &last_k_t_global * &u_full - &f_global;
        res.support_reactions.clear();
        for sup in &model.supports {
            let ni = node_idx_map[sup.node_id.as_str()];
            res.support_reactions.push(super::types::SupportReactionResult {
                support_id: sup.id.clone(),
                node_id: sup.node_id.clone(),
                rx_kn: r_full[ni * 3],
                ry_kn: r_full[ni * 3 + 1],
                mz_knm: r_full[ni * 3 + 2],
            });
        }

        pdelta_results.push(res);
    }

    Ok(pdelta_results)
}

/// Solves linear eigenvalue buckling (alpha_cr and buckling modes)
pub fn solve_linear_buckling(
    model: &StructuralModel,
    target_id: &str,
) -> Result<BucklingMode, CanonicalError> {
    let sys = assemble_linear_system(model)?;
    let nf = sys.free_dof_indices.len();

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }
    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    // 1. Solve 1st-order reference state
    let res_list = solve_linear_static(model, &[])?;
    let res = res_list
        .iter()
        .find(|r| r.target_id == target_id)
        .or_else(|| res_list.first())
        .ok_or_else(|| {
            CanonicalError::new(
                "MISSING_MODEL_REFERENCE",
                format!("Δεν βρέθηκε κατάσταση φόρτισης '{}'.", target_id),
                format!("Load case '{}' not found for buckling reference.", target_id),
            )
        })?;

    let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
    for d in &res.nodal_displacements {
        let ni = node_idx_map[d.node_id.as_str()];
        u_full[ni * 3] = d.ux_m;
        u_full[ni * 3 + 1] = d.uy_m;
        u_full[ni * 3 + 2] = d.theta_rad;
    }

    // 2. Assemble reference geometric stiffness K_G(N_ref)
    let mut k_g_global = DMatrix::<f64>::zeros(sys.total_dofs, sys.total_dofs);

    for el in &model.elements {
        if let Element::FrameMember {
            node_i,
            node_j,
            section_id,
            material_id,
            release_i,
            release_j,
            offset_i,
            offset_j,
            ..
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

            let (dxi, _) = match offset_i.frame {
                super::types::OffsetFrame::Local => (offset_i.dx, offset_i.dy),
                super::types::OffsetFrame::Global => (
                    offset_i.dx * cos_a + offset_i.dy * sin_a,
                    -offset_i.dx * sin_a + offset_i.dy * cos_a,
                ),
            };
            let (dxj, _) = match offset_j.frame {
                super::types::OffsetFrame::Local => (offset_j.dx, offset_j.dy),
                super::types::OffsetFrame::Global => (
                    offset_j.dx * cos_a + offset_j.dy * sin_a,
                    -offset_j.dx * sin_a + offset_j.dy * cos_a,
                ),
            };
            let l_eff = (l_nom - dxi + dxj).max(1e-4);

            let sec = sec_map[section_id.as_str()];
            let mat = mat_map[material_id.as_str()];
            let ea = mat.e_gpa * 1e6 * sec.area_a;

            let dofs = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
            let mut u_el = DVector::<f64>::zeros(6);
            for (a, &d) in dofs.iter().enumerate() {
                u_el[a] = u_full[d];
            }
            let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
            let u_loc = &t * &u_el;

            let delta_u_axial = u_loc[3] - u_loc[0];
            let n_axial = (ea / l_eff) * delta_u_axial;

            let k_g_loc = local_geometric_stiffness(n_axial, l_eff);
            let (k_g_cond, _) = condense_releases(&k_g_loc, &[0.0; 6], release_i, release_j);
            let k_g_elem_global = t.transpose() * &k_g_cond * &t;

            for (a, &da) in dofs.iter().enumerate() {
                for (b, &db) in dofs.iter().enumerate() {
                    k_g_global[(da, db)] += k_g_elem_global[(a, b)];
                }
            }
        }
    }

    // Partition K_E,ff and K_G,ff
    let mut k_e_ff = DMatrix::<f64>::zeros(nf, nf);
    let mut a_mat = DMatrix::<f64>::zeros(nf, nf); // A = -K_G,ff

    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_e_ff[(i, j)] = sys.k_global[(fi, fj)];
            a_mat[(i, j)] = -k_g_global[(fi, fj)];
        }
    }

    let k_e_cholesky = k_e_ff.cholesky().ok_or_else(|| {
        CanonicalError::new(
            "SOLVER_SINGULAR_MATRIX",
            "Ο πίνακας K_E δεν είναι θετικά ορισμένος.",
            "Stiffness matrix K_E is not positive definite.",
        )
    })?;

    // Solve generalized eigenproblem K_E^-1 * A * phi = (1 / alpha) * phi via Power Iteration
    let mut phi = DVector::<f64>::from_element(nf, 1.0);
    let mut lambda = 0.0f64;

    for _iter in 0..200 {
        let v = &a_mat * &phi;
        let y = k_e_cholesky.solve(&v);
        let y_norm = y.norm();
        if y_norm < 1e-12 {
            break;
        }
        lambda = y_norm;
        phi = y / y_norm;
    }

    if lambda < 1e-9 {
        return Err(CanonicalError::new(
            "NO_BUCKLING_FOUND",
            "Δεν εντοπίστηκαν θλιπτικές δυνάμεις ικανές να προκαλέσουν λυγισμό.",
            "No compressive forces found capable of causing buckling.",
        ));
    }

    let alpha_cr = 1.0 / lambda;

    // Full global eigenvector
    let mut full_phi = DVector::<f64>::zeros(sys.total_dofs);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        full_phi[fi] = phi[i];
    }

    // Normalize so that max(|dux|, |duy|) = 1.0
    let mut max_trans = 0.0f64;
    for n_idx in 0..model.nodes.len() {
        let ux = full_phi[n_idx * 3].abs();
        let uy = full_phi[n_idx * 3 + 1].abs();
        if ux > max_trans {
            max_trans = ux;
        }
        if uy > max_trans {
            max_trans = uy;
        }
    }
    let norm_factor = if max_trans > 1e-9 {
        1.0 / max_trans
    } else {
        1.0 / full_phi.abs().max().max(1.0)
    };

    let mut node_displacements = Vec::new();
    for (i, node) in model.nodes.iter().enumerate() {
        node_displacements.push(NodeDisplacement {
            node_id: node.id.clone(),
            dux: full_phi[i * 3] * norm_factor,
            duy: full_phi[i * 3 + 1] * norm_factor,
            dtheta: full_phi[i * 3 + 2] * norm_factor,
        });
    }

    Ok(BucklingMode {
        mode_index: 1,
        alpha_cr,
        node_displacements,
    })
}

pub fn solve_linear_buckling_analysis(
    model: &StructuralModel,
    combinations: &[Combination],
) -> Result<Vec<ResultSet>, CanonicalError> {
    let target_id = combinations
        .first()
        .map(|c| c.id.clone())
        .or_else(|| model.load_cases.first().map(|l| l.id.clone()))
        .unwrap_or_else(|| "REF".to_string());
    let mode = solve_linear_buckling(model, &target_id)?;

    Ok(vec![ResultSet {
        revision: model.revision,
        target_id,
        target_kind: "Combination".to_string(),
        analysis_kind: AnalysisKind::LinearBuckling,
        superposition_valid: false,
        nodal_displacements: Vec::new(),
        support_reactions: Vec::new(),
        member_results: Vec::new(),
        cable_results: Vec::new(),
        buckling_modes: vec![mode],
        dynamic_data: None,
    }])
}
