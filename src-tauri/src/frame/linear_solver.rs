use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use super::element_stiffness::{
    condense_releases, element_transformation_matrix, evaluate_point_load, evaluate_thermal_load,
    integrate_distributed_load, local_frame_stiffness,
};
use super::types::{
    AnalysisKind, CanonicalError, Combination, Element, Load, NodalDisplacementResult, ResultSet,
    StructuralModel, SupportKind, SupportReactionResult,
};

pub struct AssembledSystem {
    pub total_dofs: usize,
    pub k_global: DMatrix<f64>,
    pub free_dof_indices: Vec<usize>,
    pub restrained_dof_indices: Vec<usize>,
    pub prescribed_displacements: DVector<f64>,
    pub node_rotations: Vec<Option<f64>>, // None or Some(angle_rad) for rotated nodes
    pub nodal_dof_map: Vec<usize>,
}

pub fn assemble_linear_system(model: &StructuralModel) -> Result<AssembledSystem, CanonicalError> {
    let num_nodes = model.nodes.len();
    let total_dofs = num_nodes * 3;

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }

    let mut k_global = DMatrix::<f64>::zeros(total_dofs, total_dofs);

    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    // 1. Element stiffness assembly
    for el in &model.elements {
        match el {
            Element::FrameMember {
                node_i,
                node_j,
                section_id,
                material_id,
                release_i,
                release_j,
                offset_i,
                offset_j,
                ..
            } => {
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
                let l = (l_nom - dxi + dxj).max(1e-4);

                let sec = sec_map[section_id.as_str()];
                let mat = mat_map[material_id.as_str()];
                let e = mat.e_gpa * 1e6; // kN/m^2

                let k_loc = local_frame_stiffness(e, sec.area_a, sec.inertia_i, l);
                let (k_cond, _) = condense_releases(&k_loc, &[0.0; 6], release_i, release_j);
                let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);

                let k_elem_global = t.transpose() * k_cond * t;

                let dof_indices = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
                for (a, &da) in dof_indices.iter().enumerate() {
                    for (b, &db) in dof_indices.iter().enumerate() {
                        k_global[(da, db)] += k_elem_global[(a, b)];
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

                let sec = sec_map[section_id.as_str()];
                let mat = mat_map[material_id.as_str()];
                let e = mat.e_gpa * 1e6;
                let k_axial = (e * sec.area_a) / l;

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
            Element::Cable { .. } => {
                // Initial linear chord stiffness approximation
            }
        }
    }

    // 2. Identify inclined support nodes and apply rotation transformation to K_global
    let mut node_rotations = vec![None; num_nodes];
    let mut is_restrained = vec![false; total_dofs];
    let mut prescribed = DVector::<f64>::zeros(total_dofs);

    for sup in &model.supports {
        let n_idx = node_idx_map[sup.node_id.as_str()];
        let base = n_idx * 3;
        match sup.kind {
            SupportKind::Fixed => {
                is_restrained[base] = true;
                is_restrained[base + 1] = true;
                is_restrained[base + 2] = true;
            }
            SupportKind::Pinned => {
                is_restrained[base] = true;
                is_restrained[base + 1] = true;
            }
            SupportKind::Roller { angle_deg } => {
                // Roller slides along angle_deg; normal direction angle_deg + 90 deg is restrained
                let theta = angle_deg.to_radians();
                node_rotations[n_idx] = Some(theta);
                is_restrained[base + 1] = true; // Rotated normal DOF is restrained
            }
            SupportKind::Guided { angle_deg } => {
                let theta = angle_deg.to_radians();
                node_rotations[n_idx] = Some(theta);
                is_restrained[base + 1] = true;
                is_restrained[base + 2] = true;
            }
            SupportKind::Elastic { kx, ky, ktheta } => {
                k_global[(base, base)] += kx;
                k_global[(base + 1, base + 1)] += ky;
                k_global[(base + 2, base + 2)] += ktheta;
            }
        }
    }

    // Rotate K_global blocks for inclined nodes
    for (n_idx, &rot) in node_rotations.iter().enumerate() {
        if let Some(theta) = rot {
            let c = theta.cos();
            let s = theta.sin();
            let base = n_idx * 3;

            // Transform rows at node n_idx: [u_t, u_n] = [c s; -s c] * [ux, uy]
            // R = [c s; -s c]
            for col in 0..total_dofs {
                let k0 = k_global[(base, col)];
                let k1 = k_global[(base + 1, col)];
                k_global[(base, col)] = c * k0 + s * k1;
                k_global[(base + 1, col)] = -s * k0 + c * k1;
            }
            // Transform columns at node n_idx
            for row in 0..total_dofs {
                let k0 = k_global[(row, base)];
                let k1 = k_global[(row, base + 1)];
                k_global[(row, base)] = c * k0 + s * k1;
                k_global[(row, base + 1)] = -s * k0 + c * k1;
            }
        }
    }

    let mut free_dofs = Vec::new();
    let mut restrained_dofs = Vec::new();
    for (i, &restr) in is_restrained.iter().enumerate() {
        if restr {
            restrained_dofs.push(i);
        } else {
            free_dofs.push(i);
        }
    }

    Ok(AssembledSystem {
        total_dofs,
        k_global,
        free_dof_indices: free_dofs,
        restrained_dof_indices: restrained_dofs,
        prescribed_displacements: prescribed,
        node_rotations,
        nodal_dof_map: (0..total_dofs).collect(),
    })
}

/// Assembles equivalent global nodal load vector F for a given LoadCase
pub fn assemble_load_vector(
    model: &StructuralModel,
    sys: &AssembledSystem,
    load_case_id: &str,
) -> DVector<f64> {
    let mut f_global = DVector::<f64>::zeros(sys.total_dofs);
    let num_nodes = model.nodes.len();

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }
    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    let lc = match model.load_cases.iter().find(|lc| lc.id == load_case_id) {
        Some(c) => c,
        None => return f_global,
    };

    // 1. Direct nodal loads
    for load in &lc.loads {
        if let Load::NodalLoad {
            node_id,
            fx,
            fy,
            mz,
        } = load
        {
            if let Some(&ni) = node_idx_map.get(node_id.as_str()) {
                f_global[ni * 3] += fx;
                f_global[ni * 3 + 1] += fy;
                f_global[ni * 3 + 2] += mz;
            }
        }
    }

    // 2. Member loads & equivalent nodal loads (F_equiv = sum T^T * f0_condensed)
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
            let l = (l_nom - dxi + dxj).max(1e-4);

            let sec = sec_map[section_id.as_str()];
            let mat = mat_map[material_id.as_str()];
            let e = mat.e_gpa * 1e6;

            let mut f0_loc = [0.0; 6];

            // Self-weight
            if lc.self_weight {
                let gamma = mat.density_gamma;
                let q_sw = integrate_distributed_load(
                    l,
                    0.0,
                    -gamma * sec.area_a,
                    0.0,
                    -gamma * sec.area_a,
                    0.0,
                    1.0,
                    cos_a,
                    sin_a,
                    super::types::LoadFrame::Global,
                );
                for i in 0..6 {
                    f0_loc[i] += q_sw[i];
                }
            }

            // Member interior loads
            for load in &lc.loads {
                match load {
                    Load::MemberPointLoad {
                        element_id,
                        fx,
                        fy,
                        mz,
                        x_ratio,
                        frame,
                    } if element_id == id => {
                        let f0_pt = evaluate_point_load(
                            l, *fx, *fy, *mz, *x_ratio, cos_a, sin_a, *frame,
                        );
                        for i in 0..6 {
                            f0_loc[i] += f0_pt[i];
                        }
                    }
                    Load::MemberDistributedLoad {
                        element_id,
                        qx1,
                        qy1,
                        qx2,
                        qy2,
                        a_ratio,
                        b_ratio,
                        frame,
                    } if element_id == id => {
                        let f0_dist = integrate_distributed_load(
                            l, *qx1, *qy1, *qx2, *qy2, *a_ratio, *b_ratio, cos_a, sin_a, *frame,
                        );
                        for i in 0..6 {
                            f0_loc[i] += f0_dist[i];
                        }
                    }
                    Load::ThermalLoad {
                        element_id,
                        dt_uniform,
                        dt_gradient,
                    } if element_id == id => {
                        let f0_th = evaluate_thermal_load(
                            e,
                            sec.area_a,
                            sec.inertia_i,
                            mat.alpha_t,
                            sec.depth_h,
                            *dt_uniform,
                            *dt_gradient,
                        );
                        for i in 0..6 {
                            f0_loc[i] += f0_th[i];
                        }
                    }
                    _ => {}
                }
            }

            // Condense out releases
            let k_loc = local_frame_stiffness(e, sec.area_a, sec.inertia_i, l);
            let (_, f0_cond) = condense_releases(&k_loc, &f0_loc, release_i, release_j);

            // Transform to global DOFs: F_nodal += T^T * f0_cond
            let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
            let f0_cond_vec = nalgebra::DVector::from_row_slice(&f0_cond);
            let f_eq_global = t.transpose() * f0_cond_vec;

            let dofs = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
            for (idx, &dof) in dofs.iter().enumerate() {
                f_global[dof] += f_eq_global[idx];
            }
        }
    }

    // Rotate load components for inclined nodes
    for (n_idx, &rot) in sys.node_rotations.iter().enumerate() {
        if let Some(theta) = rot {
            let c = theta.cos();
            let s = theta.sin();
            let base = n_idx * 3;
            let fx = f_global[base];
            let fy = f_global[base + 1];
            f_global[base] = c * fx + s * fy;
            f_global[base + 1] = -s * fx + c * fy;
        }
    }

    f_global
}

/// Solves linear static system for all load cases and combinations
pub fn solve_linear_static(
    model: &StructuralModel,
    combinations: &[Combination],
) -> Result<Vec<ResultSet>, CanonicalError> {
    super::validation::validate_model(model)?;
    let diag = super::diagnostics::evaluate_diagnostics(model)?;
    if !diag.is_stable {
        let details = serde_json::to_value(&diag).unwrap_or_default();
        return Err(CanonicalError::new(
            "KINEMATIC_INSTABILITY",
            format!(
                "Ο φορέας είναι κινηματικά ασταθής ({} μηχανισμοί).",
                diag.kinematic_instability_degree
            ),
            format!(
                "The structure is kinematically unstable ({} mechanism modes).",
                diag.kinematic_instability_degree
            ),
        )
        .with_details(serde_json::json!({ "revision": model.revision, "diagnostics": details })));
    }

    let sys = assemble_linear_system(model)?;
    let nf = sys.free_dof_indices.len();
    let ns = sys.restrained_dof_indices.len();

    // Partition K_ff, K_fs, K_sf, K_ss
    let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
    let mut k_fs = DMatrix::<f64>::zeros(nf, ns);
    let mut k_sf = DMatrix::<f64>::zeros(ns, nf);
    let mut k_ss = DMatrix::<f64>::zeros(ns, ns);

    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_ff[(i, j)] = sys.k_global[(fi, fj)];
        }
        for (j, &sj) in sys.restrained_dof_indices.iter().enumerate() {
            k_fs[(i, j)] = sys.k_global[(fi, sj)];
        }
    }
    for (i, &si) in sys.restrained_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_sf[(i, j)] = sys.k_global[(si, fj)];
        }
        for (j, &sj) in sys.restrained_dof_indices.iter().enumerate() {
            k_ss[(i, j)] = sys.k_global[(si, sj)];
        }
    }

    let k_ff_cholesky = k_ff.cholesky().ok_or_else(|| {
        CanonicalError::new(
            "SOLVER_SINGULAR_MATRIX",
            "Ο πίνακας δυσκαμψίας K_ff δεν είναι θετικά ορισμένος.",
            "Stiffness matrix K_ff is not positive definite.",
        )
    })?;

    // 1. Solve each basic LoadCase
    let mut lc_displacements = HashMap::new();
    let mut lc_reactions = HashMap::new();

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }

    for lc in &model.load_cases {
        let f_global = assemble_load_vector(model, &sys, &lc.id);

        let mut f_f = DVector::<f64>::zeros(nf);
        for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
            f_f[i] = f_global[fi];
        }

        let mut u_s = DVector::<f64>::zeros(ns);
        // Add support settlements for this load case
        for load in &lc.loads {
            if let Load::SupportSettlement {
                support_id,
                dx,
                dy,
                dtheta,
            } = load
            {
                if let Some(sup) = model.supports.iter().find(|s| s.id == *support_id) {
                    let ni = node_idx_map[sup.node_id.as_str()];
                    let base = ni * 3;
                    for (si, &dof) in sys.restrained_dof_indices.iter().enumerate() {
                        if dof == base {
                            u_s[si] = *dx;
                        } else if dof == base + 1 {
                            u_s[si] = *dy;
                        } else if dof == base + 2 {
                            u_s[si] = *dtheta;
                        }
                    }
                }
            }
        }

        // K_ff * u_f = F_f - K_fs * u_s
        let rhs = f_f - &k_fs * &u_s;
        let u_f = k_ff_cholesky.solve(&rhs);

        // Reactions: R_s = K_sf * u_f + K_ss * u_s - F_s
        let mut f_s = DVector::<f64>::zeros(ns);
        for (i, &si) in sys.restrained_dof_indices.iter().enumerate() {
            f_s[i] = f_global[si];
        }
        let r_s = &k_sf * &u_f + &k_ss * &u_s - &f_s;

        // Assemble full global displacement vector u_full
        let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
        for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
            u_full[fi] = u_f[i];
        }
        for (i, &si) in sys.restrained_dof_indices.iter().enumerate() {
            u_full[si] = u_s[i];
        }

        // Rotate back displacements for inclined nodes to Cartesian (ux, uy)
        for (n_idx, &rot) in sys.node_rotations.iter().enumerate() {
            if let Some(theta) = rot {
                let c = theta.cos();
                let s = theta.sin();
                let base = n_idx * 3;
                let ut = u_full[base];
                let un = u_full[base + 1];
                u_full[base] = c * ut - s * un;
                u_full[base + 1] = s * ut + c * un;
            }
        }

        // Full reaction vector
        let mut r_full = DVector::<f64>::zeros(sys.total_dofs);
        for (i, &si) in sys.restrained_dof_indices.iter().enumerate() {
            r_full[si] = r_s[i];
        }
        // Rotate reactions back to Cartesian (Rx, Ry)
        for (n_idx, &rot) in sys.node_rotations.iter().enumerate() {
            if let Some(theta) = rot {
                let c = theta.cos();
                let s = theta.sin();
                let base = n_idx * 3;
                let rt = r_full[base];
                let rn = r_full[base + 1];
                r_full[base] = c * rt - s * rn;
                r_full[base + 1] = s * rt + c * rn;
            }
        }

        lc_displacements.insert(lc.id.clone(), u_full);
        lc_reactions.insert(lc.id.clone(), r_full);
    }

    // 2. Generate result sets for combinations (or raw load cases if combinations empty)
    let targets: Vec<(String, String, Vec<(String, f64)>)> = if combinations.is_empty() {
        model
            .load_cases
            .iter()
            .map(|lc| (lc.id.clone(), "LoadCase".to_string(), vec![(lc.id.clone(), 1.0)]))
            .collect()
    } else {
        combinations
            .iter()
            .map(|co| {
                (
                    co.id.clone(),
                    "Combination".to_string(),
                    co.factors.iter().map(|f| (f.load_case_id.clone(), f.factor)).collect(),
                )
            })
            .collect()
    };

    let mut result_sets = Vec::new();
    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    for (target_id, target_kind, factors) in targets {
        // Superpose displacements and reactions
        let mut u_comb = DVector::<f64>::zeros(sys.total_dofs);
        let mut r_comb = DVector::<f64>::zeros(sys.total_dofs);

        for (lc_id, fact) in &factors {
            if let Some(u) = lc_displacements.get(lc_id) {
                u_comb += u * *fact;
            }
            if let Some(r) = lc_reactions.get(lc_id) {
                r_comb += r * *fact;
            }
        }

        let mut nodal_displacements = Vec::new();
        for (i, node) in model.nodes.iter().enumerate() {
            nodal_displacements.push(NodalDisplacementResult {
                node_id: node.id.clone(),
                ux_m: u_comb[i * 3],
                uy_m: u_comb[i * 3 + 1],
                theta_rad: u_comb[i * 3 + 2],
            });
        }

        let mut support_reactions = Vec::new();
        for sup in &model.supports {
            let ni = node_idx_map[sup.node_id.as_str()];
            support_reactions.push(SupportReactionResult {
                support_id: sup.id.clone(),
                node_id: sup.node_id.clone(),
                rx_kn: r_comb[ni * 3],
                ry_kn: r_comb[ni * 3 + 1],
                mz_knm: r_comb[ni * 3 + 2],
            });
        }

        // Member results
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
                let l = (l_nom - dxi + dxj).max(1e-4);

                let sec = sec_map[section_id.as_str()];
                let mat = mat_map[material_id.as_str()];
                let e = mat.e_gpa * 1e6;

                let dofs = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
                let mut u_elem_global = DVector::<f64>::zeros(6);
                for (a, &d) in dofs.iter().enumerate() {
                    u_elem_global[a] = u_comb[d];
                }

                let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
                let u_loc = &t * &u_elem_global;
                let mut u_loc_arr = [0.0; 6];
                for a in 0..6 {
                    u_loc_arr[a] = u_loc[a];
                }

                let k_loc = local_frame_stiffness(e, sec.area_a, sec.inertia_i, l);

                // Collect loads and fixed-end forces for this member scaled by combination factors
                let mut f0_total_loc = [0.0; 6];
                let mut combined_loads = Vec::new();
                let mut has_self_weight = false;

                for (lc_id, fact) in &factors {
                    if let Some(lc) = model.load_cases.iter().find(|c| c.id == *lc_id) {
                        if lc.self_weight {
                            has_self_weight = true;
                            let q_sw = integrate_distributed_load(
                                l,
                                0.0,
                                -mat.density_gamma * sec.area_a * fact,
                                0.0,
                                -mat.density_gamma * sec.area_a * fact,
                                0.0,
                                1.0,
                                cos_a,
                                sin_a,
                                super::types::LoadFrame::Global,
                            );
                            for a in 0..6 {
                                f0_total_loc[a] += q_sw[a];
                            }
                        }

                        for load in &lc.loads {
                            match load {
                                Load::MemberPointLoad {
                                    element_id,
                                    fx,
                                    fy,
                                    mz,
                                    x_ratio,
                                    frame,
                                } if element_id == id => {
                                    let f0_pt = evaluate_point_load(
                                        l,
                                        *fx * fact,
                                        *fy * fact,
                                        *mz * fact,
                                        *x_ratio,
                                        cos_a,
                                        sin_a,
                                        *frame,
                                    );
                                    for a in 0..6 {
                                        f0_total_loc[a] += f0_pt[a];
                                    }
                                    combined_loads.push(load);
                                }
                                Load::MemberDistributedLoad {
                                    element_id,
                                    qx1,
                                    qy1,
                                    qx2,
                                    qy2,
                                    a_ratio,
                                    b_ratio,
                                    frame,
                                } if element_id == id => {
                                    let f0_dist = integrate_distributed_load(
                                        l,
                                        *qx1 * fact,
                                        *qy1 * fact,
                                        *qx2 * fact,
                                        *qy2 * fact,
                                        *a_ratio,
                                        *b_ratio,
                                        cos_a,
                                        sin_a,
                                        *frame,
                                    );
                                    for a in 0..6 {
                                        f0_total_loc[a] += f0_dist[a];
                                    }
                                    combined_loads.push(load);
                                }
                                Load::ThermalLoad {
                                    element_id,
                                    dt_uniform,
                                    dt_gradient,
                                } if element_id == id => {
                                    let f0_th = evaluate_thermal_load(
                                        e,
                                        sec.area_a,
                                        sec.inertia_i,
                                        mat.alpha_t,
                                        sec.depth_h,
                                        *dt_uniform * fact,
                                        *dt_gradient * fact,
                                    );
                                    for a in 0..6 {
                                        f0_total_loc[a] += f0_th[a];
                                    }
                                    combined_loads.push(load);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Condense releases
                let (k_cond, f0_cond) =
                    condense_releases(&k_loc, &f0_total_loc, release_i, release_j);
                let f_end = &k_cond * &u_loc - DVector::from_row_slice(&f0_cond);

                let mut f_end_arr = [0.0; 6];
                for a in 0..6 {
                    f_end_arr[a] = f_end[a];
                }

                let mr = super::diagrams::evaluate_member_diagram(
                    id,
                    l,
                    e,
                    sec.area_a,
                    sec.inertia_i,
                    cos_a,
                    sin_a,
                    &u_loc_arr,
                    &f_end_arr,
                    &combined_loads,
                    has_self_weight,
                    mat.density_gamma,
                );
                member_results.push(mr);
            } else if let Element::TrussBar {
                id,
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } = el
            {
                let ni = node_idx_map[node_i.as_str()];
                let nj = node_idx_map[node_j.as_str()];
                let (xi, yi) = (model.nodes[ni].x, model.nodes[ni].y);
                let (xj, yj) = (model.nodes[nj].x, model.nodes[nj].y);
                let dx = xj - xi;
                let dy = yj - yi;
                let l = dx.hypot(dy).max(1e-6);
                let cos_a = dx / l;
                let sin_a = dy / l;

                let sec = sec_map[section_id.as_str()];
                let mat = mat_map[material_id.as_str()];
                let ea = mat.e_gpa * 1e6 * sec.area_a;

                let uxi = u_comb[ni * 3];
                let uyi = u_comb[ni * 3 + 1];
                let uxj = u_comb[nj * 3];
                let uyj = u_comb[nj * 3 + 1];

                let delta_l = cos_a * (uxj - uxi) + sin_a * (uyj - uyi);
                let n_force = (ea / l) * delta_l;

                let ni_kn = -n_force;
                let nj_kn = n_force;

                let end_forces = super::types::MemberEndForces {
                    ni_kn,
                    qi_kn: 0.0,
                    mi_knm: 0.0,
                    nj_kn,
                    qj_kn: 0.0,
                    mj_knm: 0.0,
                };

                let stations = vec![
                    super::types::StationResult {
                        x_m: 0.0,
                        n_kn: n_force,
                        q_kn: 0.0,
                        m_knm: 0.0,
                        ux_local_m: cos_a * uxi + sin_a * uyi,
                        uy_local_m: -sin_a * uxi + cos_a * uyi,
                        theta_rad: 0.0,
                    },
                    super::types::StationResult {
                        x_m: l,
                        n_kn: n_force,
                        q_kn: 0.0,
                        m_knm: 0.0,
                        ux_local_m: cos_a * uxj + sin_a * uyj,
                        uy_local_m: -sin_a * uxj + cos_a * uyj,
                        theta_rad: 0.0,
                    },
                ];

                let extrema = super::types::MemberExtrema {
                    max_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                    min_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                    max_q: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                    min_q: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                    max_n: super::types::ExtremaPoint { x_m: 0.0, value: n_force },
                    min_n: super::types::ExtremaPoint { x_m: 0.0, value: n_force },
                    max_abs_deflection_m: super::types::ExtremaPoint { x_m: 0.0, value: 0.0 },
                };

                member_results.push(super::types::MemberResult {
                    element_id: id.clone(),
                    length_eff_m: l,
                    end_forces_local: end_forces,
                    stations,
                    extrema,
                });
            }
        }

        result_sets.push(ResultSet {
            revision: model.revision,
            target_id,
            target_kind,
            analysis_kind: AnalysisKind::LinearStatic,
            superposition_valid: true,
            nodal_displacements,
            support_reactions,
            member_results,
            cable_results: Vec::new(),
            buckling_modes: Vec::new(),
            dynamic_data: None,
        });
    }

    Ok(result_sets)
}
