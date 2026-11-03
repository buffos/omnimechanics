use super::types::*;
use crate::frame::types::CanonicalError;

const STANDARD_GRAVITY: f64 = 9.81;

/// Validates SeismicSettings against invariants INV-ACT-05
pub fn validate_seismic_settings(settings: &SeismicSettings) -> Result<(), CanonicalError> {
    if settings.ag_reference_g <= 0.0
        || settings.gamma_i <= 0.0
        || settings.soil_factor_s <= 0.0
        || settings.tb_s <= 0.0
        || settings.tc_s <= settings.tb_s
        || settings.td_s <= settings.tc_s
        || settings.behavior_factor_q < 1.0
        || settings.damping_ratio_percent <= 0.0
        || settings.beta_lower_bound < 0.0
        || settings.beta_lower_bound > 1.0
        || settings.psi_e_phi <= 0.0
    {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_SEISMIC_PARAMETERS",
            "Μη έγκυρες σεισμικές παράμετροι ΕΛΟΤ EN 1998-1 (πρέπει q >= 1.0, 0 < TB < TC < TD, ag > 0, γI > 0).",
            "Invalid EN 1998-1 seismic parameters (must satisfy q >= 1.0, 0 < TB < TC < TD, ag > 0, gamma_I > 0).",
        ));
    }

    if settings.include_vertical_component && settings.vertical_behavior_factor_qv < 1.0 {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_SEISMIC_PARAMETERS",
            "Ο συντελεστής συμπεριφοράς κατακόρυφης συνιστώσας qv πρέπει να είναι >= 1.0.",
            "Vertical behavior factor qv must be >= 1.0.",
        ));
    }

    Ok(())
}

/// Evaluates horizontal EN 1998-1 Type 1 spectrum at period T
pub fn evaluate_horizontal_spectrum_point(
    t: f64,
    ag_g: f64,
    s: f64,
    tb: f64,
    tc: f64,
    td: f64,
    q: f64,
    eta: f64,
    beta: f64,
) -> SpectrumPoint {
    let t_clamped = if t < 0.0 { 0.0 } else { t };

    // Elastic Se(T) / g (EN 1998-1:2004 §3.2.2.2)
    let se_g = if t_clamped <= tb {
        ag_g * s * (1.0 + (t_clamped / tb) * (eta * 2.5 - 1.0))
    } else if t_clamped <= tc {
        ag_g * s * eta * 2.5
    } else if t_clamped <= td {
        ag_g * s * eta * 2.5 * (tc / t_clamped)
    } else {
        ag_g * s * eta * 2.5 * (tc * td / (t_clamped * t_clamped))
    };

    // Design Sd(T) / g (EN 1998-1:2004 §3.2.2.5)
    let sd_g = if t_clamped <= tb {
        ag_g * s * ((2.0 / 3.0) + (t_clamped / tb) * ((2.5 / q) - (2.0 / 3.0)))
    } else if t_clamped <= tc {
        ag_g * s * (2.5 / q)
    } else if t_clamped <= td {
        let val = ag_g * s * (2.5 / q) * (tc / t_clamped);
        let min_val = beta * ag_g;
        if val < min_val { min_val } else { val }
    } else {
        let val = ag_g * s * (2.5 / q) * (tc * td / (t_clamped * t_clamped));
        let min_val = beta * ag_g;
        if val < min_val { min_val } else { val }
    };

    SpectrumPoint {
        period_s: t,
        se_m_s2: se_g * STANDARD_GRAVITY,
        sd_m_s2: sd_g * STANDARD_GRAVITY,
        se_g,
        sd_g,
    }
}

/// Evaluates vertical EN 1998-1 Type 1 spectrum at period T
pub fn evaluate_vertical_spectrum_point(
    t: f64,
    avg_g: f64,
    tb: f64,
    tc: f64,
    td: f64,
    qv: f64,
    eta: f64,
    beta: f64,
) -> SpectrumPoint {
    let t_clamped = if t < 0.0 { 0.0 } else { t };
    let s_v = 1.0;

    let se_g = if t_clamped <= tb {
        avg_g * s_v * (1.0 + (t_clamped / tb) * (eta * 3.0 - 1.0))
    } else if t_clamped <= tc {
        avg_g * s_v * eta * 3.0
    } else if t_clamped <= td {
        avg_g * s_v * eta * 3.0 * (tc / t_clamped)
    } else {
        avg_g * s_v * eta * 3.0 * (tc * td / (t_clamped * t_clamped))
    };

    let sd_g = if t_clamped <= tb {
        avg_g * s_v * ((2.0 / 3.0) + (t_clamped / tb) * ((2.5 / qv) - (2.0 / 3.0)))
    } else if t_clamped <= tc {
        avg_g * s_v * (2.5 / qv)
    } else if t_clamped <= td {
        let val = avg_g * s_v * (2.5 / qv) * (tc / t_clamped);
        let min_val = beta * avg_g;
        if val < min_val { min_val } else { val }
    } else {
        let val = avg_g * s_v * (2.5 / qv) * (tc * td / (t_clamped * t_clamped));
        let min_val = beta * avg_g;
        if val < min_val { min_val } else { val }
    };

    SpectrumPoint {
        period_s: t,
        se_m_s2: se_g * STANDARD_GRAVITY,
        sd_m_s2: sd_g * STANDARD_GRAVITY,
        se_g,
        sd_g,
    }
}

/// Computes the complete Design Spectrum (horizontal + optional vertical)
pub fn compute_design_spectrum(
    request: &ComputeDesignSpectrumRequest,
) -> Result<ComputeDesignSpectrumResponse, CanonicalError> {
    validate_seismic_settings(&request.settings)?;

    let s = &request.settings;
    let ag_g = s.ag_reference_g * s.gamma_i;
    let ag_m_s2 = ag_g * STANDARD_GRAVITY;

    let damping_xi = s.damping_ratio_percent;
    let eta_damping = (10.0 / (5.0 + damping_xi)).sqrt().max(0.55);

    let max_t = request.max_period_s.unwrap_or(4.0);
    if max_t <= 0.0 {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_SPECTRUM_PERIOD",
            "Η μέγιστη περίοδος του φάσματος πρέπει να είναι θετικός αριθμός.",
            "Maximum spectrum period must be a positive number.",
        ));
    }

    let n_pts = request.num_points.unwrap_or(100).max(20);

    // Build discrete evaluation periods including key transition points: 0, tb, tc, td, max_t
    let mut periods = Vec::new();
    for i in 0..=n_pts {
        let t = (i as f64 / n_pts as f64) * max_t;
        periods.push(t);
    }
    // Ensure critical corner periods are present
    for cp in [s.tb_s, s.tc_s, s.td_s] {
        if cp <= max_t && !periods.iter().any(|&p| (p - cp).abs() < 1e-5) {
            periods.push(cp);
        }
    }
    periods.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    periods.dedup_by(|a, b| (*a - *b).abs() < 1e-5);

    let mut horizontal_curve = Vec::new();
    for &t in &periods {
        horizontal_curve.push(evaluate_horizontal_spectrum_point(
            t,
            ag_g,
            s.soil_factor_s,
            s.tb_s,
            s.tc_s,
            s.td_s,
            s.behavior_factor_q,
            eta_damping,
            s.beta_lower_bound,
        ));
    }

    // Optional Vertical Spectrum
    let (vertical_curve, avg_m_s2) = if s.include_vertical_component {
        let avg_g = 0.90 * ag_g;
        let tb_v = 0.05;
        let tc_v = 0.15;
        let td_v = 1.00;
        let qv = s.vertical_behavior_factor_qv;

        let mut v_curve = Vec::new();
        for &t in &periods {
            v_curve.push(evaluate_vertical_spectrum_point(
                t,
                avg_g,
                tb_v,
                tc_v,
                td_v,
                qv,
                eta_damping,
                s.beta_lower_bound,
            ));
        }
        (Some(v_curve), Some(avg_g * STANDARD_GRAVITY))
    } else {
        (None, None)
    };

    // Query specific periods if provided
    let mut queried_ordinates = Vec::new();
    if let Some(ref query_pts) = request.query_periods_s {
        for &t in query_pts {
            if t < 0.0 || !t.is_finite() {
                return Err(CanonicalError::new(
                    "ACTIONS_INVALID_SPECTRUM_PERIOD",
                    "Η ζητούμενη ιδιοπερίοδος T πρέπει να είναι μη αρνητικός πεπερασμένος αριθμός.",
                    "Requested natural period T must be a non-negative finite number.",
                ));
            }
            queried_ordinates.push(evaluate_horizontal_spectrum_point(
                t,
                ag_g,
                s.soil_factor_s,
                s.tb_s,
                s.tc_s,
                s.td_s,
                s.behavior_factor_q,
                eta_damping,
                s.beta_lower_bound,
            ));
        }
    }

    let spectrum = DesignSpectrum {
        ag_m_s2,
        ag_g,
        avg_m_s2,
        eta_damping,
        soil_factor_s: s.soil_factor_s,
        tb_s: s.tb_s,
        tc_s: s.tc_s,
        td_s: s.td_s,
        beta_lower_bound: s.beta_lower_bound,
        horizontal_curve,
        vertical_curve,
        queried_ordinates: queried_ordinates.clone(),
    };

    Ok(ComputeDesignSpectrumResponse {
        spectrum,
        queried_ordinates,
    })
}

/// Computes the seismic mass combination rule (G + psi_E,i * Q)
pub fn compute_seismic_mass_rule(
    request: &ComputeSeismicMassRuleRequest,
) -> Result<SeismicMassCombinationRule, CanonicalError> {
    validate_seismic_settings(&request.settings)?;

    let mut terms = Vec::new();
    let mut parts_el = Vec::new();
    let mut parts_en = Vec::new();

    for lc in &request.load_cases {
        match &lc.action_type {
            ActionType::Permanent => {
                terms.push(SeismicMassTerm {
                    load_case_id: lc.id.clone(),
                    factor: 1.00,
                });
                parts_el.push(format!("1.00·{}", lc.code));
                parts_en.push(format!("1.00·{}", lc.code));
            }
            ActionType::Imposed { category } => {
                let key = format!("Imposed_{}", category.to_uppercase());
                let psi2 = request
                    .profile
                    .psi_factors
                    .get(&key)
                    .map(|p| p.psi_2)
                    .unwrap_or(0.30);

                let psi_e = request.settings.psi_e_phi * psi2;
                let factor = (psi_e * 10_000.0).round() / 10_000.0;
                if factor > 0.0 {
                    terms.push(SeismicMassTerm {
                        load_case_id: lc.id.clone(),
                        factor,
                    });
                    parts_el.push(format!("{:.2}·{}", factor, lc.code));
                    parts_en.push(format!("{:.2}·{}", factor, lc.code));
                }
            }
            ActionType::Snow => {
                let psi2 = request
                    .profile
                    .psi_factors
                    .get("Snow")
                    .map(|p| p.psi_2)
                    .unwrap_or(0.00);
                let psi_e = request.settings.psi_e_phi * psi2;
                let factor = (psi_e * 10_000.0).round() / 10_000.0;
                if factor > 0.0 {
                    terms.push(SeismicMassTerm {
                        load_case_id: lc.id.clone(),
                        factor,
                    });
                    parts_el.push(format!("{:.2}·{}", factor, lc.code));
                    parts_en.push(format!("{:.2}·{}", factor, lc.code));
                }
            }
            ActionType::Wind | ActionType::Temperature => {
                // Wind and temperature have psi_2 = 0 in Eurocode, omitted from seismic mass
                let psi2 = request
                    .profile
                    .psi_factors
                    .get(match lc.action_type {
                        ActionType::Wind => "Wind",
                        _ => "Temperature",
                    })
                    .map(|p| p.psi_2)
                    .unwrap_or(0.00);
                let psi_e = request.settings.psi_e_phi * psi2;
                let factor = (psi_e * 10_000.0).round() / 10_000.0;
                if factor > 0.0 {
                    terms.push(SeismicMassTerm {
                        load_case_id: lc.id.clone(),
                        factor,
                    });
                    parts_el.push(format!("{:.2}·{}", factor, lc.code));
                    parts_en.push(format!("{:.2}·{}", factor, lc.code));
                }
            }
            ActionType::Seismic | ActionType::Accidental => {
                // Seismic and accidental actions do not contribute to gravitational mass
            }
        }
    }

    let formula_el = format!("Σ Gk,j + Σ ψE,i·Qk,i [{}]", parts_el.join(" + "));
    let formula_en = format!("Σ Gk,j + Σ psi_E,i·Qk,i [{}]", parts_en.join(" + "));

    Ok(SeismicMassCombinationRule {
        terms,
        symbolic_formula: BilingualText::new(formula_el, formula_en),
    })
}

/// Computes the EN 1998-1 §4.3.3.2 lateral force distribution
pub fn compute_lateral_forces(
    request: &ComputeLateralForcesRequest,
) -> Result<LateralForceDistributionResult, CanonicalError> {
    validate_seismic_settings(&request.settings)?;

    let t1 = request.fundamental_period_t1_s;
    if t1 <= 0.0 || !t1.is_finite() {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_LATERAL_FORCE_INPUT",
            "Η θεμελιώδης ιδιοπερίοδος T1 πρέπει να είναι θετικός πεπερασμένος αριθμός.",
            "Fundamental period T1 must be a positive finite number.",
        ));
    }

    if request.levels.is_empty() {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_LATERAL_FORCE_INPUT",
            "Απαιτείται τουλάχιστον ένα επίπεδο/όροφος για τον υπολογισμό των σεισμικών δυνάμεων.",
            "At least one level/storey is required for lateral force computation.",
        ));
    }

    let total_mass: f64 = request.levels.iter().map(|l| l.mass_tonnes).sum();
    if total_mass <= 0.0 || !total_mass.is_finite() {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_LATERAL_FORCE_INPUT",
            "Η συνολική σεισμική μάζα πρέπει να είναι θετική.",
            "Total seismic mass must be positive.",
        ));
    }

    let sum_zm: f64 = request.levels.iter().map(|l| l.elevation_z_m * l.mass_tonnes).sum();
    if sum_zm <= 0.0 || !sum_zm.is_finite() {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_LATERAL_FORCE_INPUT",
            "Το γινόμενο Σ(zi·mi) πρέπει να είναι θετικό.",
            "Sum of products Sigma(zi * mi) must be positive.",
        ));
    }

    // EN 1998-1 §4.3.3.2 (2)P: lambda = 0.85 if T1 <= 2*TC and building has more than 2 storeys
    let lambda = if t1 <= 2.0 * request.settings.tc_s && request.n_storeys > 2 {
        0.85
    } else {
        1.00
    };

    let s = &request.settings;
    let ag_g = s.ag_reference_g * s.gamma_i;
    let damping_xi = s.damping_ratio_percent;
    let eta_damping = (10.0 / (5.0 + damping_xi)).sqrt().max(0.55);

    let sd_pt = evaluate_horizontal_spectrum_point(
        t1,
        ag_g,
        s.soil_factor_s,
        s.tb_s,
        s.tc_s,
        s.td_s,
        s.behavior_factor_q,
        eta_damping,
        s.beta_lower_bound,
    );

    // Fb = Sd(T1) * m * lambda
    // Mass is in tonnes (1000 kg), Sd is in m/s2 => Fb in kN
    let base_shear_kn = sd_pt.sd_m_s2 * total_mass * lambda;

    let mut story_forces = Vec::new();
    for l in &request.levels {
        let zm = l.elevation_z_m * l.mass_tonnes;
        let force_kn = base_shear_kn * (zm / sum_zm);
        story_forces.push(LateralStoryForce {
            level_name: l.level_name.clone(),
            elevation_z_m: l.elevation_z_m,
            mass_tonnes: l.mass_tonnes,
            force_kn,
        });
    }

    Ok(LateralForceDistributionResult {
        fundamental_period_t1_s: t1,
        lambda_correction: lambda,
        sd_t1_m_s2: sd_pt.sd_m_s2,
        total_mass_tonnes: total_mass,
        base_shear_kn,
        story_forces,
    })
}

/// Computes the CQC (Der Kiureghian) or SRSS modal correlation matrix
pub fn compute_cqc_matrix(
    request: &ComputeCqcMatrixRequest,
) -> Result<ModalCorrelationMatrix, CanonicalError> {
    let n = request.modal_periods_s.len();
    if n == 0 {
        return Ok(ModalCorrelationMatrix {
            rule: request.rule.clone(),
            modal_periods_s: Vec::new(),
            matrix: Vec::new(),
        });
    }

    for &t in &request.modal_periods_s {
        if t <= 0.0 || !t.is_finite() {
            return Err(CanonicalError::new(
                "ACTIONS_INVALID_SPECTRUM_PERIOD",
                "Οι ιδιοπερίοδοι T πρέπει να είναι θετικοί πεπερασμένοι αριθμοί.",
                "Modal periods T must be positive finite numbers.",
            ));
        }
    }

    if request.damping_ratio_percent <= 0.0 || !request.damping_ratio_percent.is_finite() {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_SEISMIC_PARAMETERS",
            "Ο λόγος απόσβεσης ζ πρέπει να είναι θετικός πεπερασμένος αριθμός.",
            "Damping ratio zeta must be a positive finite number.",
        ));
    }

    let is_srss = request.rule.to_uppercase() == "SRSS";
    let zeta = request.damping_ratio_percent / 100.0; // convert percentage to fraction e.g. 5% -> 0.05
    let zeta_sq = zeta * zeta;

    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix[i][j] = 1.0;
            } else if is_srss {
                matrix[i][j] = 0.0;
            } else {
                // Der Kiureghian CQC coefficient
                // r = w_j / w_i = T_i / T_j
                let ti = request.modal_periods_s[i];
                let tj = request.modal_periods_s[j];
                let r = ti / tj;
                let r_sq = r * r;
                let r_1_5 = r.powf(1.5);

                let numerator = 8.0 * zeta_sq * (1.0 + r) * r_1_5;
                let term1 = (1.0 - r_sq).powi(2);
                let term2 = 4.0 * zeta_sq * r * (1.0 + r).powi(2);
                let denom = term1 + term2;

                let rho = if denom > 1e-12 {
                    numerator / denom
                } else {
                    1.0
                };

                matrix[i][j] = rho;
            }
        }
    }

    Ok(ModalCorrelationMatrix {
        rule: if is_srss { "SRSS".to_string() } else { "CQC".to_string() },
        modal_periods_s: request.modal_periods_s.clone(),
        matrix,
    })
}
