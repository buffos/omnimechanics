use std::collections::HashSet;

use super::linear_solver::solve_linear_static;
use super::types::{
    AxialBehavior, CanonicalError, Combination, Element, ResultSet, StructuralModel, SupportKind,
};

pub fn has_unilateral_elements(model: &StructuralModel) -> bool {
    for el in &model.elements {
        if let Element::TrussBar { axial_behavior, .. } = el {
            if *axial_behavior != AxialBehavior::Both {
                return true;
            }
        }
    }
    for sup in &model.supports {
        if sup.unilateral {
            return true;
        }
    }
    false
}

pub fn solve_unilateral_active_set(
    model: &StructuralModel,
    combinations: &[Combination],
) -> Result<Vec<ResultSet>, CanonicalError> {
    let mut all_results = Vec::new();

    let targets: Vec<Combination> = if combinations.is_empty() {
        model
            .load_cases
            .iter()
            .map(|lc| Combination {
                id: lc.id.clone(),
                name: lc.name.clone(),
                factors: vec![super::types::LoadCaseFactor {
                    load_case_id: lc.id.clone(),
                    factor: 1.0,
                }],
            })
            .collect()
    } else {
        combinations.to_vec()
    };

    for target_comb in targets {
        let mut active_model = model.clone();
        let max_iterations = 50;
        let mut converged = false;

        let mut inactive_trusses: HashSet<String> = HashSet::new();
        let mut inactive_supports: HashSet<String> = HashSet::new();

        let mut last_result: Option<ResultSet> = None;

        for _iter in 0..max_iterations {
            // Re-build active_model based on current active sets
            active_model.elements = model
                .elements
                .iter()
                .filter(|el| !inactive_trusses.contains(el.id()))
                .cloned()
                .collect();

            active_model.supports = model
                .supports
                .iter()
                .filter(|sup| !inactive_supports.contains(&sup.id))
                .cloned()
                .collect();

            let res_list = solve_linear_static(&active_model, &[target_comb.clone()])?;
            let res = res_list.into_iter().next().ok_or_else(|| {
                CanonicalError::new(
                    "SOLVER_ERROR",
                    "Αποτυχία υπολογισμού ενεργού συνόλου.",
                    "Failed to compute active set solution.",
                )
            })?;

            let mut changed = false;

            // 1. Check truss bars
            for el in &model.elements {
                if let Element::TrussBar {
                    id,
                    node_i,
                    node_j,
                    axial_behavior,
                    section_id,
                    material_id,
                } = el
                {
                    let ni_disp = res.nodal_displacements.iter().find(|d| d.node_id == *node_i);
                    let nj_disp = res.nodal_displacements.iter().find(|d| d.node_id == *node_j);

                    if let (Some(di), Some(dj)) = (ni_disp, nj_disp) {
                        let node_i_geom = model.nodes.iter().find(|n| n.id == *node_i).unwrap();
                        let node_j_geom = model.nodes.iter().find(|n| n.id == *node_j).unwrap();
                        let dx = node_j_geom.x - node_i_geom.x;
                        let dy = node_j_geom.y - node_i_geom.y;
                        let l = dx.hypot(dy).max(1e-6);
                        let c = dx / l;
                        let s = dy / l;

                        let delta_u = (dj.ux_m - di.ux_m) * c + (dj.uy_m - di.uy_m) * s;

                        let sec = model.sections.iter().find(|sec| sec.id == *section_id).unwrap();
                        let mat = model.materials.iter().find(|m| m.id == *material_id).unwrap();
                        let ea = mat.e_gpa * 1e6 * sec.area_a;
                        let axial_force = (ea / l) * delta_u; // > 0 tension, < 0 compression

                        match axial_behavior {
                            AxialBehavior::TensionOnly => {
                                if inactive_trusses.contains(id) {
                                    if delta_u > 1e-7 {
                                        inactive_trusses.remove(id);
                                        changed = true;
                                    }
                                } else if axial_force < -1e-6 {
                                    inactive_trusses.insert(id.clone());
                                    changed = true;
                                }
                            }
                            AxialBehavior::CompressionOnly => {
                                if inactive_trusses.contains(id) {
                                    if delta_u < -1e-7 {
                                        inactive_trusses.remove(id);
                                        changed = true;
                                    }
                                } else if axial_force > 1e-6 {
                                    inactive_trusses.insert(id.clone());
                                    changed = true;
                                }
                            }
                            AxialBehavior::Both => {}
                        }
                    }
                }
            }

            // 2. Check unilateral supports (uplift)
            for sup in &model.supports {
                if sup.unilateral {
                    let r_sup = res.support_reactions.iter().find(|r| r.support_id == sup.id);
                    let d_node = res.nodal_displacements.iter().find(|d| d.node_id == sup.node_id);

                    match &sup.kind {
                        SupportKind::Roller { angle_deg } => {
                            let theta = angle_deg.to_radians();
                            let n_vec = (-theta.sin(), theta.cos()); // normal direction

                            if inactive_supports.contains(&sup.id) {
                                // Currently lifted off: check if node has penetrated ground
                                if let Some(d) = d_node {
                                    let un = d.ux_m * n_vec.0 + d.uy_m * n_vec.1;
                                    if un < -1e-7 {
                                        inactive_supports.remove(&sup.id);
                                        changed = true;
                                    }
                                }
                            } else if let Some(r) = r_sup {
                                let rn = r.rx_kn * n_vec.0 + r.ry_kn * n_vec.1;
                                if rn < -1e-6 {
                                    // Negative normal reaction => support is in tension (pulling down on structure)
                                    inactive_supports.insert(sup.id.clone());
                                    changed = true;
                                }
                            }
                        }
                        SupportKind::Pinned | SupportKind::Fixed => {
                            if inactive_supports.contains(&sup.id) {
                                if let Some(d) = d_node {
                                    if d.uy_m < -1e-7 {
                                        inactive_supports.remove(&sup.id);
                                        changed = true;
                                    }
                                }
                            } else if let Some(r) = r_sup {
                                if r.ry_kn < -1e-6 {
                                    inactive_supports.insert(sup.id.clone());
                                    changed = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            last_result = Some(res);
            if !changed {
                converged = true;
                break;
            }
        }

        let mut final_res = last_result.ok_or_else(|| {
            CanonicalError::new(
                "NONLINEAR_NOT_CONVERGED",
                "Ο επιλυτής ενεργού συνόλου δεν συνέκλινε.",
                "Active-set solver failed to converge.",
            )
        })?;

        if !converged {
            return Err(CanonicalError::new(
                "NONLINEAR_NOT_CONVERGED",
                "Υπέρβαση μέγιστων επαναλήψεων ενεργού συνόλου.",
                "Active-set solver exceeded maximum iterations.",
            ));
        }

        // Populate inactive elements with zero internal forces
        for inact_id in &inactive_trusses {
            if !final_res.member_results.iter().any(|m| m.element_id == *inact_id) {
                if let Some(Element::TrussBar { node_i, node_j, .. }) =
                    model.elements.iter().find(|e| e.id() == inact_id)
                {
                    let node_i_geom = model.nodes.iter().find(|n| n.id == *node_i).unwrap();
                    let node_j_geom = model.nodes.iter().find(|n| n.id == *node_j).unwrap();
                    let l = (node_j_geom.x - node_i_geom.x)
                        .hypot(node_j_geom.y - node_i_geom.y)
                        .max(1e-6);

                    final_res.member_results.push(super::types::MemberResult {
                        element_id: inact_id.clone(),
                        length_eff_m: l,
                        end_forces_local: super::types::MemberEndForces {
                            ni_kn: 0.0,
                            qi_kn: 0.0,
                            mi_knm: 0.0,
                            nj_kn: 0.0,
                            qj_kn: 0.0,
                            mj_knm: 0.0,
                        },
                        stations: vec![
                            super::types::StationResult {
                                x_m: 0.0,
                                n_kn: 0.0,
                                q_kn: 0.0,
                                m_knm: 0.0,
                                ux_local_m: 0.0,
                                uy_local_m: 0.0,
                                theta_rad: 0.0,
                            },
                            super::types::StationResult {
                                x_m: l,
                                n_kn: 0.0,
                                q_kn: 0.0,
                                m_knm: 0.0,
                                ux_local_m: 0.0,
                                uy_local_m: 0.0,
                                theta_rad: 0.0,
                            },
                        ],
                        extrema: super::types::MemberExtrema {
                            max_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            min_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            max_q: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            min_q: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            max_n: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            min_n: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                            max_abs_deflection_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                        },
                    });
                }
            }
        }

        // Populate inactive supports with zero reactions
        for inact_sup in &inactive_supports {
            if !final_res.support_reactions.iter().any(|r| r.support_id == *inact_sup) {
                if let Some(sup) = model.supports.iter().find(|s| s.id == *inact_sup) {
                    final_res.support_reactions.push(super::types::SupportReactionResult {
                        support_id: sup.id.clone(),
                        node_id: sup.node_id.clone(),
                        rx_kn: 0.0,
                        ry_kn: 0.0,
                        mz_knm: 0.0,
                    });
                }
            }
        }

        final_res.superposition_valid = false;
        all_results.push(final_res);
    }

    Ok(all_results)
}
