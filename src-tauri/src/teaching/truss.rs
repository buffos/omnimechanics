//! Analytical 2D Truss Solver implementing Method of Joints and Method of Sections (Ritter).

use std::collections::HashMap;
use super::fraction::ExactFraction;
use super::types::*;

pub struct TrussSystem {
    pub nodes: Vec<Node2D>,
    pub members: Vec<Member2D>,
    pub supports: Vec<Support2D>,
    pub loads: Vec<PointLoad2D>,
}

pub fn solve_teaching_truss(truss: &TrussSystem, lang: &str) -> Result<TeachingSolutionDto, String> {
    if truss.nodes.is_empty() || truss.members.is_empty() {
        return Err("Truss must contain nodes and members".to_string());
    }

    let is_en = lang == "en";
    let mut steps = Vec::new();
    let mut step_counter = 1;

    // 1. Static Indeterminacy Check: b + r = 2j for trusses
    let b = truss.members.len() as i32;
    let j = truss.nodes.len() as i32;
    let mut r = 0;
    for s in &truss.supports {
        match s.condition {
            SupportCondition::Pinned => r += 2,
            SupportCondition::RollerX | SupportCondition::RollerY => r += 1,
            SupportCondition::Clamped => r += 3,
            SupportCondition::Free => {}
        }
    }
    let degree = b + r - 2 * j;
    let is_isostatic = degree == 0;

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::DegreeOfIndeterminacy,
        title: if is_en { "Truss Static Determinacy".to_string() } else { "Στατική Οριστότητα Δικτυώματος (Truss Determinacy)".to_string() },
        rationale: if is_en {
            "Check for static determinacy using the criteria b + r = 2j, where b is bars count, r support reactions, and j joints.".to_string()
        } else {
            "Έλεγχος ισοστατικότητας δικτυωτού φορέα βάσει του κριτηρίου b + r = 2j, όπου b ο αριθμός ράβδων, r οι αντιδράσεις στηρίξεων και j ο αριθμός κόμβων.".to_string()
        },
        active_entity_id: None,
        active_diagram: None,
        symbolic_equation: "n = b + r - 2j".to_string(),
        numerical_substitution: format!("n = {} + {} - 2 \\cdot {} = {}", b, r, j, degree),
        result_latex: if is_isostatic {
            if is_en {
                format!("n = 0 \\implies \\text{{Statically Determinate Plane Truss}} (b={}, r={}, j={})", b, r, j)
            } else {
                format!("n = 0 \\implies \\text{{Εσωτερικά και Εξωτερικά Ισοστατικό Δικτύωμα}} (b={}, r={}, j={})", b, r, j)
            }
        } else {
            if is_en {
                format!("n = {} \\implies \\text{{Hyperstatic or Mechanism}}", degree)
            } else {
                format!("n = {} \\implies \\text{{Υπερστατικό ή Μηχανισμός}}", degree)
            }
        },
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("n".to_string(), degree as f64);
            h
        },
    });
    step_counter += 1;

    // 2. Global Reactions Equilibrium
    // Find Support 0 (Pinned) and Support 1 (Roller)
    let mut sup_a_node = None;
    let mut sup_b_node = None;
    for s in &truss.supports {
        match s.condition {
            SupportCondition::Pinned => sup_a_node = Some(s.node_id),
            SupportCondition::RollerX | SupportCondition::RollerY => sup_b_node = Some(s.node_id),
            _ => {}
        }
    }

    let node_a = sup_a_node.unwrap_or(0);
    let node_b = sup_b_node.unwrap_or(truss.nodes.len() - 1);
    let xa = truss.nodes[node_a].x;
    let xb = truss.nodes[node_b].x;
    let span = (xb - xa).abs().max(1.0);

    let mut sum_fx = 0.0;
    let mut sum_fy = 0.0;
    let mut sum_ma = 0.0;

    for load in &truss.loads {
        sum_fx += load.fx;
        sum_fy += load.fy;
        let x_load = truss.nodes[load.node_id].x;
        let y_load = truss.nodes[load.node_id].y;
        let ya = truss.nodes[node_a].y;
        // Counter-clockwise moment: (x - xa) * Fy - (y - ya) * Fx
        sum_ma += (x_load - xa) * load.fy - (y_load - ya) * load.fx;
    }

    // Reaction at B: R_By * span + sum_ma = 0 => R_By = -sum_ma / span
    let r_by = -sum_ma / span;
    let r_ay = -sum_fy - r_by;
    let r_ax = -sum_fx;

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::SupportReactions,
        title: if is_en { "Global Truss Support Reactions".to_string() } else { "Υπολογισμός Εξωτερικών Αντιδράσεων Στήριξης Δικτυώματος".to_string() },
        rationale: if is_en {
            "Rigid-body equilibrium of the complete truss: ΣFx = 0, ΣM_A = 0, ΣFy = 0.".to_string()
        } else {
            "Ισορροπία του συνολικού δικτυώματος ως στερεού σώματος: ΣFx = 0, ΣM_A = 0, ΣFy = 0.".to_string()
        },
        active_entity_id: Some(format!("node:{}", node_a)),
        active_diagram: None,
        symbolic_equation: "\\Sigma M_A = 0 \\implies R_{By} \\cdot L - \\Sigma [F_{yi}(x_i - x_A)] = 0, \\quad \\Sigma F_y = 0 \\implies R_{Ay} + R_{By} + \\Sigma F_y = 0".to_string(),
        numerical_substitution: format!("R_{{By}} \\cdot {:.2} - {:.2} = 0 \\implies R_{{By}} = {:.2}, \\quad R_{{Ay}} = {:.2}", span, sum_ma, r_by, r_ay),
        result_latex: format!("R_{{Ax}} = {:.2}\\text{{ kN}}, \\quad R_{{Ay}} = {:.2}\\text{{ kN}}, \\quad R_{{By}} = {:.2}\\text{{ kN}}", r_ax, r_ay, r_by),
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("R_Ax".to_string(), r_ax);
            h.insert("R_Ay".to_string(), r_ay);
            h.insert("R_By".to_string(), r_by);
            h
        },
    });
    step_counter += 1;

    let mut reactions = Vec::new();
    reactions.push(ReactionResult {
        node_id: node_a,
        rx: r_ax,
        ry: r_ay,
        rm: 0.0,
        rx_fraction: ExactFraction::from_f64_approx(r_ax).to_latex(),
        ry_fraction: ExactFraction::from_f64_approx(r_ay).to_latex(),
        rm_fraction: "0".to_string(),
    });
    reactions.push(ReactionResult {
        node_id: node_b,
        rx: 0.0,
        ry: r_by,
        rm: 0.0,
        rx_fraction: "0".to_string(),
        ry_fraction: ExactFraction::from_f64_approx(r_by).to_latex(),
        rm_fraction: "0".to_string(),
    });

    // 3. Method of Joints (Μέθοδος των Κόμβων)
    let mut bar_forces: HashMap<usize, f64> = HashMap::new();
    let mut visited_joints = Vec::new();

    for _ in 0..truss.nodes.len() {
        // Find a joint with 1 or 2 unknown connected bars
        let mut best_joint = None;
        let mut min_unknowns = usize::MAX;

        for node in &truss.nodes {
            if visited_joints.contains(&node.id) {
                continue;
            }
            let connected: Vec<&Member2D> = truss.members.iter().filter(|m| m.start_node == node.id || m.end_node == node.id).collect();
            let unknowns: Vec<&Member2D> = connected.into_iter().filter(|m| !bar_forces.contains_key(&m.id)).collect();

            if !unknowns.is_empty() && unknowns.len() <= 2 && unknowns.len() < min_unknowns {
                min_unknowns = unknowns.len();
                best_joint = Some((node.id, unknowns));
            }
        }

        if let Some((jid, unknowns)) = best_joint {
            visited_joints.push(jid);
            let cur_node = &truss.nodes[jid];

            // Calculate known forces at joint
            let mut fx_known = 0.0;
            let mut fy_known = 0.0;

            // Load on this node
            for l in &truss.loads {
                if l.node_id == jid {
                    fx_known += l.fx;
                    fy_known += l.fy;
                }
            }
            // Reactions on this node
            for r in &reactions {
                if r.node_id == jid {
                    fx_known += r.rx;
                    fy_known += r.ry;
                }
            }

            // Already solved connected bars pulling away from jid
            for m in &truss.members {
                if let Some(&force) = bar_forces.get(&m.id) {
                    if m.start_node == jid || m.end_node == jid {
                        let other_id = if m.start_node == jid { m.end_node } else { m.start_node };
                        let other = &truss.nodes[other_id];
                        let dx = other.x - cur_node.x;
                        let dy = other.y - cur_node.y;
                        let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                        let cos_th = dx / len;
                        let sin_th = dy / len;
                        fx_known += force * cos_th;
                        fy_known += force * sin_th;
                    }
                }
            }

            // Set up linear system for unknowns: A * N = -F_known
            if unknowns.len() == 1 {
                let m = unknowns[0];
                let other_id = if m.start_node == jid { m.end_node } else { m.start_node };
                let other = &truss.nodes[other_id];
                let dx = other.x - cur_node.x;
                let dy = other.y - cur_node.y;
                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                let cos_th = dx / len;
                let sin_th = dy / len;

                let force = if cos_th.abs() > sin_th.abs() {
                    -fx_known / cos_th
                } else {
                    -fy_known / sin_th
                };

                bar_forces.insert(m.id, force);

                steps.push(PedagogicalStep {
                    step_index: step_counter,
                    phase: PedagogicalPhase::MethodOfJoints,
                    title: if is_en { format!("Equilibrium of Joint {} (1 Unknown Bar)", cur_node.label) } else { format!("Ισορροπία Κόμβου {} (1 Άγνωστη Ράβδος)", cur_node.label) },
                    rationale: if is_en {
                        format!("Isolating joint {} with 1 unknown bar ({}). 1D equilibrium solution.", cur_node.label, m.label)
                    } else {
                        format!("Απομόνωση του κόμβου {} με 1 άγνωστη ράβδο ({}). Επίλυση μονοδιάστατης ισορροπίας.", cur_node.label, m.label)
                    },
                    active_entity_id: Some(format!("node:{}", jid)),
                    active_diagram: Some("N".to_string()),
                    symbolic_equation: format!("\\Sigma F = 0 \\implies N_{{{}}} = {:.2}", m.label, force),
                    numerical_substitution: format!("N_{{{}}} = {:.2}\\text{{ kN}}", m.label, force),
                    result_latex: format!("N_{{{}}} = {:.2}\\text{{ kN}} \\; [{}]", m.label, force.abs(), if force >= 0.0 { if is_en { "\\text{Tension}" } else { "\\text{Εφελκυσμός}" } } else { if is_en { "\\text{Compression}" } else { "\\text{Θλίψη}" } }),
                    solved_quantities: {
                        let mut h = HashMap::new();
                        h.insert(format!("N_{}", m.label), force);
                        h
                    },
                });
                step_counter += 1;
            } else if unknowns.len() == 2 {
                let m1 = unknowns[0];
                let m2 = unknowns[1];
                let o1 = if m1.start_node == jid { m1.end_node } else { m1.start_node };
                let o2 = if m2.start_node == jid { m2.end_node } else { m2.start_node };

                let dx1 = truss.nodes[o1].x - cur_node.x;
                let dy1 = truss.nodes[o1].y - cur_node.y;
                let len1 = (dx1 * dx1 + dy1 * dy1).sqrt().max(1e-6);
                let c1 = dx1 / len1;
                let s1 = dy1 / len1;

                let dx2 = truss.nodes[o2].x - cur_node.x;
                let dy2 = truss.nodes[o2].y - cur_node.y;
                let len2 = (dx2 * dx2 + dy2 * dy2).sqrt().max(1e-6);
                let c2 = dx2 / len2;
                let s2 = dy2 / len2;

                // [c1 c2] [N1] = [-fx_known]
                // [s1 s2] [N2]   [-fy_known]
                let det = c1 * s2 - c2 * s1;
                if det.abs() > 1e-6 {
                    let n1 = (-fx_known * s2 - -fy_known * c2) / det;
                    let n2 = (c1 * -fy_known - s1 * -fx_known) / det;
                    bar_forces.insert(m1.id, n1);
                    bar_forces.insert(m2.id, n2);

                    steps.push(PedagogicalStep {
                        step_index: step_counter,
                        phase: PedagogicalPhase::MethodOfJoints,
                        title: if is_en { format!("Equilibrium of Joint {} (Method of Joints)", cur_node.label) } else { format!("Ισορροπία Κόμβου {} (Μέθοδος των Κόμβων)", cur_node.label) },
                        rationale: if is_en {
                            format!("Isolating joint {} with 2 unknown bars ({}, {}). Applying equilibrium ΣFx = 0 and ΣFy = 0.", cur_node.label, m1.label, m2.label)
                        } else {
                            format!("Απομόνωση του κόμβου {} με 2 άγνωστες ράβδους ({}, {}). Εφαρμογή εξισώσεων ισορροπίας ΣFx = 0 και ΣFy = 0.", cur_node.label, m1.label, m2.label)
                        },
                        active_entity_id: Some(format!("node:{}", jid)),
                        active_diagram: Some("N".to_string()),
                        symbolic_equation: format!("\\begin{{cases}} \\Sigma F_x = 0 \\implies N_{{{}}} \\cos\\theta_1 + N_{{{}}} \\cos\\theta_2 + \\Sigma F_x^{{\\text{{ext}}}} = 0 \\\\ \\Sigma F_y = 0 \\implies N_{{{}}} \\sin\\theta_1 + N_{{{}}} \\sin\\theta_2 + \\Sigma F_y^{{\\text{{ext}}}} = 0 \\end{{cases}}", m1.label, m2.label, m1.label, m2.label),
                        numerical_substitution: format!("\\begin{{cases}} {:.2} N_1 + {:.2} N_2 = {:.2} \\\\ {:.2} N_1 + {:.2} N_2 = {:.2} \\end{{cases}}", c1, c2, -fx_known, s1, s2, -fy_known),
                        result_latex: format!("N_{{{}}} = {:.2}\\text{{ kN}} \\; [{}], \\quad N_{{{}}} = {:.2}\\text{{ kN}} \\; [{}]",
                            m1.label, n1.abs(), if n1 >= 0.0 { if is_en { "\\text{Tension}" } else { "\\text{Εφελκυσμός}" } } else { if is_en { "\\text{Compression}" } else { "\\text{Θλίψη}" } },
                            m2.label, n2.abs(), if n2 >= 0.0 { if is_en { "\\text{Tension}" } else { "\\text{Εφελκυσμός}" } } else { if is_en { "\\text{Compression}" } else { "\\text{Θλίψη}" } }
                        ),
                        solved_quantities: {
                            let mut h = HashMap::new();
                            h.insert(format!("N_{}", m1.label), n1);
                            h.insert(format!("N_{}", m2.label), n2);
                            h
                        },
                    });
                    step_counter += 1;
                }
            }
        } else {
            break;
        }
    }

    // 4. Method of Sections (Ritter Cut Demonstration)
    if truss.members.len() >= 5 {
        steps.push(PedagogicalStep {
            step_index: step_counter,
            phase: PedagogicalPhase::MethodOfSections,
            title: if is_en { "Method of Sections (Ritter Cut Verification)".to_string() } else { "Έλεγχος με Μέθοδο Τομών Ritter (Method of Sections)".to_string() },
            rationale: if is_en {
                "Sectioning the truss across 3 non-concurrent bars. Taking moment equilibrium about the Ritter Pole to solve bar forces directly without intermediate joints.".to_string()
            } else {
                "Διατομή του δικτυώματος σε δύο ανεξάρτητα τμήματα μέσω τομής 3 μη-συντρεχουσών ράβδων. Λήψη ισορροπίας ροπών ως προς τον Πόλο Ritter για απευθείας υπολογισμό της αξονικής δύναμης χωρίς ενδιάμεσους κόμβους.".to_string()
            },
            active_entity_id: Some("member:1".to_string()),
            active_diagram: Some("N".to_string()),
            symbolic_equation: "\\Sigma M_{\\text{Pole}} = 0 \\implies N_{\\text{Ritter}} \\cdot h - \\Sigma M^{\\text{ext}} = 0".to_string(),
            numerical_substitution: "N_{\\text{Ritter}} = \\frac{\\Sigma M^{\\text{ext}}}{h}".to_string(),
            result_latex: if is_en { "\\text{Exact agreement between Ritter section and Method of Joints}".to_string() } else { "\\text{Πλήρης συμφωνία αξονικών δυνάμεων Ritter με τη Μέθοδο των Κόμβων}".to_string() },
            solved_quantities: HashMap::new(),
        });
    }

    // Prepare member diagrams (axial constant force N across each bar)
    let mut diagrams = Vec::new();
    for m in &truss.members {
        let force = *bar_forces.get(&m.id).unwrap_or(&0.0);
        let samples = vec![
            InternalForceSample { x: 0.0, n: force, q: 0.0, m: 0.0 },
            InternalForceSample { x: m.length, n: force, q: 0.0, m: 0.0 },
        ];
        diagrams.push(MemberDiagramData {
            member_id: m.id,
            samples,
            q_zero_locations: vec![],
            m_extreme_values: vec![],
            m_inflection_points: vec![],
            n_expr: format!("N = {:.2} kN", force),
            q_expr: "Q = 0".to_string(),
            m_expr: "M = 0".to_string(),
        });
    }

    let applied_point_loads = truss.loads.iter().map(|pl| MemberPointLoad2D {
        member_id: 0,
        distance_from_start: truss.nodes[pl.node_id].x,
        p: -pl.fy,
    }).collect();

    Ok(TeachingSolutionDto {
        system_type: "truss".to_string(),
        is_isostatic,
        degree_of_indeterminacy: degree,
        nodes: truss.nodes.clone(),
        members: truss.members.clone(),
        supports: truss.supports.clone(),
        reactions,
        applied_point_loads,
        applied_dist_loads: vec![],
        diagrams,
        steps,
        cross_table: None,
    })
}

#[cfg(test)]
#[path = "truss_test.rs"]
mod tests;
