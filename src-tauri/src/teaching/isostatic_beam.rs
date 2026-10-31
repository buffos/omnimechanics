//! Analytical solver for isostatic beams and frames with step-by-step pedagogical derivations.

use std::collections::HashMap;
use super::fraction::ExactFraction;
use super::types::*;

pub struct BeamLoadSpec {
    pub point_loads: Vec<(f64, f64)>, // (x, P) downwards positive
    pub dist_loads: Vec<(f64, f64, f64)>, // (x_start, x_end, q) downwards positive
    pub point_moments: Vec<(f64, f64)>, // (x, M) clockwise positive
}

pub fn solve_isostatic_beam_system(
    length: f64,
    supports: &[(f64, SupportCondition)],
    loads: &BeamLoadSpec,
    lang: &str,
) -> Result<TeachingSolutionDto, String> {
    if length <= 0.0 {
        return Err("Length must be positive".to_string());
    }

    let is_en = lang == "en";
    let mut steps = Vec::new();
    let mut step_counter = 1;

    // Step 1: Degree of static indeterminacy
    let mut unknown_reactions = 0;
    for &(_, cond) in supports {
        match cond {
            SupportCondition::Pinned => unknown_reactions += 2,
            SupportCondition::RollerX | SupportCondition::RollerY => unknown_reactions += 1,
            SupportCondition::Clamped => unknown_reactions += 3,
            SupportCondition::Free => {}
        }
    }
    let degree = unknown_reactions - 3;
    let is_isostatic = degree == 0;

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::DegreeOfIndeterminacy,
        title: if is_en { "Degree of Static Indeterminacy".to_string() } else { "Βαθμός Στατικής Αοριστίας (Degree of Indeterminacy)".to_string() },
        rationale: if is_en {
            "Check for static determinacy using the number of unknown support reactions r and the global equations of equilibrium (ΣFx=0, ΣFy=0, ΣM=0).".to_string()
        } else {
            "Έλεγχος στατικής οριστότητας φορέα βάσει του αριθμού άγνωστων αντιδράσεων στήριξης r και των εξισώσεων ισορροπίας (ΣFx=0, ΣFy=0, ΣM=0).".to_string()
        },
        active_entity_id: None,
        active_diagram: None,
        symbolic_equation: "n = r - 3".to_string(),
        numerical_substitution: format!("n = {} - 3", unknown_reactions),
        result_latex: if is_isostatic {
            if is_en { "n = 0 \\implies \\text{Statically Determinate System}".to_string() } else { "n = 0 \\implies \\text{Ισοστατικός Φορέας (Isostatic System)}".to_string() }
        } else {
            if is_en { format!("n = {} \\implies \\text{{Hyperstatic System}}", degree) } else { format!("n = {} \\implies \\text{{Υπερστατικός Φορέας (Hyperstatic)}}", degree) }
        },
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("n".to_string(), degree as f64);
            h
        },
    });
    step_counter += 1;

    // We focus on standard isostatic supports:
    // Case A: Clamped at x = 0 (Cantilever)
    // Case B: Pinned at x_A, Roller at x_B (Simply Supported or Overhanging)
    let is_cantilever = supports.len() == 1 && supports[0].1 == SupportCondition::Clamped;
    let mut reactions = Vec::new();

    let (r_ax, r_ay, r_by, m_a) = if is_cantilever {
        let x_clamp = supports[0].0;
        let mut total_py = 0.0;
        let mut total_m = 0.0;
        for &(x, p) in &loads.point_loads {
            total_py += p;
            total_m += p * (x - x_clamp);
        }
        for &(x1, x2, q) in &loads.dist_loads {
            let q_len = x2 - x1;
            let q_total = q * q_len;
            let q_center = (x1 + x2) / 2.0;
            total_py += q_total;
            total_m += q_total * (q_center - x_clamp);
        }
        for &(_, m) in &loads.point_moments {
            total_m += m;
        }

        // Reaction Step
        steps.push(PedagogicalStep {
            step_index: step_counter,
            phase: PedagogicalPhase::SupportReactions,
            title: if is_en { "Clamped Support Reactions".to_string() } else { "Αντιδράσεις Στήριξης Πάκτωσης (Clamped Support Reactions)".to_string() },
            rationale: if is_en { "Equilibrium of vertical forces and moments at clamp: ΣFy = 0 and ΣM = 0.".to_string() } else { "Ισορροπία δυνάμεων και ροπής στην πάκτωση: ΣFy = 0 και ΣM = 0.".to_string() },
            active_entity_id: Some("node:0".to_string()),
            active_diagram: None,
            symbolic_equation: "R_{Ay} = \\Sigma P_y + \\int q(x) dx, \\quad M_A = \\Sigma [P_i (x_i - x_A)] + \\int q(x)(x - x_A) dx".to_string(),
            numerical_substitution: format!("R_{{Ay}} = {:.2}, \\quad M_A = {:.2}", total_py, total_m),
            result_latex: format!("R_{{Ay}} = {:.2}\\text{{ kN}}, \\quad M_A = {:.2}\\text{{ kNm}}", total_py, total_m),
            solved_quantities: {
                let mut h = HashMap::new();
                h.insert("R_Ay".to_string(), total_py);
                h.insert("M_A".to_string(), total_m);
                h
            },
        });
        step_counter += 1;

        reactions.push(ReactionResult {
            node_id: 0,
            rx: 0.0,
            ry: total_py,
            rm: total_m,
            rx_fraction: "0".to_string(),
            ry_fraction: ExactFraction::from_f64_approx(total_py).to_latex(),
            rm_fraction: ExactFraction::from_f64_approx(total_m).to_latex(),
        });

        (0.0, total_py, 0.0, total_m)
    } else {
        // Find support A (pinned) and support B (roller)
        let (xa, ya_cond) = supports[0];
        let (xb, _) = if supports.len() > 1 { supports[1] } else { (length, SupportCondition::RollerX) };
        let span_ab = xb - xa;

        // Sum moments about A: Σ M_A = 0 => R_By * span_ab - Σ P_i*(x_i - xa) - Σ q*len*(x_c - xa) - Σ M_k = 0
        let mut sum_moment_about_a = 0.0;
        let mut total_vertical_load = 0.0;

        for &(x, p) in &loads.point_loads {
            sum_moment_about_a += p * (x - xa);
            total_vertical_load += p;
        }
        for &(x1, x2, q) in &loads.dist_loads {
            let q_len = x2 - x1;
            let q_res = q * q_len;
            let q_arm = (x1 + x2) / 2.0 - xa;
            sum_moment_about_a += q_res * q_arm;
            total_vertical_load += q_res;
        }
        for &(_, m) in &loads.point_moments {
            sum_moment_about_a += m;
        }

        let r_by_val = sum_moment_about_a / span_ab;
        let r_ay_val = total_vertical_load - r_by_val;

        // Step: Σ M_A = 0
        steps.push(PedagogicalStep {
            step_index: step_counter,
            phase: PedagogicalPhase::SupportReactions,
            title: if is_en { "Moment Equilibrium about Node A (ΣM_A = 0)".to_string() } else { "Ισορροπία Ροπών περί τον Κόμβο A (ΣM_A = 0)".to_string() },
            rationale: if is_en { "Moment equilibrium about support A to determine vertical reaction R_By.".to_string() } else { "Λήψη ροπών ως προς την άρθρωση A για τον προσδιορισμό της κατακόρυφης αντίδρασης στήριξης R_By.".to_string() },
            active_entity_id: Some("node:1".to_string()),
            active_diagram: None,
            symbolic_equation: "\\Sigma M_A = 0 \\implies R_{By} \\cdot L_{AB} - \\Sigma [P_i (x_i - x_A)] - \\Sigma \\left[q_j \\cdot l_j \\cdot d_j\\right] = 0".to_string(),
            numerical_substitution: format!("R_{{By}} \\cdot {:.2} - {:.2} = 0", span_ab, sum_moment_about_a),
            result_latex: format!("R_{{By}} = \\frac{{{:.2}}}{{{:.2}}} = {:.2}\\text{{ kN}}", sum_moment_about_a, span_ab, r_by_val),
            solved_quantities: {
                let mut h = HashMap::new();
                h.insert("R_By".to_string(), r_by_val);
                h
            },
        });
        step_counter += 1;

        // Step: Σ F_y = 0
        steps.push(PedagogicalStep {
            step_index: step_counter,
            phase: PedagogicalPhase::SupportReactions,
            title: if is_en { "Vertical Force Equilibrium (ΣFy = 0)".to_string() } else { "Κατακόρυφη Ισορροπία Δυνάμεων (ΣFy = 0)".to_string() },
            rationale: if is_en { "Equilibrium along y-axis to determine reaction R_Ay.".to_string() } else { "Ισορροπία στον άξονα y για τον προσδιορισμό της αντίδρασης R_Ay.".to_string() },
            active_entity_id: Some("node:0".to_string()),
            active_diagram: None,
            symbolic_equation: "\\Sigma F_y = 0 \\implies R_{Ay} + R_{By} - \\Sigma P_i - \\Sigma Q_j = 0".to_string(),
            numerical_substitution: format!("R_{{Ay}} + {:.2} - {:.2} = 0", r_by_val, total_vertical_load),
            result_latex: format!("R_{{Ay}} = {:.2} - {:.2} = {:.2}\\text{{ kN}}", total_vertical_load, r_by_val, r_ay_val),
            solved_quantities: {
                let mut h = HashMap::new();
                h.insert("R_Ay".to_string(), r_ay_val);
                h
            },
        });
        step_counter += 1;

        reactions.push(ReactionResult {
            node_id: 0,
            rx: 0.0,
            ry: r_ay_val,
            rm: 0.0,
            rx_fraction: "0".to_string(),
            ry_fraction: ExactFraction::from_f64_approx(r_ay_val).to_latex(),
            rm_fraction: "0".to_string(),
        });
        reactions.push(ReactionResult {
            node_id: 1,
            rx: 0.0,
            ry: r_by_val,
            rm: 0.0,
            rx_fraction: "0".to_string(),
            ry_fraction: ExactFraction::from_f64_approx(r_by_val).to_latex(),
            rm_fraction: "0".to_string(),
        });

        (0.0, r_ay_val, r_by_val, 0.0)
    };

    // Helper to evaluate internal forces at any station x
    // If include_at_x is true, loads/supports located at x are included (x^+).
    // If false, they are not yet included (x^-).
    let eval_forces = |x: f64, include_at_x: bool| -> (f64, f64) {
        let mut q_x = 0.0;
        let mut m_x = 0.0;
        let at_test = |pos: f64| -> bool {
            if include_at_x {
                pos <= x + 1e-9
            } else {
                pos < x - 1e-9
            }
        };

        // Support A
        let (xa, _) = supports[0];
        if at_test(xa) {
            q_x += r_ay;
            m_x += r_ay * (x - xa);
            if is_cantilever {
                m_x -= m_a;
            }
        }
        // Support B
        if supports.len() > 1 {
            let (xb, _) = supports[1];
            if at_test(xb) {
                q_x += r_by;
                m_x += r_by * (x - xb);
            }
        }

        // Point loads
        for &(px, p) in &loads.point_loads {
            if at_test(px) {
                q_x -= p;
                m_x -= p * (x - px);
            }
        }

        // Distributed loads
        for &(x1, x2, q) in &loads.dist_loads {
            if x > x1 {
                let eff_x = x.min(x2);
                let q_len = eff_x - x1;
                let q_force = q * q_len;
                let q_arm = x - (x1 + eff_x) / 2.0;
                q_x -= q_force;
                m_x -= q_force * q_arm;
            }
        }

        // Point moments
        for &(mx, m) in &loads.point_moments {
            if at_test(mx) {
                m_x -= m;
            }
        }

        (q_x, m_x)
    };

    // Step: Internal force functions M(x), Q(x), N(x)
    // Identify critical stations in [0, length]
    let mut stations = vec![0.0, length];
    for &(x, _) in supports {
        if x >= 0.0 && x <= length { stations.push(x); }
    }
    for &(px, _) in &loads.point_loads {
        if px >= 0.0 && px <= length { stations.push(px); }
    }
    for &(x1, x2, _) in &loads.dist_loads {
        if x1 >= 0.0 && x1 <= length { stations.push(x1); }
        if x2 >= 0.0 && x2 <= length { stations.push(x2); }
    }
    stations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    stations.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

    let mut samples = Vec::new();
    let mut q_zeros = Vec::new();
    let mut m_extremes = Vec::new();
    let mut m_inflections = Vec::new();

    // Start with left boundary at 0.0 before reaction (for vertical step at x=0)
    let (q_0_left, m_0_left) = eval_forces(0.0, false);
    samples.push(InternalForceSample { x: 0.0, n: 0.0, q: q_0_left, m: m_0_left });

    for idx in 0..(stations.len() - 1) {
        let x_start = stations[idx];
        let x_end = stations[idx + 1];
        let seg_len = x_end - x_start;
        if seg_len < 1e-6 { continue; }

        let n_sub = 25;
        for s in 0..=n_sub {
            let frac = s as f64 / n_sub as f64;
            let x = x_start + frac * seg_len;

            // At x_start of segment, we take the right limit (include_at_x = true)
            // At x_end of segment, we take the left limit (include_at_x = false)
            let include_at_x = s == 0;
            let (q_val, m_val) = eval_forces(x, include_at_x);

            if let Some(prev) = samples.last() {
                if prev.x < x && prev.q * q_val <= 0.0 && (prev.q - q_val).abs() > 1e-6 {
                    let root_x = prev.x + (x - prev.x) * (prev.q / (prev.q - q_val));
                    if root_x > 0.05 * length && root_x < 0.95 * length {
                        q_zeros.push(root_x);
                    }
                }
                if prev.x < x && prev.m * m_val <= 0.0 && (prev.m - m_val).abs() > 1e-6 {
                    let inf_x = prev.x + (x - prev.x) * (prev.m / (prev.m - m_val));
                    if inf_x > 0.05 * length && inf_x < 0.95 * length {
                        m_inflections.push(inf_x);
                    }
                }
            }

            samples.push(InternalForceSample {
                x,
                n: 0.0,
                q: q_val,
                m: m_val,
            });
        }
    }

    // At right boundary x = length, evaluate right limit (closes vertically to 0)
    let (q_end_right, m_end_right) = eval_forces(length, true);
    samples.push(InternalForceSample { x: length, n: 0.0, q: q_end_right, m: m_end_right });

    // Dedup adjacent identical q_zeros and compute extreme moments
    q_zeros.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
    for &xz in &q_zeros {
        let (_, m_at_z) = eval_forces(xz, true);
        m_extremes.push(m_at_z);
    }

    // Step: Internal force diagrams
    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::InternalForcesPiecewise,
        title: if is_en { "Shear Force Diagram Q(x) & Zero Crossings".to_string() } else { "Διάγραμμα Τεμνουσών Δυνάμεων Q(x) & Μηδενισμοί".to_string() },
        rationale: if is_en {
            "Shear force function Q(x) from load integral: dQ/dx = -q(x). At points where Q(x) = 0, bending moment reaches its maximum value (M_max).".to_string()
        } else {
            "Η τέμνουσα δύναμη Q(x) υπολογίζεται από το ολοκλήρωμα του κατανεμημένου φορτίου: dQ/dx = -q(x). Στα σημεία όπου Q(x) = 0, η καμπτική ροπή M(x) λαμβάνει ακρότατη τιμή (M_max).".to_string()
        },
        active_entity_id: Some("member:0".to_string()),
        active_diagram: Some("Q".to_string()),
        symbolic_equation: "Q(x) = R_{Ay} - \\int q(x) dx - \\Sigma P_i = 0 \\implies x_0".to_string(),
        numerical_substitution: if !q_zeros.is_empty() {
            format!("Q(x_0) = 0 \\implies x_0 = {:.3}\\text{{ m}}", q_zeros[0])
        } else {
            if is_en { "Q(x) \\neq 0 \\text{ within span}".to_string() } else { "Q(x) \\neq 0 \\text{ εντός του ανοίγματος}".to_string() }
        },
        result_latex: if !q_zeros.is_empty() {
            format!("x_0 = {:.3}\\text{{ m}} \\quad ({})", q_zeros[0], if is_en { "\\text{Peak Moment Station}" } else { "\\text{Θέση Μέγιστης Ροπής}" })
        } else {
            if is_en { "Monotonic shear variation".to_string() } else { "Μονοτονική μεταβολή τέμνουσας δύναμης".to_string() }
        },
        solved_quantities: {
            let mut h = HashMap::new();
            if !q_zeros.is_empty() {
                h.insert("x_zero".to_string(), q_zeros[0]);
            }
            h
        },
    });
    step_counter += 1;

    // Step: Maximum Bending Moment M_max
    if !q_zeros.is_empty() && !m_extremes.is_empty() {
        steps.push(PedagogicalStep {
            step_index: step_counter,
            phase: PedagogicalPhase::CriticalExtrema,
            title: if is_en { "Maximum Bending Moment M_max at Zero Shear Station".to_string() } else { "Μέγιστη Καμπτική Ροπή M_max στο Σημείο Μηδενισμού Τέμνουσας".to_string() },
            rationale: if is_en {
                "Stationary extremum theorem: dM/dx = Q(x). Since Q(x_0) = 0, the bending moment is maximal at x_0.".to_string()
            } else {
                "Θεώρημα Ακροτάτων: dM/dx = Q(x). Εφόσον Q(x_0) = 0, η ροπή M(x_0) ισούται με τη μέγιστη τιμή στο άνοιγμα.".to_string()
            },
            active_entity_id: Some("member:0".to_string()),
            active_diagram: Some("M".to_string()),
            symbolic_equation: "M_{\\max} = M(x_0) = R_{Ay} \\cdot x_0 - \\frac{q \\cdot x_0^2}{2}".to_string(),
            numerical_substitution: format!("M({:.3}) = {:.2} \\cdot {:.3} - \\dots", q_zeros[0], r_ay, q_zeros[0]),
            result_latex: format!("M_{{\\max}} = {:.2}\\text{{ kNm}}", m_extremes[0]),
            solved_quantities: {
                let mut h = HashMap::new();
                h.insert("M_max".to_string(), m_extremes[0]);
                h
            },
        });
    }

    let nodes = vec![
        Node2D { id: 0, x: 0.0, y: 0.0, label: "A".to_string() },
        Node2D { id: 1, x: length, y: 0.0, label: "B".to_string() },
    ];
    let members = vec![
        Member2D {
            id: 0,
            start_node: 0,
            end_node: 1,
            length,
            ea: 30_000_000.0,
            ei: 100_000.0,
            label: "AB".to_string(),
        }
    ];
    let sup_list = supports.iter().enumerate().map(|(idx, &(x, cond))| {
        Support2D {
            node_id: if x > length / 2.0 { 1 } else { 0 },
            condition: cond,
        }
    }).collect();

    let diagram_data = MemberDiagramData {
        member_id: 0,
        samples,
        q_zero_locations: q_zeros,
        m_extreme_values: m_extremes,
        m_inflection_points: m_inflections,
        n_expr: "N(x) = 0".to_string(),
        q_expr: "Q(x) = R_{Ay} - q*x".to_string(),
        m_expr: "M(x) = R_{Ay}*x - q*x^2 / 2".to_string(),
    };

    let applied_point_loads: Vec<MemberPointLoad2D> = loads.point_loads.iter().map(|&(px, p)| {
        MemberPointLoad2D {
            member_id: 0,
            distance_from_start: px,
            p,
        }
    }).collect();

    let applied_dist_loads: Vec<DistributedLoad2D> = loads.dist_loads.iter().map(|&(x1, x2, q)| {
        DistributedLoad2D {
            member_id: 0,
            q_start: q,
            q_end: q,
            direction: "global_y".to_string(),
        }
    }).collect();

    Ok(TeachingSolutionDto {
        system_type: if is_cantilever { "cantilever".to_string() } else { "beam".to_string() },
        is_isostatic: true,
        degree_of_indeterminacy: 0,
        nodes,
        members,
        supports: sup_list,
        reactions,
        applied_point_loads,
        applied_dist_loads,
        diagrams: vec![diagram_data],
        steps,
        cross_table: None,
    })
}

#[cfg(test)]
#[path = "isostatic_beam_test.rs"]
mod tests;
