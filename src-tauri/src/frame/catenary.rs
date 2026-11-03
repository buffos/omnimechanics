use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use super::element_stiffness::{
    condense_releases, element_transformation_matrix, local_frame_stiffness,
};
use super::linear_solver::{
    assemble_linear_system, assemble_load_vector, AssembledSystem,
};
use super::pdelta_buckling::local_geometric_stiffness;
use super::types::*;

/// Irvine Elastic Catenary Element State
#[derive(Debug, Clone)]
pub struct CatenaryState {
    pub element_id: String,
    pub l0: f64,
    pub h: f64,
    pub v: f64,
    pub vi_reaction: f64,
    pub vj_reaction: f64,
    pub max_tension: f64,
    pub mid_sag: f64,
    pub is_slack: bool,
    pub curve_points: Vec<Point2D>,
    pub k_tangent_global: DMatrix<f64>, // 4x4 for [uxi, uyi, uxj, uyj]
    pub f_internal_global: DVector<f64>, // 4x1 for [fxi, fyi, fxj, fyj]
}

/// Check if model contains any Cable elements
pub fn has_cable_elements(model: &StructuralModel) -> bool {
    model.elements.iter().any(|el| matches!(el, Element::Cable { .. }))
}

/// Convert input CablePrestress (UnstressedLength, SagRatio, or Pretension) to L0
pub fn compute_unstressed_length(
    prestress: &CablePrestress,
    chord_dx: f64,
    chord_dy: f64,
    ea: f64,
    w0: f64,
) -> f64 {
    let lc = chord_dx.hypot(chord_dy).max(1e-4);

    match prestress {
        CablePrestress::UnstressedLength { l0 } => *l0,
        CablePrestress::Pretension { t0 } => {
            // Under pretension T0, Lc ≈ L0 * (1 + T0 / EA) => L0 = Lc / (1 + T0 / EA)
            let strain = (*t0 / ea).clamp(0.0, 0.5);
            lc / (1.0 + strain)
        }
        CablePrestress::SagRatio { sag_to_span } => {
            // Parabolic Irvine approximation:
            // sag f = sag_to_span * Lc
            // H ≈ w0 * Lc^2 / (8 * f) = w0 * Lc / (8 * sag_to_span)
            let r = sag_to_span.clamp(1e-4, 0.5);
            let h = if w0 > 1e-6 {
                (w0 * lc) / (8.0 * r)
            } else {
                0.01 * ea
            };
            let geometric_length = lc * (1.0 + (8.0 / 3.0) * r * r);
            let strain = (h / ea).clamp(0.0, 0.5);
            geometric_length / (1.0 + strain)
        }
    }
}

/// Computes relative span (dx, dy) given (H, V, L0, EA, w0)
pub fn irvine_spans(h: f64, v: f64, l0: f64, ea: f64, w0: f64) -> (f64, f64) {
    let w = w0.max(1e-6);
    let u1 = v / h;
    let u2 = (v - w * l0) / h;

    let asinh_u1 = u1.asinh();
    let asinh_u2 = u2.asinh();

    let dx = (h * l0 / ea) + (h / w) * (asinh_u1 - asinh_u2);

    let r1 = (h * h + v * v).sqrt();
    let v2 = v - w * l0;
    let r2 = (h * h + v2 * v2).sqrt();

    let dy = (l0 / ea) * (v - 0.5 * w * l0) + (1.0 / w) * (r1 - r2);

    (dx, dy)
}

/// Solves inner 2x2 Irvine catenary for (H, V) matching target span (dx_target, dy_target)
pub fn solve_irvine_catenary(
    dx_target: f64,
    dy_target: f64,
    l0: f64,
    ea: f64,
    w0: f64,
) -> Result<(f64, f64, DMatrix<f64>, bool), CanonicalError> {
    let w = w0.max(1e-6);
    let lc = dx_target.hypot(dy_target).max(1e-4);

    // Initial guess
    let mut h = (ea * ((lc - l0) / l0).max(1e-4)).clamp(1e-3, 0.5 * ea);
    let mut v = h * (dy_target / dx_target.max(1e-4)) + 0.5 * w * l0;

    let max_iter = 60;
    let tol = 1e-10;
    let mut converged = false;
    let mut is_slack = false;

    let mut last_kt = DMatrix::<f64>::zeros(2, 2);

    for _iter in 0..max_iter {
        h = h.max(1e-5);
        let u1 = v / h;
        let v2 = v - w * l0;
        let u2 = v2 / h;

        let r1 = (h * h + v * v).sqrt().max(1e-6);
        let r2 = (h * h + v2 * v2).sqrt().max(1e-6);

        let asinh_diff = u1.asinh() - u2.asinh();

        let dx_calc = (h * l0 / ea) + (h / w) * asinh_diff;
        let dy_calc = (l0 / ea) * (v - 0.5 * w * l0) + (1.0 / w) * (r1 - r2);

        let res_x = dx_calc - dx_target;
        let res_y = dy_calc - dy_target;

        if res_x.hypot(res_y) < tol {
            converged = true;
            break;
        }

        // Flexibility Jacobian F = [ d_dx/dH, d_dx/dV; d_dy/dH, d_dy/dV ]
        let f11 = (l0 / ea) + (1.0 / w) * asinh_diff - (1.0 / w) * (v / r1 - v2 / r2);
        let f12 = (1.0 / w) * (h / r1 - h / r2);
        let f21 = f12;
        let f22 = (l0 / ea) + (1.0 / w) * (v / r1 - v2 / r2);

        let det = f11 * f22 - f12 * f21;
        if det.abs() < 1e-14 {
            // Degenerate/slack state
            is_slack = true;
            h = 1e-4 * ea;
            v = 0.5 * w * l0;
            break;
        }

        // Invert flexibility to get 2x2 stiffness
        last_kt[(0, 0)] = f22 / det;
        last_kt[(0, 1)] = -f12 / det;
        last_kt[(1, 0)] = -f21 / det;
        last_kt[(1, 1)] = f11 / det;

        let dh = last_kt[(0, 0)] * (-res_x) + last_kt[(0, 1)] * (-res_y);
        let dv = last_kt[(1, 0)] * (-res_x) + last_kt[(1, 1)] * (-res_y);

        // Damped Newton step
        let step = 0.8;
        h += step * dh;
        v += step * dv;

        if h < 1e-5 * ea {
            is_slack = true;
        }
    }

    // Check slack condition
    if !converged || h < 1e-4 * ea {
        is_slack = true;
    }

    // Regularize tangent stiffness and force if slack
    if is_slack || last_kt[(0, 0)] <= 0.0 || last_kt[(1, 1)] <= 0.0 {
        is_slack = true;
        h = 0.0;
        v = 0.5 * w * l0;
        let k_reg = 1e-6 * (ea / l0);
        last_kt[(0, 0)] = k_reg;
        last_kt[(1, 1)] = k_reg;
        last_kt[(0, 1)] = 0.0;
        last_kt[(1, 0)] = 0.0;
    }

    Ok((h, v, last_kt, is_slack))
}

/// Evaluates full catenary curve points and mid-span sag
pub fn evaluate_catenary_geometry(
    x_i: f64,
    y_i: f64,
    x_j: f64,
    y_j: f64,
    h: f64,
    v: f64,
    l0: f64,
    ea: f64,
    w0: f64,
) -> (Vec<Point2D>, f64, f64) {
    let w = w0.max(1e-6);
    let dx = x_j - x_i;
    let dy = y_j - y_i;
    let sx = if dx.abs() > 1e-9 { dx.signum() } else { 1.0 };

    let n_pts = 21;
    let mut curve_points = Vec::with_capacity(n_pts);
    let mut mid_sag = 0.0;

    if h <= 1e-6 {
        // Slack cable drape
        let lc = dx.hypot(dy).max(1e-4);
        let excess = (l0 - lc).max(0.0);
        mid_sag = (0.375 * l0 * excess).sqrt();
        let max_tension = 0.5 * w * l0;
        for k in 0..n_pts {
            let xi = k as f64 / (n_pts - 1) as f64;
            let pt_x = x_i + xi * dx;
            let pt_y = y_i + xi * dy - 4.0 * mid_sag * xi * (1.0 - xi);
            curve_points.push(Point2D { x: pt_x, y: pt_y });
        }
        return (curve_points, mid_sag, max_tension);
    }

    let asinh_u1 = (v / h).asinh();

    for k in 0..n_pts {
        let s = (k as f64 / (n_pts - 1) as f64) * l0;
        let u_s = (v - w * s) / h;
        let asinh_us = u_s.asinh();

        let xs = (h * s / ea) + (h / w) * (asinh_u1 - asinh_us);
        let r1 = (h * h + v * v).sqrt();
        let rs = (h * h + (v - w * s).powi(2)).sqrt();
        let ys = (s / ea) * (v - 0.5 * w * s) + (1.0 / w) * (r1 - rs);

        let pt_x = x_i + sx * xs;
        let pt_y = y_i + ys;
        curve_points.push(Point2D { x: pt_x, y: pt_y });

        if k == (n_pts - 1) / 2 {
            let chord_y_at_mid = y_i + (pt_x - x_i) * (dy / dx.abs().max(1e-6));
            mid_sag = (chord_y_at_mid - pt_y).abs();
        }
    }

    let max_tension = (h * h + v * v).sqrt().max((h * h + (v - w * l0).powi(2)).sqrt());
    (curve_points, mid_sag, max_tension)
}

/// Solves hybrid structure with Irvine Elastic Catenary cables via outer Newton-Raphson
pub fn solve_catenary_cable(
    model: &StructuralModel,
    combinations: &[Combination],
) -> Result<Vec<ResultSet>, CanonicalError> {
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

    let targets: Vec<Combination> = if combinations.is_empty() {
        model
            .load_cases
            .iter()
            .map(|lc| Combination {
                id: lc.id.clone(),
                name: lc.name.clone(),
                factors: vec![LoadCaseFactor {
                    load_case_id: lc.id.clone(),
                    factor: 1.0,
                }],
            })
            .collect()
    } else {
        combinations.to_vec()
    };

    let mut result_sets = Vec::new();

    for target_comb in targets {
        // Linear load vector for target
        let mut f_target = DVector::<f64>::zeros(sys.total_dofs);
        for f_entry in &target_comb.factors {
            let flc = assemble_load_vector(model, &sys, &f_entry.load_case_id);
            f_target += f_entry.factor * flc;
        }

        let mut u_full = DVector::<f64>::zeros(sys.total_dofs);
        let max_outer_iter = 80;
        let mut outer_converged = false;
        let mut cable_states: Vec<CatenaryState> = Vec::new();
        let mut slack_warning = false;
        let mut last_k_global = DMatrix::<f64>::zeros(sys.total_dofs, sys.total_dofs);

        for _iter in 0..max_outer_iter {
            cable_states.clear();
            let mut k_total = sys.k_global.clone();
            let mut f_int = &sys.k_global * &u_full;

            // Process each Cable element
            for el in &model.elements {
                if let Element::Cable {
                    id,
                    node_i,
                    node_j,
                    section_id,
                    material_id,
                    prestress,
                } = el
                {
                    let ni = node_idx_map[node_i.as_str()];
                    let nj = node_idx_map[node_j.as_str()];
                    let xi_0 = model.nodes[ni].x;
                    let yi_0 = model.nodes[ni].y;
                    let xj_0 = model.nodes[nj].x;
                    let yj_0 = model.nodes[nj].y;

                    let sec = sec_map[section_id.as_str()];
                    let mat = mat_map[material_id.as_str()];
                    let ea = mat.e_gpa * 1e6 * sec.area_a;
                    let w0 = mat.density_gamma * sec.area_a;

                    let l0 = compute_unstressed_length(
                        prestress,
                        xj_0 - xi_0,
                        yj_0 - yi_0,
                        ea,
                        w0,
                    );

                    // Deformed end positions
                    let xi = xi_0 + u_full[ni * 3];
                    let yi = yi_0 + u_full[ni * 3 + 1];
                    let xj = xj_0 + u_full[nj * 3];
                    let yj = yj_0 + u_full[nj * 3 + 1];

                    let dx = xj - xi;
                    let dy = yj - yi;
                    let sx = if dx.abs() > 1e-9 { dx.signum() } else { 1.0 };
                    let dx_target = dx.abs().max(1e-4);

                    let (h, v, kt_2x2, is_slack) =
                        solve_irvine_catenary(dx_target, dy, l0, ea, w0)?;
                    if is_slack {
                        slack_warning = true;
                    }

                    let (curve_points, mid_sag, max_tension) =
                        evaluate_catenary_geometry(xi, yi, xj, yj, h, v, l0, ea, w0);

                    // Forces in global coordinates
                    let (fx_i, fy_i, fx_j, fy_j) = if is_slack {
                        (0.0, 0.5 * w0 * l0, 0.0, 0.5 * w0 * l0)
                    } else {
                        (-sx * h, v, sx * h, w0.max(1e-6) * l0 - v)
                    };

                    // 4x4 tangent stiffness
                    let mut k_cable_4x4 = DMatrix::<f64>::zeros(4, 4);
                    // Transforming 2x2 local to 4x4 global
                    for a in 0..2 {
                        for b in 0..2 {
                            let sign_a = if a == 0 { sx } else { 1.0 };
                            let sign_b = if b == 0 { sx } else { 1.0 };
                            let val = kt_2x2[(a, b)] * sign_a * sign_b;
                            k_cable_4x4[(a + 2, b + 2)] = val;
                            k_cable_4x4[(a, b)] = val;
                            k_cable_4x4[(a, b + 2)] = -val;
                            k_cable_4x4[(a + 2, b)] = -val;
                        }
                    }

                    let dofs_cable = [ni * 3, ni * 3 + 1, nj * 3, nj * 3 + 1];
                    for (a, &da) in dofs_cable.iter().enumerate() {
                        for (b, &db) in dofs_cable.iter().enumerate() {
                            k_total[(da, db)] += k_cable_4x4[(a, b)];
                        }
                    }

                    f_int[ni * 3] += fx_i;
                    f_int[ni * 3 + 1] += fy_i;
                    f_int[nj * 3] += fx_j;
                    f_int[nj * 3 + 1] += fy_j;

                    cable_states.push(CatenaryState {
                        element_id: id.clone(),
                        l0,
                        h,
                        v,
                        vi_reaction: fy_i,
                        vj_reaction: fy_j,
                        max_tension,
                        mid_sag,
                        is_slack,
                        curve_points,
                        k_tangent_global: k_cable_4x4,
                        f_internal_global: DVector::from_vec(vec![fx_i, fy_i, fx_j, fy_j]),
                    });
                }
            }

            last_k_global = k_total.clone();

            // Check residual on free DOFs
            let r_global = &f_target - &f_int;
            let mut r_f = DVector::<f64>::zeros(nf);
            for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
                r_f[i] = r_global[fi];
            }

            let r_norm = r_f.norm();
            let f_norm = f_target.norm().max(1.0);
            if r_norm / f_norm < 1e-5 {
                outer_converged = true;
                break;
            }

            // Partition K_total into K_ff
            let mut k_ff = DMatrix::<f64>::zeros(nf, nf);
            for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
                for (j, &fj) in sys.free_dof_indices.iter().enumerate() {
                    k_ff[(i, j)] = k_total[(fi, fj)];
                }
            }

            let du_f = k_ff.clone().cholesky().map(|chol| chol.solve(&r_f)).or_else(|| {
                // SVD fallback if near-slack
                let svd = k_ff.svd(true, true);
                svd.solve(&r_f, 1e-8).ok()
            }).ok_or_else(|| {
                CanonicalError::new(
                    "CABLE_COMPLETE_SLACK_MECHANISM",
                    "Ο φορέας απώλεσε την ευστάθειά του λόγω πλήρους χαλάρωσης καλωδίων.",
                    "Structure lost stability due to complete cable slack mechanism.",
                )
            })?;

            // Backtracking / damped update
            let step = 0.9;
            for (i, &fi) in sys.free_dof_indices.iter().enumerate() {
                u_full[fi] += step * du_f[i];
            }
        }

        if !outer_converged {
            return Err(CanonicalError::new(
                "NONLINEAR_NOT_CONVERGED",
                "Η ανάλυση καλωδίων Irvine δεν συνέκλινε.",
                "Irvine cable analysis failed to converge.",
            ));
        }

        // Build results
        let mut nodal_displacements = Vec::new();
        for (i, node) in model.nodes.iter().enumerate() {
            nodal_displacements.push(NodalDisplacementResult {
                node_id: node.id.clone(),
                ux_m: u_full[i * 3],
                uy_m: u_full[i * 3 + 1],
                theta_rad: u_full[i * 3 + 2],
            });
        }

        let mut support_reactions = Vec::new();
        let r_full = &last_k_global * &u_full - &f_target;
        for sup in &model.supports {
            let ni = node_idx_map[sup.node_id.as_str()];
            support_reactions.push(SupportReactionResult {
                support_id: sup.id.clone(),
                node_id: sup.node_id.clone(),
                rx_kn: r_full[ni * 3],
                ry_kn: r_full[ni * 3 + 1],
                mz_knm: r_full[ni * 3 + 2],
            });
        }

        let mut cable_results = Vec::new();
        for cs in &cable_states {
            cable_results.push(CableResult {
                element_id: cs.element_id.clone(),
                unstressed_length_l0_m: cs.l0,
                chord_length_m: {
                    let el = model.elements.iter().find(|e| e.id() == cs.element_id).unwrap();
                    if let Element::Cable { node_i, node_j, .. } = el {
                        let ni = node_idx_map[node_i.as_str()];
                        let nj = node_idx_map[node_j.as_str()];
                        let xi = model.nodes[ni].x + u_full[ni * 3];
                        let yi = model.nodes[ni].y + u_full[ni * 3 + 1];
                        let xj = model.nodes[nj].x + u_full[nj * 3];
                        let yj = model.nodes[nj].y + u_full[nj * 3 + 1];
                        (xj - xi).hypot(yj - yi)
                    } else {
                        cs.l0
                    }
                },
                horizontal_tension_h_kn: cs.h,
                vertical_reaction_vi_kn: cs.vi_reaction,
                vertical_reaction_vj_kn: cs.vj_reaction,
                max_tension_kn: cs.max_tension,
                mid_sag_m: cs.mid_sag,
                is_slack: cs.is_slack,
                curve_points_global: cs.curve_points.clone(),
            });
        }

        // Frame member results
        let mut member_results = Vec::new();
        for el in &model.elements {
            if let Element::FrameMember {
                id,
                node_i,
                node_j,
                section_id,
                material_id,
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
                let l = (l_nom - offset_i.dx + offset_j.dx).max(1e-4);

                let sec = sec_map[section_id.as_str()];
                let mat = mat_map[material_id.as_str()];

                let dofs = [ni * 3, ni * 3 + 1, ni * 3 + 2, nj * 3, nj * 3 + 1, nj * 3 + 2];
                let mut u_el = DVector::<f64>::zeros(6);
                for (a, &d) in dofs.iter().enumerate() {
                    u_el[a] = u_full[d];
                }
                let t = element_transformation_matrix(cos_a, sin_a, offset_i, offset_j);
                let u_loc = &t * &u_el;

                let k_loc = local_frame_stiffness(
                    mat.e_gpa * 1e6,
                    sec.area_a,
                    sec.inertia_i,
                    l,
                );
                let f_loc = &k_loc * &u_loc;

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

        result_sets.push(ResultSet {
            revision: model.revision,
            target_id: target_comb.id.clone(),
            target_kind: "Combination".to_string(),
            analysis_kind: AnalysisKind::LinearStatic, // or non-linear static with cables
            superposition_valid: false,
            nodal_displacements,
            support_reactions,
            member_results,
            cable_results,
            buckling_modes: Vec::new(),
            dynamic_data: None,
        });
    }

    Ok(result_sets)
}
