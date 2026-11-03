use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use super::types::{
    CanonicalError, DiagnosticMessage, Diagnostics, Element, MechanismClassification,
    MechanismMode, NodeDisplacement, StructuralModel, SupportKind,
};

pub fn evaluate_diagnostics(model: &StructuralModel) -> Result<Diagnostics, CanonicalError> {
    super::validation::validate_model(model)?;

    let num_nodes = model.nodes.len();
    if num_nodes == 0 {
        return Ok(Diagnostics {
            revision: model.revision,
            static_indeterminacy_degree: 0,
            kinematic_instability_degree: 0,
            free_dofs: 0,
            is_stable: true,
            mechanism_modes: Vec::new(),
            warnings: Vec::new(),
        });
    }

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }

    let total_dofs = num_nodes * 3;
    let mut restrained = vec![false; total_dofs];
    let mut reaction_count = 0;

    for sup in &model.supports {
        if let Some(&n_idx) = node_idx_map.get(sup.node_id.as_str()) {
            let base_dof = n_idx * 3;
            match &sup.kind {
                SupportKind::Fixed => {
                    restrained[base_dof] = true;
                    restrained[base_dof + 1] = true;
                    restrained[base_dof + 2] = true;
                    reaction_count += 3;
                }
                SupportKind::Pinned => {
                    restrained[base_dof] = true;
                    restrained[base_dof + 1] = true;
                    reaction_count += 2;
                }
                SupportKind::Roller { .. } => {
                    // Restrains 1 direction normal to roller
                    restrained[base_dof + 1] = true;
                    reaction_count += 1;
                }
                SupportKind::Guided { .. } => {
                    // Restrains 1 normal direction + rotation
                    restrained[base_dof + 1] = true;
                    restrained[base_dof + 2] = true;
                    reaction_count += 2;
                }
                SupportKind::Elastic { kx, ky, ktheta } => {
                    if *kx > 1e9 {
                        restrained[base_dof] = true;
                        reaction_count += 1;
                    }
                    if *ky > 1e9 {
                        restrained[base_dof + 1] = true;
                        reaction_count += 1;
                    }
                    if *ktheta > 1e9 {
                        restrained[base_dof + 2] = true;
                        reaction_count += 1;
                    }
                }
            }
        }
    }

    // 1. Calculate static indeterminacy degree: n_s = R + B - 3*N - H
    let mut internal_unknowns = 0;
    let mut hinge_releases = 0;
    for el in &model.elements {
        match el {
            Element::FrameMember {
                release_i,
                release_j,
                ..
            } => {
                internal_unknowns += 3;
                if release_i.axial {
                    hinge_releases += 1;
                }
                if release_i.shear {
                    hinge_releases += 1;
                }
                if release_i.moment {
                    hinge_releases += 1;
                }
                if release_j.axial {
                    hinge_releases += 1;
                }
                if release_j.shear {
                    hinge_releases += 1;
                }
                if release_j.moment {
                    hinge_releases += 1;
                }
            }
            Element::TrussBar { .. } => {
                internal_unknowns += 1;
            }
            Element::Spring { node_j, .. } => {
                if node_j.is_some() {
                    internal_unknowns += 1;
                }
            }
            Element::Cable { .. } => {
                internal_unknowns += 1;
            }
        }
    }

    let static_indeterminacy =
        reaction_count as i32 + internal_unknowns as i32 - (3 * num_nodes as i32) - hinge_releases as i32;

    // 2. Assemble topological stiffness matrix to detect kinematic mechanisms
    let mut k_global = DMatrix::<f64>::zeros(total_dofs, total_dofs);

    let sec_map: HashMap<&str, (f64, f64)> = model
        .sections
        .iter()
        .map(|s| (s.id.as_str(), (s.area_a, s.inertia_i)))
        .collect();
    let mat_map: HashMap<&str, f64> = model
        .materials
        .iter()
        .map(|m| (m.id.as_str(), m.e_gpa * 1e6)) // kN/m^2
        .collect();

    for el in &model.elements {
        match el {
            Element::FrameMember {
                node_i,
                node_j,
                section_id,
                material_id,
                release_i,
                release_j,
                ..
            } => {
                let ni = node_idx_map[node_i.as_str()];
                let nj = node_idx_map[node_j.as_str()];
                let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
                let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
                let dx = xj - xi;
                let dy = yj - yi;
                let l = dx.hypot(dy).max(1e-6);
                let c = dx / l;
                let s = dy / l;

                let (area, inertia) = sec_map.get(section_id.as_str()).copied().unwrap_or((0.01, 1e-4));
                let e = mat_map.get(material_id.as_str()).copied().unwrap_or(2.1e8);

                let ea_l = (e * area) / l;
                let ei_l3 = (e * inertia) / (l * l * l);

                // Local 6x6 Euler-Bernoulli beam stiffness
                let mut k_loc = DMatrix::<f64>::zeros(6, 6);
                k_loc[(0, 0)] = ea_l;
                k_loc[(0, 3)] = -ea_l;
                k_loc[(3, 0)] = -ea_l;
                k_loc[(3, 3)] = ea_l;

                k_loc[(1, 1)] = 12.0 * ei_l3;
                k_loc[(1, 2)] = 6.0 * ei_l3 * l;
                k_loc[(1, 4)] = -12.0 * ei_l3;
                k_loc[(1, 5)] = 6.0 * ei_l3 * l;

                k_loc[(2, 1)] = 6.0 * ei_l3 * l;
                k_loc[(2, 2)] = 4.0 * ei_l3 * l * l;
                k_loc[(2, 4)] = -6.0 * ei_l3 * l;
                k_loc[(2, 5)] = 2.0 * ei_l3 * l * l;

                k_loc[(4, 1)] = -12.0 * ei_l3;
                k_loc[(4, 2)] = -6.0 * ei_l3 * l;
                k_loc[(4, 4)] = 12.0 * ei_l3;
                k_loc[(4, 5)] = -6.0 * ei_l3 * l;

                k_loc[(5, 1)] = 6.0 * ei_l3 * l;
                k_loc[(5, 2)] = 2.0 * ei_l3 * l * l;
                k_loc[(5, 4)] = -6.0 * ei_l3 * l;
                k_loc[(5, 5)] = 4.0 * ei_l3 * l * l;

                // Static condensation for releases
                let mut released_dofs = Vec::new();
                if release_i.axial {
                    released_dofs.push(0);
                }
                if release_i.shear {
                    released_dofs.push(1);
                }
                if release_i.moment {
                    released_dofs.push(2);
                }
                if release_j.axial {
                    released_dofs.push(3);
                }
                if release_j.shear {
                    released_dofs.push(4);
                }
                if release_j.moment {
                    released_dofs.push(5);
                }

                if !released_dofs.is_empty() {
                    let mut active_dofs = Vec::new();
                    for idx in 0..6 {
                        if !released_dofs.contains(&idx) {
                            active_dofs.push(idx);
                        }
                    }
                    if !active_dofs.is_empty() {
                        let na = active_dofs.len();
                        let nr = released_dofs.len();
                        let mut k_aa = DMatrix::<f64>::zeros(na, na);
                        let mut k_ar = DMatrix::<f64>::zeros(na, nr);
                        let mut k_ra = DMatrix::<f64>::zeros(nr, na);
                        let mut k_rr = DMatrix::<f64>::zeros(nr, nr);

                        for i in 0..na {
                            for j in 0..na {
                                k_aa[(i, j)] = k_loc[(active_dofs[i], active_dofs[j])];
                            }
                            for j in 0..nr {
                                k_ar[(i, j)] = k_loc[(active_dofs[i], released_dofs[j])];
                            }
                        }
                        for i in 0..nr {
                            for j in 0..na {
                                k_ra[(i, j)] = k_loc[(released_dofs[i], active_dofs[j])];
                            }
                            for j in 0..nr {
                                k_rr[(i, j)] = k_loc[(released_dofs[i], released_dofs[j])];
                            }
                        }

                        if let Some(k_rr_inv) = k_rr.try_inverse() {
                            let k_cond = k_aa - &k_ar * k_rr_inv * k_ra;
                            k_loc.fill(0.0);
                            for i in 0..na {
                                for j in 0..na {
                                    k_loc[(active_dofs[i], active_dofs[j])] = k_cond[(i, j)];
                                }
                            }
                        }
                    } else {
                        k_loc.fill(0.0);
                    }
                }

                // Global transformation
                let mut t = DMatrix::<f64>::zeros(6, 6);
                t[(0, 0)] = c;
                t[(0, 1)] = s;
                t[(1, 0)] = -s;
                t[(1, 1)] = c;
                t[(2, 2)] = 1.0;
                t[(3, 3)] = c;
                t[(3, 4)] = s;
                t[(4, 3)] = -s;
                t[(4, 4)] = c;
                t[(5, 5)] = 1.0;

                let k_g = t.transpose() * k_loc * t;

                let dof_map = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
                for (a, &da) in dof_map.iter().enumerate() {
                    for (b, &db) in dof_map.iter().enumerate() {
                        k_global[(da, db)] += k_g[(a, b)];
                    }
                }
            }
            Element::TrussBar {
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } => {
                let ni = node_idx_map[node_i.as_str()];
                let nj = node_idx_map[node_j.as_str()];
                let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
                let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
                let dx = xj - xi;
                let dy = yj - yi;
                let l = dx.hypot(dy).max(1e-6);
                let c = dx / l;
                let s = dy / l;

                let (area, _) = sec_map.get(section_id.as_str()).copied().unwrap_or((0.005, 1e-4));
                let e = mat_map.get(material_id.as_str()).copied().unwrap_or(2.1e8);
                let k_axial = (e * area) / l;

                let k_mat = [
                    [c * c, c * s, -c * c, -c * s],
                    [c * s, s * s, -c * s, -s * s],
                    [-c * c, -c * s, c * c, c * s],
                    [-c * s, -s * s, c * s, s * s],
                ];
                let dof_map = [ni * 3, ni * 3 + 1, nj * 3, nj * 3 + 1];
                for a in 0..4 {
                    for b in 0..4 {
                        k_global[(dof_map[a], dof_map[b])] += k_axial * k_mat[a][b];
                    }
                }
            }
            Element::Spring {
                node_i,
                node_j,
                kx,
                ky,
                ktheta,
                angle_deg,
                ..
            } => {
                let ni = node_idx_map[node_i.as_str()];
                let rad = angle_deg.to_radians();
                let c = rad.cos();
                let s = rad.sin();

                // Rotate spring stiffness
                let kxx = kx * c * c + ky * s * s;
                let kyy = kx * s * s + ky * c * c;
                let kxy = (kx - ky) * c * s;

                k_global[(ni * 3, ni * 3)] += kxx;
                k_global[(ni * 3 + 1, ni * 3 + 1)] += kyy;
                k_global[(ni * 3, ni * 3 + 1)] += kxy;
                k_global[(ni * 3 + 1, ni * 3)] += kxy;
                k_global[(ni * 3 + 2, ni * 3 + 2)] += ktheta;

                if let Some(nj_str) = node_j {
                    if let Some(&nj) = node_idx_map.get(nj_str.as_str()) {
                        k_global[(nj * 3, nj * 3)] += kxx;
                        k_global[(nj * 3 + 1, nj * 3 + 1)] += kyy;
                        k_global[(nj * 3, nj * 3 + 1)] += kxy;
                        k_global[(nj * 3 + 1, nj * 3)] += kxy;
                        k_global[(nj * 3 + 2, nj * 3 + 2)] += ktheta;

                        k_global[(ni * 3, nj * 3)] -= kxx;
                        k_global[(ni * 3 + 1, nj * 3 + 1)] -= kyy;
                        k_global[(ni * 3, nj * 3 + 1)] -= kxy;
                        k_global[(ni * 3 + 1, nj * 3)] -= kxy;
                        k_global[(ni * 3 + 2, nj * 3 + 2)] -= ktheta;

                        k_global[(nj * 3, ni * 3)] -= kxx;
                        k_global[(nj * 3 + 1, ni * 3 + 1)] -= kyy;
                        k_global[(nj * 3, ni * 3 + 1)] -= kxy;
                        k_global[(nj * 3 + 1, ni * 3)] -= kxy;
                        k_global[(nj * 3 + 2, ni * 3 + 2)] -= ktheta;
                    }
                }
            }
            Element::Cable {
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } => {
                let ni = node_idx_map[node_i.as_str()];
                let nj = node_idx_map[node_j.as_str()];
                let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
                let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
                let dx = xj - xi;
                let dy = yj - yi;
                let l = dx.hypot(dy).max(1e-6);
                let c = dx / l;
                let s = dy / l;

                let (area, _) = sec_map.get(section_id.as_str()).copied().unwrap_or((0.001, 1e-6));
                let e = mat_map.get(material_id.as_str()).copied().unwrap_or(1.6e8);
                let k_cable = (e * area) / l;

                let k_mat = [
                    [c * c, c * s, -c * c, -c * s],
                    [c * s, s * s, -c * s, -s * s],
                    [-c * c, -c * s, c * c, c * s],
                    [-c * s, -s * s, c * s, s * s],
                ];
                let dof_map = [ni * 3, ni * 3 + 1, nj * 3, nj * 3 + 1];
                for a in 0..4 {
                    for b in 0..4 {
                        k_global[(dof_map[a], dof_map[b])] += k_cable * k_mat[a][b];
                    }
                }
            }
        }
    }

    // Apply support boundary conditions: Extract free DOFs
    let mut free_dof_indices = Vec::new();
    for (i, &is_restr) in restrained.iter().enumerate() {
        if !is_restr {
            free_dof_indices.push(i);
        }
    }

    let nf = free_dof_indices.len();
    if nf == 0 {
        return Ok(Diagnostics {
            revision: model.revision,
            static_indeterminacy_degree: static_indeterminacy,
            kinematic_instability_degree: 0,
            free_dofs: 0,
            is_stable: true,
            mechanism_modes: Vec::new(),
            warnings: Vec::new(),
        });
    }

    let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
    for (i, &fi) in free_dof_indices.iter().enumerate() {
        for (j, &fj) in free_dof_indices.iter().enumerate() {
            k_ff[(i, j)] = k_global[(fi, fj)];
        }
    }

    // Perform SVD to reveal rank and nullspace
    let svd = k_ff.svd(true, true);
    let s_max = svd.singular_values[0];
    let sv_tol = 1e-7 * s_max.max(1.0);

    let mut mechanism_modes = Vec::new();
    let v_t = svd.v_t.unwrap();

    for i in 0..nf {
        let sv = svd.singular_values[i];
        if sv < sv_tol {
            let row = v_t.row(i);
            let mut full_vec = DVector::<f64>::zeros(total_dofs);
            for (local_idx, &global_dof) in free_dof_indices.iter().enumerate() {
                full_vec[global_dof] = row[local_idx];
            }

            // Normalize so that max(|ux|, |uy|) = 1.0 (or max absolute = 1.0)
            let mut max_trans = 0.0f64;
            for n_idx in 0..num_nodes {
                let ux = full_vec[n_idx * 3].abs();
                let uy = full_vec[n_idx * 3 + 1].abs();
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
                let max_rot = full_vec.abs().max();
                if max_rot > 1e-9 {
                    1.0 / max_rot
                } else {
                    1.0
                }
            };

            let mut node_displacements = Vec::new();
            for n_idx in 0..num_nodes {
                node_displacements.push(NodeDisplacement {
                    node_id: model.nodes[n_idx].id.clone(),
                    dux: full_vec[n_idx * 3] * norm_factor,
                    duy: full_vec[n_idx * 3 + 1] * norm_factor,
                    dtheta: full_vec[n_idx * 3 + 2] * norm_factor,
                });
            }

            // Classify mechanism mode
            let classification = if max_trans > 1e-3 {
                MechanismClassification::GlobalSwayOrHingeChain
            } else {
                MechanismClassification::LocalHingeMechanism
            };

            mechanism_modes.push(MechanismMode {
                mode_index: mechanism_modes.len() + 1,
                classification,
                node_displacements,
            });
        }
    }

    let nk = mechanism_modes.len() as u32;
    let is_stable = nk == 0;

    let mut warnings = Vec::new();
    if !is_stable {
        warnings.push(DiagnosticMessage {
            code: "KINEMATIC_INSTABILITY".to_string(),
            element_id: None,
            node_id: None,
            message_el: format!(
                "Ο φορέας είναι κινηματικά ασταθής (ανιχνεύθηκαν {} μηχανισμοί).",
                nk
            ),
            message_en: format!(
                "The structure is kinematically unstable ({} mechanism modes detected).",
                nk
            ),
        });
    }

    Ok(Diagnostics {
        revision: model.revision,
        static_indeterminacy_degree: static_indeterminacy,
        kinematic_instability_degree: nk,
        free_dofs: nf,
        is_stable,
        mechanism_modes,
        warnings,
    })
}
