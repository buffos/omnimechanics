//! Analytical solver for Moment Distribution (Cross Method) and Kani Iteration for continuous beams.

use std::collections::HashMap;
use super::types::*;

pub struct ContinuousBeamConfig {
    pub l1: f64,
    pub l2: f64,
    pub q1: f64, // Distributed load on span 1 (kN/m)
    pub p2: f64, // Center point load on span 2 (kN)
    pub left_fixed: bool,
    pub right_fixed: bool,
}

pub fn solve_continuous_beam_cross(config: &ContinuousBeamConfig, lang: &str) -> Result<TeachingSolutionDto, String> {
    if config.l1 <= 0.0 || config.l2 <= 0.0 {
        return Err("Span lengths must be positive".to_string());
    }

    let is_en = lang == "en";
    let mut steps = Vec::new();
    let mut step_counter = 1;

    // 1. Degree of Indeterminacy
    // Two-span continuous beam with intermediate support: hyperstatic (degree = 1 for simple ends, degree = 2-3 for fixed)
    let degree = (if config.left_fixed { 1 } else { 0 }) + 1 + (if config.right_fixed { 1 } else { 0 });

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::DegreeOfIndeterminacy,
        title: if is_en { "Continuous Beam Static Indeterminacy".to_string() } else { "Στατική Αοριστία Συνεχούς Δοκού (Static Indeterminacy)".to_string() },
        rationale: if is_en {
            "The two-span continuous beam is hyperstatic. The Moment Distribution Method (Cross) solves support moments iteratively without simultaneous equations.".to_string()
        } else {
            "Η συνεχής δοκός δύο ανοιγμάτων είναι υπερστατικός φορέας. Η μέθοδος Cross (Moment Distribution) επιλύει τις ροπές στους κόμβους χωρίς άμεση επίλυση συστήματος εξισώσεων μέσω διαδοχικών κύκλων κατανομής και μεταβίβασης.".to_string()
        },
        active_entity_id: None,
        active_diagram: None,
        symbolic_equation: "n = r - 3".to_string(),
        numerical_substitution: if is_en { format!("n = {} \\implies \\text{{Hyperstatic System}}", degree) } else { format!("n = {} \\implies \\text{{Υπερστατικός Φορέας}}", degree) },
        result_latex: if is_en {
            format!("n = {} \\quad (\\text{{Solved via Cross \\& Kani Method}})", degree)
        } else {
            format!("n = {} \\quad (\\text{{Επίλυση με Μέθοδο Cross \\& Kani}})", degree)
        },
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("n".to_string(), degree as f64);
            h
        },
    });
    step_counter += 1;

    // 2. Relative Stiffness and Distribution Factors (DF)
    // Assume constant EI
    let k_ba = if config.left_fixed { 4.0 / config.l1 } else { 3.0 / config.l1 };
    let k_bc = if config.right_fixed { 4.0 / config.l2 } else { 3.0 / config.l2 };
    let sum_k_b = k_ba + k_bc;

    let df_ba = k_ba / sum_k_b;
    let df_bc = k_bc / sum_k_b;
    let df_ab = if config.left_fixed { 0.0 } else { 1.0 };
    let df_cb = if config.right_fixed { 0.0 } else { 1.0 };

    let mut dist_factors = HashMap::new();
    dist_factors.insert("AB".to_string(), df_ab);
    dist_factors.insert("BA".to_string(), df_ba);
    dist_factors.insert("BC".to_string(), df_bc);
    dist_factors.insert("CB".to_string(), df_cb);

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::CrossIteration,
        title: if is_en { "Stiffness & Distribution Factors (DF) at Joint B".to_string() } else { "Υπολογισμός Συντελεστών Κατανομής DF (Distribution Factors)".to_string() },
        rationale: if is_en {
            "Member relative stiffness k = 4EI/L (or 3EI/L for hinged end) and distribution factors DF = k / Σk at intermediate joint B.".to_string()
        } else {
            "Υπολογισμός της δυσκαμψίας κάθε ράβδου k = 4EI/L (ή 3EI/L για αρθρωτό άκρο) και των συντελεστών κατανομής DF = k / Σk στον ενδιάμεσο κόμβο B.".to_string()
        },
        active_entity_id: Some("node:1".to_string()),
        active_diagram: None,
        symbolic_equation: "DF_{BA} = \\frac{k_{BA}}{k_{BA} + k_{BC}}, \\quad DF_{BC} = \\frac{k_{BC}}{k_{BA} + k_{BC}}".to_string(),
        numerical_substitution: format!("k_{{BA}} = {:.4}, \\; k_{{BC}} = {:.4} \\implies DF_{{BA}} = {:.3}, \\; DF_{{BC}} = {:.3}", k_ba, k_bc, df_ba, df_bc),
        result_latex: format!("DF_{{BA}} = {:.3}, \\quad DF_{{BC}} = {:.3} \\quad (\\Sigma DF = {:.1})", df_ba, df_bc, df_ba + df_bc),
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("DF_BA".to_string(), df_ba);
            h.insert("DF_BC".to_string(), df_bc);
            h
        },
    });
    step_counter += 1;

    // 3. Fixed End Moments (FEM)
    // Span 1: q1 -> FEM_AB = -q*L^2/12, FEM_BA = +q*L^2/12
    let fem_ab = -config.q1 * config.l1 * config.l1 / 12.0;
    let fem_ba = config.q1 * config.l1 * config.l1 / 12.0;

    // Span 2: P2 at midspan -> FEM_BC = -P*L/8, FEM_CB = +P*L/8
    let fem_bc = -config.p2 * config.l2 / 8.0;
    let fem_cb = config.p2 * config.l2 / 8.0;

    let mut fem_map = HashMap::new();
    fem_map.insert("AB".to_string(), fem_ab);
    fem_map.insert("BA".to_string(), fem_ba);
    fem_map.insert("BC".to_string(), fem_bc);
    fem_map.insert("CB".to_string(), fem_cb);

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::CrossIteration,
        title: if is_en { "Fixed-End Moments (FEM)".to_string() } else { "Ροπές Τέλους Πάκτωσης FEM (Fixed-End Moments)".to_string() },
        rationale: if is_en {
            "Fixed-end moment evaluation under span loading: Uniform load q -> FEM = ±qL²/12, Central point load P -> FEM = ±PL/8.".to_string()
        } else {
            "Υπολογισμός των τέλειων πακτώσεων λόγω φορτίων στα ανοίγματα: Ομοιόμορφο q -> FEM = ±qL²/12, Συγκεντρωμένο P στο μέσο -> FEM = ±PL/8.".to_string()
        },
        active_entity_id: None,
        active_diagram: Some("M".to_string()),
        symbolic_equation: "FEM_{AB} = -\\frac{q_1 L_1^2}{12}, \\; FEM_{BA} = +\\frac{q_1 L_1^2}{12}, \\; FEM_{BC} = -\\frac{P_2 L_2}{8}, \\; FEM_{CB} = +\\frac{P_2 L_2}{8}".to_string(),
        numerical_substitution: format!("FEM_{{AB}} = -\\frac{{{:.1} \\cdot {:.1}^2}}{{12}} = {:.2}, \\quad FEM_{{BC}} = -\\frac{{{:.1} \\cdot {:.1}}}{{8}} = {:.2}", config.q1, config.l1, fem_ab, config.p2, config.l2, fem_bc),
        result_latex: format!("FEM_{{AB}} = {:.2}, \\; FEM_{{BA}} = {:.2}, \\; FEM_{{BC}} = {:.2}, \\; FEM_{{CB}} = {:.2}\\text{{ kNm}}", fem_ab, fem_ba, fem_bc, fem_cb),
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("FEM_AB".to_string(), fem_ab);
            h.insert("FEM_BA".to_string(), fem_ba);
            h.insert("FEM_BC".to_string(), fem_bc);
            h.insert("FEM_CB".to_string(), fem_cb);
            h
        },
    });
    step_counter += 1;

    // 4. Moment Distribution Iteration Cycles (Cross Table)
    let mut cycles = Vec::new();

    // Initial FEM row
    let mut cur_ab = fem_ab;
    let mut cur_ba = fem_ba;
    let mut cur_bc = fem_bc;
    let mut cur_cb = fem_cb;

    let mut tot_ab = cur_ab;
    let mut tot_ba = cur_ba;
    let mut tot_bc = cur_bc;
    let mut tot_cb = cur_cb;

    for cycle in 1..=4 {
        // Balancing at joint B:
        let unbal_b = cur_ba + cur_bc;
        let dist_ba = -df_ba * unbal_b;
        let dist_bc = -df_bc * unbal_b;

        let mut dist_row = HashMap::new();
        dist_row.insert("AB".to_string(), 0.0);
        dist_row.insert("BA".to_string(), dist_ba);
        dist_row.insert("BC".to_string(), dist_bc);
        dist_row.insert("CB".to_string(), 0.0);
        cycles.push(CrossCycleRow {
            cycle_index: cycle,
            operation: format!("Κατανομή Κύκλου {} (Distribution)", cycle),
            member_end_moments: dist_row,
        });

        tot_ba += dist_ba;
        tot_bc += dist_bc;

        // Carry-over (COF = 0.5) to A and C (if fixed)
        let co_ab = if config.left_fixed { 0.5 * dist_ba } else { 0.0 };
        let co_cb = if config.right_fixed { 0.5 * dist_bc } else { 0.0 };

        let mut co_row = HashMap::new();
        co_row.insert("AB".to_string(), co_ab);
        co_row.insert("BA".to_string(), 0.0);
        co_row.insert("BC".to_string(), 0.0);
        co_row.insert("CB".to_string(), co_cb);
        cycles.push(CrossCycleRow {
            cycle_index: cycle,
            operation: format!("Μεταβίβαση Κύκλου {} (Carry-Over 0.5)", cycle),
            member_end_moments: co_row,
        });

        tot_ab += co_ab;
        tot_cb += co_cb;

        cur_ba = 0.0;
        cur_bc = 0.0;
        cur_ab = co_ab;
        cur_cb = co_cb;

        if unbal_b.abs() < 0.02 {
            break;
        }
    }

    let mut final_moments = HashMap::new();
    final_moments.insert("AB".to_string(), tot_ab);
    final_moments.insert("BA".to_string(), tot_ba);
    final_moments.insert("BC".to_string(), tot_bc);
    final_moments.insert("CB".to_string(), tot_cb);

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::CrossIteration,
        title: if is_en { "Cross Table Convergence & Final Support Moments".to_string() } else { "Σύγκλιση Πίνακα Cross & Τελικές Ροπές Στηρίξεων".to_string() },
        rationale: if is_en {
            "Summation of all distribution and carry-over cycles. Joint B reaches exact moment equilibrium: M_BA + M_BC = 0.".to_string()
        } else {
            "Άθροιση όλων των κύκλων κατανομής και μεταβίβασης. Στον κόμβο B επιτυγχάνεται τέλεια ισορροπία ροπών: M_BA + M_BC = 0.".to_string()
        },
        active_entity_id: Some("node:1".to_string()),
        active_diagram: Some("M".to_string()),
        symbolic_equation: "M_{\\text{final}} = FEM + \\Sigma \\Delta M_{\\text{dist}} + \\Sigma \\Delta M_{\\text{carry}}".to_string(),
        numerical_substitution: format!("M_{{BA}} = {:.2}\\text{{ kNm}}, \\; M_{{BC}} = {:.2}\\text{{ kNm}}", tot_ba, tot_bc),
        result_latex: format!("M_{{AB}} = {:.2}, \\; M_{{BA}} = {:.2}, \\; M_{{BC}} = {:.2}, \\; M_{{CB}} = {:.2}\\text{{ kNm}}", tot_ab, tot_ba, tot_bc, tot_cb),
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("M_AB".to_string(), tot_ab);
            h.insert("M_BA".to_string(), tot_ba);
            h.insert("M_BC".to_string(), tot_bc);
            h.insert("M_CB".to_string(), tot_cb);
            h
        },
    });
    step_counter += 1;

    // 5. Reactions and Internal Force Diagrams (M, Q, N)
    let m_a = if config.left_fixed { tot_ab.abs() } else { 0.0 };
    let m_b = tot_ba.abs();
    let m_c = if config.right_fixed { tot_cb.abs() } else { 0.0 };

    // Span 1 equilibrium:
    let v_a = config.q1 * config.l1 / 2.0 + (m_a - m_b) / config.l1;
    let v_b1 = config.q1 * config.l1 - v_a;

    // Span 2 equilibrium:
    let v_b2 = config.p2 / 2.0 + (m_b - m_c) / config.l2;
    let v_c = config.p2 - v_b2;

    let r_a = v_a;
    let r_b = v_b1 + v_b2;
    let r_c = v_c;

    let reactions = vec![
        ReactionResult {
            node_id: 0,
            rx: 0.0,
            ry: r_a,
            rm: if config.left_fixed { -m_a } else { 0.0 },
            rx_fraction: "0".to_string(),
            ry_fraction: format!("{:.2}", r_a),
            rm_fraction: if config.left_fixed { format!("{:.2}", -m_a) } else { "0".to_string() },
        },
        ReactionResult {
            node_id: 1,
            rx: 0.0,
            ry: r_b,
            rm: 0.0,
            rx_fraction: "0".to_string(),
            ry_fraction: format!("{:.2}", r_b),
            rm_fraction: "0".to_string(),
        },
        ReactionResult {
            node_id: 2,
            rx: 0.0,
            ry: r_c,
            rm: if config.right_fixed { m_c } else { 0.0 },
            rx_fraction: "0".to_string(),
            ry_fraction: format!("{:.2}", r_c),
            rm_fraction: if config.right_fixed { format!("{:.2}", m_c) } else { "0".to_string() },
        },
    ];

    // Member 0 (Span AB) internal force diagrams
    let mut samples_0 = Vec::new();
    let mut q_zeros_0 = Vec::new();
    let mut m_extremes_0 = Vec::new();

    samples_0.push(InternalForceSample { x: 0.0, n: 0.0, q: 0.0, m: -m_a });
    samples_0.push(InternalForceSample { x: 0.0, n: 0.0, q: v_a, m: -m_a });

    let n_pts = 40;
    for i in 1..n_pts {
        let x = (i as f64 / n_pts as f64) * config.l1;
        let q_x = v_a - config.q1 * x;
        let m_x = -m_a + v_a * x - 0.5 * config.q1 * x * x;
        samples_0.push(InternalForceSample { x, n: 0.0, q: q_x, m: m_x });
    }

    samples_0.push(InternalForceSample { x: config.l1, n: 0.0, q: -v_b1, m: -m_b });
    samples_0.push(InternalForceSample { x: config.l1, n: 0.0, q: 0.0, m: -m_b });

    if config.q1 > 1e-4 && v_a > 0.0 {
        let x_0 = v_a / config.q1;
        if x_0 > 0.05 * config.l1 && x_0 < 0.95 * config.l1 {
            let m_max_1 = -m_a + (v_a * v_a) / (2.0 * config.q1);
            q_zeros_0.push(x_0);
            m_extremes_0.push(m_max_1);
        }
    }

    let diag_0 = MemberDiagramData {
        member_id: 0,
        samples: samples_0,
        q_zero_locations: q_zeros_0.clone(),
        m_extreme_values: m_extremes_0.clone(),
        m_inflection_points: vec![],
        n_expr: "N(x) = 0".to_string(),
        q_expr: format!("Q(x) = {:.2} - {:.2}x", v_a, config.q1),
        m_expr: format!("M(x) = {:.2} + {:.2}x - {:.2}x^2", -m_a, v_a, 0.5 * config.q1),
    };

    // Member 1 (Span BC) internal force diagrams
    let mut samples_1 = Vec::new();
    let mut q_zeros_1 = Vec::new();
    let mut m_extremes_1 = Vec::new();

    let x_mid = config.l2 / 2.0;
    let m_mid = -m_b + v_b2 * x_mid;

    samples_1.push(InternalForceSample { x: 0.0, n: 0.0, q: 0.0, m: -m_b });
    samples_1.push(InternalForceSample { x: 0.0, n: 0.0, q: v_b2, m: -m_b });

    let half_pts = 20;
    for i in 1..half_pts {
        let x = (i as f64 / half_pts as f64) * x_mid;
        let q_x = v_b2;
        let m_x = -m_b + v_b2 * x;
        samples_1.push(InternalForceSample { x, n: 0.0, q: q_x, m: m_x });
    }

    samples_1.push(InternalForceSample { x: x_mid, n: 0.0, q: v_b2, m: m_mid });
    samples_1.push(InternalForceSample { x: x_mid, n: 0.0, q: -v_c, m: m_mid });

    for i in 1..half_pts {
        let x = x_mid + (i as f64 / half_pts as f64) * x_mid;
        let q_x = -v_c;
        let m_x = -m_b + v_b2 * x - config.p2 * (x - x_mid);
        samples_1.push(InternalForceSample { x, n: 0.0, q: q_x, m: m_x });
    }

    samples_1.push(InternalForceSample { x: config.l2, n: 0.0, q: -v_c, m: -m_c });
    samples_1.push(InternalForceSample { x: config.l2, n: 0.0, q: 0.0, m: -m_c });

    q_zeros_1.push(x_mid);
    m_extremes_1.push(m_mid);

    let diag_1 = MemberDiagramData {
        member_id: 1,
        samples: samples_1,
        q_zero_locations: q_zeros_1.clone(),
        m_extreme_values: m_extremes_1.clone(),
        m_inflection_points: vec![],
        n_expr: "N(x) = 0".to_string(),
        q_expr: format!("Q(x) = {:.2} (x < {:.1}), \\; {:.2} (x > {:.1})", v_b2, x_mid, -v_c, x_mid),
        m_expr: format!("M(x) = {:.2} + {:.2}x (x \\le {:.1})", -m_b, v_b2, x_mid),
    };

    let diagrams = vec![diag_0, diag_1];

    steps.push(PedagogicalStep {
        step_index: step_counter,
        phase: PedagogicalPhase::InternalForcesPiecewise,
        title: if is_en { "Support Reactions & Internal Force Diagrams (M, Q)".to_string() } else { "Αντιδράσεις Στήριξης & Διαγράμματα Εντατικών Μεγεθών (M, Q)".to_string() },
        rationale: if is_en {
            "Isolating spans AB and BC to determine vertical support reactions and continuous bending moment M(x) and shear force Q(x) diagrams.".to_string()
        } else {
            "Απομόνωση των επιμέρους ανοιγμάτων AB και BC για τον προσδιορισμό των κατακόρυφων αντιδράσεων και τη χάραξη των διαγραμμάτων ροπών κάμψης M(x) και τεμνουσών Q(x).".to_string()
        },
        active_entity_id: None,
        active_diagram: Some("M".to_string()),
        symbolic_equation: "V_A = \\frac{q_1 L_1}{2} + \\frac{M_A - M_B}{L_1}, \\quad V_C = \\frac{P_2}{2} + \\frac{M_C - M_B}{L_2}, \\quad R_B = V_{B1} + V_{B2}".to_string(),
        numerical_substitution: format!(
            "R_{{Ay}} = {:.2}\\text{{ kN}}, \\quad R_{{By}} = {:.2}\\text{{ kN}}, \\quad R_{{Cy}} = {:.2}\\text{{ kN}}",
            r_a, r_b, r_c
        ),
        result_latex: format!(
            "R_{{Ay}} = {:.2}, \\; R_{{By}} = {:.2}, \\; R_{{Cy}} = {:.2}\\text{{ kN}} \\quad (M_{{max,1}} = {:.2}, \\; M_{{max,2}} = {:.2}\\text{{ kNm}})",
            r_a, r_b, r_c,
            m_extremes_0.first().copied().unwrap_or(0.0),
            m_mid
        ),
        solved_quantities: {
            let mut h = HashMap::new();
            h.insert("R_Ay".to_string(), r_a);
            h.insert("R_By".to_string(), r_b);
            h.insert("R_Cy".to_string(), r_c);
            h.insert("M_max_Span1".to_string(), m_extremes_0.first().copied().unwrap_or(0.0));
            h.insert("M_max_Span2".to_string(), m_mid);
            h
        },
    });

    let cross_table = CrossTable {
        distribution_factors: dist_factors,
        fixed_end_moments: fem_map,
        cycles,
        final_moments,
    };

    let total_len = config.l1 + config.l2;
    let nodes = vec![
        Node2D { id: 0, x: 0.0, y: 0.0, label: "A".to_string() },
        Node2D { id: 1, x: config.l1, y: 0.0, label: "B".to_string() },
        Node2D { id: 2, x: total_len, y: 0.0, label: "C".to_string() },
    ];
    let members = vec![
        Member2D { id: 0, start_node: 0, end_node: 1, length: config.l1, ea: 1e7, ei: 1e5, label: "AB".to_string() },
        Member2D { id: 1, start_node: 1, end_node: 2, length: config.l2, ea: 1e7, ei: 1e5, label: "BC".to_string() },
    ];
    let supports = vec![
        Support2D { node_id: 0, condition: if config.left_fixed { SupportCondition::Clamped } else { SupportCondition::Pinned } },
        Support2D { node_id: 1, condition: SupportCondition::RollerX },
        Support2D { node_id: 2, condition: if config.right_fixed { SupportCondition::Clamped } else { SupportCondition::RollerX } },
    ];

    Ok(TeachingSolutionDto {
        system_type: "continuous_beam".to_string(),
        is_isostatic: false,
        degree_of_indeterminacy: degree,
        nodes,
        members,
        supports,
        reactions,
        applied_point_loads: vec![
            MemberPointLoad2D {
                member_id: 1,
                distance_from_start: config.l1 + config.l2 / 2.0,
                p: config.p2,
            }
        ],
        applied_dist_loads: vec![
            DistributedLoad2D {
                member_id: 0,
                q_start: config.q1,
                q_end: config.q1,
                direction: "global_y".to_string(),
            },
        ],
        diagrams,
        steps,
        cross_table: Some(cross_table),
    })
}

#[cfg(test)]
#[path = "cross_kani_test.rs"]
mod tests;
