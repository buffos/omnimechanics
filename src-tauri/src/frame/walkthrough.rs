use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use super::element_stiffness::{
    condense_releases, element_transformation_matrix, evaluate_point_load, evaluate_thermal_load,
    integrate_distributed_load, local_frame_stiffness,
};
use super::linear_solver::{assemble_linear_system, assemble_load_vector};
use super::types::{
    CanonicalError, DsmWalkthroughData, Element, ElementWalkthrough, Load, StructuralModel,
};

pub fn generate_dsm_walkthrough(
    model: &StructuralModel,
    target_id: &str,
) -> Result<DsmWalkthroughData, CanonicalError> {
    super::validation::validate_model(model)?;
    let diag = super::diagnostics::evaluate_diagnostics(model)?;
    if !diag.is_stable {
        let details = serde_json::to_value(&diag).unwrap_or_default();
        return Err(CanonicalError::new(
            "KINEMATIC_INSTABILITY",
            "Αδυναμία παραγωγής βημάτων DSM σε κινηματικά ασταθή φορέα.",
            "Cannot generate DSM walkthrough for a kinematically unstable structure.",
        )
        .with_details(serde_json::json!({ "revision": model.revision, "diagnostics": details })));
    }

    let sys = assemble_linear_system(model)?;
    let num_nodes = model.nodes.len();
    let num_dofs = sys.total_dofs;

    let mut node_idx_map = HashMap::new();
    for (i, n) in model.nodes.iter().enumerate() {
        node_idx_map.insert(n.id.as_str(), i);
    }
    let sec_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mat_map: HashMap<&str, &super::types::Material> =
        model.materials.iter().map(|m| (m.id.as_str(), m)).collect();

    let mut element_walkthroughs = Vec::new();

    let lc = model
        .load_cases
        .iter()
        .find(|c| c.id == target_id)
        .ok_or_else(|| {
            CanonicalError::new(
                "MISSING_MODEL_REFERENCE",
                format!("Η κατάσταση φόρτισης '{}' δεν βρέθηκε.", target_id),
                format!("Load case '{}' not found in model.", target_id),
            )
        })?;

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
            let l = dx.hypot(dy).max(1e-6);
            let cos_a = dx / l;
            let sin_a = dy / l;

            let sec = sec_map[section_id.as_str()];
            let mat = mat_map[material_id.as_str()];
            let e = mat.e_gpa * 1e6;

            let k_loc = local_frame_stiffness(e, sec.area_a, sec.inertia_i, l);
            let mut f0_loc = [0.0; 6];

            if lc.self_weight {
                let q_sw = integrate_distributed_load(
                    l,
                    0.0,
                    -mat.density_gamma * sec.area_a,
                    0.0,
                    -mat.density_gamma * sec.area_a,
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

            let (k_cond, f0_cond) = condense_releases(&k_loc, &f0_loc, release_i, release_j);
            let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
            let k_global_elem = t.transpose() * &k_cond * &t;
            let f0_global_elem = t.transpose() * DVector::from_row_slice(&f0_cond);

            let dofs = vec![ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];

            element_walkthroughs.push(ElementWalkthrough {
                element_id: id.clone(),
                dof_indices_global: dofs,
                k_local: dmatrix_to_vec_vec(&k_cond),
                t_matrix: dmatrix_to_vec_vec(&t),
                k_global: dmatrix_to_vec_vec(&k_global_elem),
                f0_local: f0_cond.to_vec(),
                f0_global: f0_global_elem.as_slice().to_vec(),
            });
        }
    }

    // Free vs Restrained partitions
    let nf = sys.free_dof_indices.len();
    let ns = sys.restrained_dof_indices.len();

    let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
    let mut k_fs = DMatrix::<f64>::zeros(nf, ns);

    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
            k_ff[(i, j)] = sys.k_global[(fi, fj)];
        }
        for (j, &sj) in sys.restrained_dof_indices.iter().enumerate() {
            k_fs[(i, j)] = sys.k_global[(fi, sj)];
        }
    }

    let f_global = assemble_load_vector(model, &sys, target_id);
    let mut f_f = DVector::<f64>::zeros(nf);
    for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
        f_f[i] = f_global[fi];
    }

    let mut u_s = DVector::<f64>::zeros(ns);
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

    let k_ff_cholesky = k_ff.clone().cholesky().ok_or_else(|| {
        CanonicalError::new(
            "SOLVER_SINGULAR_MATRIX",
            "Ο πίνακας K_ff είναι ιδιάζων.",
            "Matrix K_ff is singular.",
        )
    })?;
    let rhs = &f_f - &k_fs * &u_s;
    let u_f = k_ff_cholesky.solve(&rhs);

    let full_omitted = num_nodes > 6;
    let k_global_dense = if !full_omitted {
        Some(dmatrix_to_vec_vec(&sys.k_global))
    } else {
        None
    };

    let step_descriptions_el = vec![
        format!("Βήμα 1: Αρίθμηση {} βαθμών ελευθερίας ({} ελεύθεροι, {} δεσμικοί).", num_dofs, nf, ns),
        format!("Βήμα 2: Κατασκευή τοπικών μητρώων δυσκαμψίας k_e και μετασχηματισμών T_e για {} στοιχεία.", element_walkthroughs.len()),
        "Βήμα 3: Σύνθεση γενικού μητρώου δυσκαμψίας K και διαμέριση K_ff.".to_string(),
        "Βήμα 4: Επίλυση συστήματος K_ff * u_f = F_f - K_fs * u_s και υπολογισμός αντιδράσεων.".to_string(),
    ];

    let step_descriptions_en = vec![
        format!("Step 1: Numbering {} degrees of freedom ({} free, {} restrained).", num_dofs, nf, ns),
        format!("Step 2: Constructing local stiffness k_e and transformation T_e for {} elements.", element_walkthroughs.len()),
        "Step 3: Assembling global stiffness matrix K and partitioning K_ff.".to_string(),
        "Step 4: Solving system K_ff * u_f = F_f - K_fs * u_s and recovering support reactions.".to_string(),
    ];

    Ok(DsmWalkthroughData {
        revision: model.revision,
        target_id: target_id.to_string(),
        num_nodes,
        num_dofs,
        free_dofs: sys.free_dof_indices,
        restrained_dofs: sys.restrained_dof_indices,
        elements: element_walkthroughs,
        k_ff: dmatrix_to_vec_vec(&k_ff),
        f_f: f_f.as_slice().to_vec(),
        u_f: u_f.as_slice().to_vec(),
        full_global_matrix_omitted: full_omitted,
        k_global_dense,
        step_descriptions_el,
        step_descriptions_en,
    })
}

fn dmatrix_to_vec_vec(m: &DMatrix<f64>) -> Vec<Vec<f64>> {
    let (rows, cols) = m.shape();
    let mut out = Vec::with_capacity(rows);
    for r in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for c in 0..cols {
            row.push(m[(r, c)]);
        }
        out.push(row);
    }
    out
}
