use std::collections::{HashMap, HashSet};

use super::na_profile::validate_na_profile;
use super::types::*;
use crate::frame::types::CanonicalError;

fn get_psi_factors_for_action(action_type: &ActionType, profile: &NationalAnnexProfile) -> PsiFactors {
    let key = match action_type {
        ActionType::Permanent => return PsiFactors { psi_0: 1.0, psi_1: 1.0, psi_2: 1.0 },
        ActionType::Imposed { category } => format!("Imposed_{}", category.to_uppercase()),
        ActionType::Snow => "Snow".to_string(),
        ActionType::Wind => "Wind".to_string(),
        ActionType::Temperature => "Temperature".to_string(),
        ActionType::Seismic => return PsiFactors { psi_0: 0.0, psi_1: 0.0, psi_2: 0.0 },
        ActionType::Accidental => return PsiFactors { psi_0: 0.0, psi_1: 0.0, psi_2: 0.0 },
    };

    profile.psi_factors.get(&key).copied().unwrap_or(PsiFactors {
        psi_0: 0.70,
        psi_1: 0.50,
        psi_2: 0.30,
    })
}

fn build_symbolic_formula(
    terms: &[CombinationTerm],
    lc_map: &HashMap<&str, &LoadCaseDescriptor>,
    eq_str: &str,
) -> BilingualText {
    let mut parts_el = Vec::new();
    let mut parts_en = Vec::new();

    for (idx, t) in terms.iter().enumerate() {
        let code = lc_map.get(t.load_case_id.as_str()).map(|lc| lc.code.as_str()).unwrap_or("?");
        if idx == 0 {
            let str_val = format!("{:.2}·{}", t.factor, code);
            parts_el.push(str_val.clone());
            parts_en.push(str_val);
        } else if t.factor < 0.0 {
            let str_val = format!("- {:.2}·{}", t.factor.abs(), code);
            parts_el.push(str_val.clone());
            parts_en.push(str_val);
        } else {
            let str_val = format!("+ {:.2}·{}", t.factor, code);
            parts_el.push(str_val.clone());
            parts_en.push(str_val);
        }
    }

    let formula_el = if !eq_str.is_empty() {
        format!("{} ({})", parts_el.join(" "), eq_str)
    } else {
        parts_el.join(" ")
    };

    let eq_str_en = match eq_str {
        "Εξ. 6.10" => "Eq. 6.10",
        "Εξ. 6.10a" => "Eq. 6.10a",
        "Εξ. 6.10b" => "Eq. 6.10b",
        "Εξ. 6.12b" => "Eq. 6.12b",
        "Εξ. 6.14b" => "Eq. 6.14b",
        "Εξ. 6.15b" => "Eq. 6.15b",
        "Εξ. 6.16b" => "Eq. 6.16b",
        _ => eq_str,
    };

    let formula_en = if !eq_str_en.is_empty() {
        format!("{} ({})", parts_en.join(" "), eq_str_en)
    } else {
        parts_en.join(" ")
    };

    BilingualText::new(formula_el, formula_en)
}

/// Generates normative EN 1990 ULS & SLS combinations
pub fn generate_combinations(
    request: &GenerateCombinationsRequest,
) -> Result<CombinationSet, CanonicalError> {
    validate_na_profile(&request.profile)?;

    // Check duplicate load cases
    let mut seen_ids = HashSet::new();
    for lc in &request.load_cases {
        if !seen_ids.insert(lc.id.as_str()) {
            return Err(CanonicalError::new(
                "ACTIONS_DUPLICATE_LOAD_CASE_ID",
                format!("Διπλότυπο αναγνωριστικό περίπτωσης φόρτισης: '{}'.", lc.id),
                format!("Duplicate load case ID: '{}'.", lc.id),
            ));
        }
    }

    let mut permanent_cases = Vec::new();
    let mut variable_cases = Vec::new();

    for lc in &request.load_cases {
        match lc.action_type {
            ActionType::Permanent => permanent_cases.push(lc),
            ActionType::Accidental => {
                return Err(CanonicalError::new(
                    "ACTIONS_ACCIDENTAL_NOT_SUPPORTED",
                    "Οι τυχαίοι συνδυασμοί (Accidental) δεν υποστηρίζονται στην παρούσα έκδοση.",
                    "Accidental combinations are not supported in this version.",
                ));
            }
            ActionType::Seismic => {}
            _ => variable_cases.push(lc),
        }
    }

    if permanent_cases.is_empty() {
        return Err(CanonicalError::new(
            "ACTIONS_NO_PERMANENT_LOAD_CASE",
            "Απαιτείται τουλάχιστον μία Μόνιμη (G) φόρτιση για την παραγωγή συνδυασμών.",
            "At least one Permanent (G) load case is required for combination generation.",
        ));
    }

    if request.profile.permanent_favorability_policy == PermanentFavorabilityPolicy::PerCasePermutation
        && permanent_cases.len() > 6
    {
        return Err(CanonicalError::new(
            "ACTIONS_TOO_MANY_PERMANENT_CASES",
            "Η συνδυαστική ανά μόνιμη φόρτιση επιτρέπεται για έως 6 μόνιμες φορτίσεις.",
            "Per-case permanent permutation is limited to at most 6 permanent load cases.",
        ));
    }

    let lc_map: HashMap<&str, &LoadCaseDescriptor> =
        request.load_cases.iter().map(|lc| (lc.id.as_str(), lc)).collect();

    // Map existing combinations by generation signature
    let mut existing_by_sig: HashMap<&str, &Combination> = HashMap::new();
    let mut manual_combinations: Vec<Combination> = Vec::new();

    for comb in &request.existing_combinations {
        // Only retain if all referenced load cases still exist
        let all_exist = comb.terms.iter().all(|t| lc_map.contains_key(t.load_case_id.as_str()));
        if all_exist {
            if comb.origin == CombinationOrigin::Manual {
                manual_combinations.push(comb.clone());
            } else if let Some(ref sig) = comb.generation_signature {
                existing_by_sig.insert(sig.as_str(), comb);
            }
        }
    }

    let mut generated_combinations: Vec<Combination> = Vec::new();
    let mut id_counter = 1usize;

    // Helper to add generated combination with provenance preservation
    let mut add_comb = |name_el: String,
                        name_en: String,
                        limit_state: LimitState,
                        equation: CombinationEquation,
                        leading_lc_id: Option<String>,
                        terms: Vec<CombinationTerm>,
                        signature: String,
                        eq_str: &str| {
        let terms: Vec<CombinationTerm> = terms
            .into_iter()
            .map(|mut t| {
                t.factor = (t.factor * 10_000.0).round() / 10_000.0;
                t
            })
            .collect();

        let formula = build_symbolic_formula(&terms, &lc_map, eq_str);

        // Check if an existing UserEdited combination matches this slot
        if !request.reset_user_edited {
            if let Some(existing) = existing_by_sig.get(signature.as_str()) {
                if existing.origin == CombinationOrigin::UserEdited {
                    generated_combinations.push((*existing).clone());
                    return;
                }
            }
        }

        let id = format!("CO{}", id_counter);
        id_counter += 1;

        generated_combinations.push(Combination {
            id,
            name: BilingualText::new(name_el, name_en),
            limit_state,
            origin: CombinationOrigin::Generated,
            equation: Some(equation),
            leading_load_case_id: leading_lc_id,
            terms,
            symbolic_formula: formula,
            generation_signature: Some(signature),
        });
    };

    let p = &request.profile;
    let perm_branches = match p.permanent_favorability_policy {
        PermanentFavorabilityPolicy::UnfavorableOnly => vec![(p.gamma_g_sup, "G_SUP", true)],
        _ => vec![(p.gamma_g_sup, "G_SUP", true), (p.gamma_g_inf, "G_INF", false)],
    };

    // 1. ULS STR / GEO Combinations
    if variable_cases.is_empty() {
        // Only permanent cases exist
        for &(gamma_g, g_tag, is_unfav) in &perm_branches {
            let role = if is_unfav {
                CombinationRole::PermanentUnfavorable
            } else {
                CombinationRole::PermanentFavorable
            };
            let terms: Vec<CombinationTerm> = permanent_cases
                .iter()
                .map(|g| CombinationTerm {
                    load_case_id: g.id.clone(),
                    factor: gamma_g,
                    role,
                })
                .collect();
            let name_suffix = if is_unfav { "(Δυσμενής G)" } else { "(Ευμενής G)" };
            let name_suffix_en = if is_unfav { "(Unfavorable G)" } else { "(Favorable G)" };
            let sig = format!("ULS_STR:Eq6_10:{}:BASE", g_tag);

            add_comb(
                format!("ΟΚΑ STR - Μόνιμα {}", name_suffix),
                format!("ULS STR - Permanent {}", name_suffix_en),
                LimitState::ULS_STR,
                CombinationEquation::Eq6_10,
                None,
                terms,
                sig,
                "Εξ. 6.10",
            );
        }
    } else {
        match p.uls_equation_policy {
            UlsEquationPolicy::Single6_10 => {
                // For each variable action as LeadingVariable
                for lead_var in &variable_cases {
                    for &(gamma_g, g_tag, is_unfav) in &perm_branches {
                        let perm_role = if is_unfav {
                            CombinationRole::PermanentUnfavorable
                        } else {
                            CombinationRole::PermanentFavorable
                        };
                        let mut terms = Vec::new();
                        for g in &permanent_cases {
                            terms.push(CombinationTerm {
                                load_case_id: g.id.clone(),
                                factor: gamma_g,
                                role: perm_role,
                            });
                        }
                        terms.push(CombinationTerm {
                            load_case_id: lead_var.id.clone(),
                            factor: p.gamma_q_sup,
                            role: CombinationRole::LeadingVariable,
                        });
                        for acc_var in &variable_cases {
                            if acc_var.id != lead_var.id {
                                let psi = get_psi_factors_for_action(&acc_var.action_type, p);
                                let f = p.gamma_q_sup * psi.psi_0;
                                if f > 1e-6 {
                                    terms.push(CombinationTerm {
                                        load_case_id: acc_var.id.clone(),
                                        factor: f,
                                        role: CombinationRole::AccompanyingVariable,
                                    });
                                }
                            }
                        }

                        let name_suffix = if is_unfav { "(Δυσμενής G)" } else { "(Ευμενής G)" };
                        let name_suffix_en = if is_unfav { "(Unfavorable G)" } else { "(Favorable G)" };
                        let sig = format!("ULS_STR:Eq6_10:{}:LEAD_{}", g_tag, lead_var.id);

                        add_comb(
                            format!("ΟΚΑ STR - Κύρια {} {}", lead_var.code, name_suffix),
                            format!("ULS STR - Leading {} {}", lead_var.code, name_suffix_en),
                            LimitState::ULS_STR,
                            CombinationEquation::Eq6_10,
                            Some(lead_var.id.clone()),
                            terms,
                            sig,
                            "Εξ. 6.10",
                        );
                    }
                }
            }
            UlsEquationPolicy::Twin6_10a_6_10b => {
                // Equation 6.10a: gamma_G * G + sum(gamma_Q * psi_0 * Q_i)
                for &(gamma_g, g_tag, is_unfav) in &perm_branches {
                    let perm_role = if is_unfav {
                        CombinationRole::PermanentUnfavorable
                    } else {
                        CombinationRole::PermanentFavorable
                    };
                    let mut terms = Vec::new();
                    for g in &permanent_cases {
                        terms.push(CombinationTerm {
                            load_case_id: g.id.clone(),
                            factor: gamma_g,
                            role: perm_role,
                        });
                    }
                    for q in &variable_cases {
                        let psi = get_psi_factors_for_action(&q.action_type, p);
                        let f = p.gamma_q_sup * psi.psi_0;
                        if f > 1e-6 {
                            terms.push(CombinationTerm {
                                load_case_id: q.id.clone(),
                                factor: f,
                                role: CombinationRole::AccompanyingVariable,
                            });
                        }
                    }
                    let name_suffix = if is_unfav { "(Δυσμενής G)" } else { "(Ευμενής G)" };
                    let name_suffix_en = if is_unfav { "(Unfavorable G)" } else { "(Favorable G)" };
                    let sig = format!("ULS_STR:Eq6_10a:{}", g_tag);

                    add_comb(
                        format!("ΟΚΑ STR - Εξ. 6.10a {}", name_suffix),
                        format!("ULS STR - Eq. 6.10a {}", name_suffix_en),
                        LimitState::ULS_STR,
                        CombinationEquation::Eq6_10a,
                        None,
                        terms,
                        sig,
                        "Εξ. 6.10a",
                    );
                }

                // Equation 6.10b: xi * gamma_G * G + gamma_Q * Q_lead + sum(gamma_Q * psi_0 * Q_j)
                for lead_var in &variable_cases {
                    for &(gamma_g, g_tag, is_unfav) in &perm_branches {
                        let perm_role = if is_unfav {
                            CombinationRole::PermanentUnfavorable
                        } else {
                            CombinationRole::PermanentFavorable
                        };
                        let eff_g = if is_unfav {
                            p.xi_reduction * gamma_g
                        } else {
                            gamma_g
                        };
                        let mut terms = Vec::new();
                        for g in &permanent_cases {
                            terms.push(CombinationTerm {
                                load_case_id: g.id.clone(),
                                factor: eff_g,
                                role: perm_role,
                            });
                        }
                        terms.push(CombinationTerm {
                            load_case_id: lead_var.id.clone(),
                            factor: p.gamma_q_sup,
                            role: CombinationRole::LeadingVariable,
                        });
                        for acc_var in &variable_cases {
                            if acc_var.id != lead_var.id {
                                let psi = get_psi_factors_for_action(&acc_var.action_type, p);
                                let f = p.gamma_q_sup * psi.psi_0;
                                if f > 1e-6 {
                                    terms.push(CombinationTerm {
                                        load_case_id: acc_var.id.clone(),
                                        factor: f,
                                        role: CombinationRole::AccompanyingVariable,
                                    });
                                }
                            }
                        }
                        let name_suffix = if is_unfav { "(Δυσμενής G)" } else { "(Ευμενής G)" };
                        let name_suffix_en = if is_unfav { "(Unfavorable G)" } else { "(Favorable G)" };
                        let sig = format!("ULS_STR:Eq6_10b:{}:LEAD_{}", g_tag, lead_var.id);

                        add_comb(
                            format!("ΟΚΑ STR - Εξ. 6.10b (Κύρια {}) {}", lead_var.code, name_suffix),
                            format!("ULS STR - Eq. 6.10b (Leading {}) {}", lead_var.code, name_suffix_en),
                            LimitState::ULS_STR,
                            CombinationEquation::Eq6_10b,
                            Some(lead_var.id.clone()),
                            terms,
                            sig,
                            "Εξ. 6.10b",
                        );
                    }
                }
            }
        }
    }

    // 2. SLS Characteristic Combinations (Eq. 6.14b): 1.0 G + 1.0 Q_lead + sum(psi_0 * Q_j)
    if !variable_cases.is_empty() {
        for lead_var in &variable_cases {
            let mut terms = Vec::new();
            for g in &permanent_cases {
                terms.push(CombinationTerm {
                    load_case_id: g.id.clone(),
                    factor: 1.0,
                    role: CombinationRole::PermanentUnfavorable,
                });
            }
            terms.push(CombinationTerm {
                load_case_id: lead_var.id.clone(),
                factor: 1.0,
                role: CombinationRole::LeadingVariable,
            });
            for acc_var in &variable_cases {
                if acc_var.id != lead_var.id {
                    let psi = get_psi_factors_for_action(&acc_var.action_type, p);
                    if psi.psi_0 > 1e-6 {
                        terms.push(CombinationTerm {
                            load_case_id: acc_var.id.clone(),
                            factor: psi.psi_0,
                            role: CombinationRole::AccompanyingVariable,
                        });
                    }
                }
            }
            let sig = format!("SLS_CHAR:Eq6_14b:LEAD_{}", lead_var.id);
            add_comb(
                format!("ΟΚΛ Χαρακτηριστικός - Κύρια {}", lead_var.code),
                format!("SLS Characteristic - Leading {}", lead_var.code),
                LimitState::SLS_CHAR,
                CombinationEquation::Eq6_14b,
                Some(lead_var.id.clone()),
                terms,
                sig,
                "Εξ. 6.14b",
            );
        }
    }

    // 3. SLS Frequent Combinations (Eq. 6.15b): 1.0 G + psi_1 * Q_lead + sum(psi_2 * Q_j)
    if !variable_cases.is_empty() {
        for lead_var in &variable_cases {
            let psi_lead = get_psi_factors_for_action(&lead_var.action_type, p);
            let mut terms = Vec::new();
            for g in &permanent_cases {
                terms.push(CombinationTerm {
                    load_case_id: g.id.clone(),
                    factor: 1.0,
                    role: CombinationRole::PermanentUnfavorable,
                });
            }
            if psi_lead.psi_1 > 1e-6 {
                terms.push(CombinationTerm {
                    load_case_id: lead_var.id.clone(),
                    factor: psi_lead.psi_1,
                    role: CombinationRole::LeadingVariable,
                });
            }
            for acc_var in &variable_cases {
                if acc_var.id != lead_var.id {
                    let psi = get_psi_factors_for_action(&acc_var.action_type, p);
                    if psi.psi_2 > 1e-6 {
                        terms.push(CombinationTerm {
                            load_case_id: acc_var.id.clone(),
                            factor: psi.psi_2,
                            role: CombinationRole::AccompanyingVariable,
                        });
                    }
                }
            }
            let sig = format!("SLS_FREQ:Eq6_15b:LEAD_{}", lead_var.id);
            add_comb(
                format!("ΟΚΛ Συχνός - Κύρια {}", lead_var.code),
                format!("SLS Frequent - Leading {}", lead_var.code),
                LimitState::SLS_FREQ,
                CombinationEquation::Eq6_15b,
                Some(lead_var.id.clone()),
                terms,
                sig,
                "Εξ. 6.15b",
            );
        }
    }

    // 4. SLS Quasi-Permanent (Eq. 6.16b): 1.0 G + sum(psi_2 * Q_i)
    {
        let mut terms = Vec::new();
        for g in &permanent_cases {
            terms.push(CombinationTerm {
                load_case_id: g.id.clone(),
                factor: 1.0,
                role: CombinationRole::PermanentUnfavorable,
            });
        }
        for q in &variable_cases {
            let psi = get_psi_factors_for_action(&q.action_type, p);
            if psi.psi_2 > 1e-6 {
                terms.push(CombinationTerm {
                    load_case_id: q.id.clone(),
                    factor: psi.psi_2,
                    role: CombinationRole::AccompanyingVariable,
                });
            }
        }
        let sig = "SLS_QP:Eq6_16b".to_string();
        add_comb(
            "ΟΚΛ Οιονεί Μόνιμος".to_string(),
            "SLS Quasi-Permanent".to_string(),
            LimitState::SLS_QP,
            CombinationEquation::Eq6_16b,
            None,
            terms,
            sig,
            "Εξ. 6.16b",
        );
    }

    // 5. SEISMIC Combinations (Eq. 6.12b): 1.0 G + sum(psi_2 * Q_i) ± E
    let seismic_cases: Vec<&LoadCaseDescriptor> = request
        .load_cases
        .iter()
        .filter(|lc| matches!(lc.action_type, ActionType::Seismic))
        .collect();

    if !seismic_cases.is_empty() {
        let mut base_gravity_terms = Vec::new();
        for g in &permanent_cases {
            base_gravity_terms.push(CombinationTerm {
                load_case_id: g.id.clone(),
                factor: 1.0,
                role: CombinationRole::PermanentUnfavorable,
            });
        }
        for q in &variable_cases {
            let psi = get_psi_factors_for_action(&q.action_type, p);
            if psi.psi_2 > 1e-6 {
                base_gravity_terms.push(CombinationTerm {
                    load_case_id: q.id.clone(),
                    factor: psi.psi_2,
                    role: CombinationRole::AccompanyingVariable,
                });
            }
        }

        let include_vertical = request
            .seismic_settings
            .as_ref()
            .map(|s| s.include_vertical_component)
            .unwrap_or(false);

        if seismic_cases.len() == 1 || !include_vertical {
            // Horizontal only: +1.00 E and -1.00 E for each seismic case
            for sc in &seismic_cases {
                for &sign in &[1.0, -1.0] {
                    let mut terms = base_gravity_terms.clone();
                    terms.push(CombinationTerm {
                        load_case_id: sc.id.clone(),
                        factor: sign,
                        role: CombinationRole::SeismicPrimary,
                    });
                    let sign_str = if sign > 0.0 { "+E" } else { "-E" };
                    let sign_label = if sign > 0.0 { "(+)" } else { "(-)" };
                    let sig = format!("SEISMIC:Eq6_12b:{}:{}", sc.id, sign_str);
                    add_comb(
                        format!("Σεισμικός Συνδυασμός {} {}", sc.code, sign_label),
                        format!("Seismic Combination {} {}", sc.code, sign_label),
                        LimitState::SEISMIC,
                        CombinationEquation::Eq6_12b,
                        Some(sc.id.clone()),
                        terms,
                        sig,
                        "Εξ. 6.12b",
                    );
                }
            }
        } else if seismic_cases.len() >= 2 && include_vertical {
            // 8 signed combinations using 30% rule:
            // ± 1.00 E1 ± 0.30 E2  (4 combinations)
            // ± 0.30 E1 ± 1.00 E2  (4 combinations)
            let sc1 = seismic_cases[0];
            let sc2 = seismic_cases[1];
            let patterns = [
                (1.00, 0.30, "+E1+0.3E2"),
                (1.00, -0.30, "+E1-0.3E2"),
                (-1.00, 0.30, "-E1+0.3E2"),
                (-1.00, -0.30, "-E1-0.3E2"),
                (0.30, 1.00, "+0.3E1+E2"),
                (0.30, -1.00, "+0.3E1-E2"),
                (-0.30, 1.00, "-0.3E1+E2"),
                (-0.30, -1.00, "-0.3E1-E2"),
            ];

            for (f1, f2, tag) in patterns {
                let mut terms = base_gravity_terms.clone();
                terms.push(CombinationTerm {
                    load_case_id: sc1.id.clone(),
                    factor: f1,
                    role: if f1.abs() > 0.5 {
                        CombinationRole::SeismicPrimary
                    } else {
                        CombinationRole::SeismicAccompanying
                    },
                });
                terms.push(CombinationTerm {
                    load_case_id: sc2.id.clone(),
                    factor: f2,
                    role: if f2.abs() > 0.5 {
                        CombinationRole::SeismicPrimary
                    } else {
                        CombinationRole::SeismicAccompanying
                    },
                });

                let sig = format!("SEISMIC:Eq6_12b:{}:{}:{}", sc1.id, sc2.id, tag);
                add_comb(
                    format!("Σεισμικός Συνδυασμός {}", tag),
                    format!("Seismic Combination {}", tag),
                    LimitState::SEISMIC,
                    CombinationEquation::Eq6_12b,
                    Some(sc1.id.clone()),
                    terms,
                    sig,
                    "Εξ. 6.12b",
                );
            }
        }
    }

    // Append manual combinations
    for mut mc in manual_combinations {
        // Ensure unique ID if conflict
        if generated_combinations.iter().any(|c| c.id == mc.id) {
            mc.id = format!("CO{}", id_counter);
            id_counter += 1;
        }
        generated_combinations.push(mc);
    }

    // Build Envelopes
    let uls_str_ids: Vec<String> = generated_combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::ULS_STR)
        .map(|c| c.id.clone())
        .collect();

    let sls_char_ids: Vec<String> = generated_combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SLS_CHAR)
        .map(|c| c.id.clone())
        .collect();

    let sls_freq_ids: Vec<String> = generated_combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SLS_FREQ)
        .map(|c| c.id.clone())
        .collect();

    let sls_qp_ids: Vec<String> = generated_combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SLS_QP)
        .map(|c| c.id.clone())
        .collect();

    let seismic_ids: Vec<String> = generated_combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SEISMIC)
        .map(|c| c.id.clone())
        .collect();

    let mut envelopes = Vec::new();
    if !uls_str_ids.is_empty() {
        envelopes.push(EnvelopeSpecification {
            id: "ENV_ULS_STR".to_string(),
            name: BilingualText::new("Περιβάλλουσα ΟΚΑ (STR)", "ULS (STR) Envelope"),
            limit_state: LimitState::ULS_STR,
            combination_ids: uls_str_ids,
        });
    }
    if !sls_char_ids.is_empty() {
        envelopes.push(EnvelopeSpecification {
            id: "ENV_SLS_CHAR".to_string(),
            name: BilingualText::new("Περιβάλλουσα ΟΚΛ (Χαρακτηριστική)", "SLS (Characteristic) Envelope"),
            limit_state: LimitState::SLS_CHAR,
            combination_ids: sls_char_ids,
        });
    }
    if !sls_freq_ids.is_empty() {
        envelopes.push(EnvelopeSpecification {
            id: "ENV_SLS_FREQ".to_string(),
            name: BilingualText::new("Περιβάλλουσα ΟΚΛ (Συχνή)", "SLS (Frequent) Envelope"),
            limit_state: LimitState::SLS_FREQ,
            combination_ids: sls_freq_ids,
        });
    }
    if !sls_qp_ids.is_empty() {
        envelopes.push(EnvelopeSpecification {
            id: "ENV_SLS_QP".to_string(),
            name: BilingualText::new("Περιβάλλουσα ΟΚΛ (Οιονεί Μόνιμη)", "SLS (Quasi-Permanent) Envelope"),
            limit_state: LimitState::SLS_QP,
            combination_ids: sls_qp_ids,
        });
    }
    if !seismic_ids.is_empty() {
        envelopes.push(EnvelopeSpecification {
            id: "ENV_SEISMIC".to_string(),
            name: BilingualText::new("Περιβάλλουσα Σεισμικών", "Seismic Envelope"),
            limit_state: LimitState::SEISMIC,
            combination_ids: seismic_ids,
        });
    }

    Ok(CombinationSet {
        revision: 1,
        combinations: generated_combinations,
        envelopes,
        warnings: Vec::new(),
    })
}

/// Saves or updates a custom combination
pub fn save_custom_combination(
    request: &SaveCustomCombinationRequest,
) -> Result<CombinationSet, CanonicalError> {
    let draft = &request.combination;

    // Invariant: Non-empty terms with finite non-zero factor
    let mut valid_terms = Vec::new();
    for t in &draft.terms {
        if !t.factor.is_finite() {
            return Err(CanonicalError::new(
                "ACTIONS_INVALID_COMBINATION_FACTOR",
                "Ο συντελεστής φόρτισης πρέπει να είναι πεπερασμένος αριθμός.",
                "Load factor must be a finite number.",
            ));
        }
        if t.factor.abs() > 1e-9 {
            valid_terms.push(t.clone());
        }
    }

    if valid_terms.is_empty() {
        return Err(CanonicalError::new(
            "ACTIONS_EMPTY_COMBINATION_TERMS",
            "Ο συνδυασμός πρέπει να περιέχει τουλάχιστον έναν μη μηδενικό όρο φόρτισης.",
            "Combination must contain at least one non-zero load case factor.",
        ));
    }

    // Invariant: Load case references must exist
    let lc_ids: HashSet<&str> = request.load_cases.iter().map(|lc| lc.id.as_str()).collect();
    for t in &valid_terms {
        if !lc_ids.contains(t.load_case_id.as_str()) {
            return Err(CanonicalError::new(
                "ACTIONS_UNKNOWN_LOAD_CASE_REF",
                format!("Ο συνδυασμός αναφέρεται σε άγνωστη περίπτωση φόρτισης '{}'.", t.load_case_id),
                format!("Combination references unknown load case ID '{}'.", t.load_case_id),
            ));
        }
    }

    let lc_map: HashMap<&str, &LoadCaseDescriptor> =
        request.load_cases.iter().map(|lc| (lc.id.as_str(), lc)).collect();
    let formula = build_symbolic_formula(&valid_terms, &lc_map, "");

    let mut combinations = request.existing_combinations.clone();

    if let Some(ref edit_id) = draft.id {
        if let Some(target) = combinations.iter_mut().find(|c| c.id == *edit_id) {
            target.name = draft.name.clone();
            target.limit_state = draft.limit_state;
            target.terms = valid_terms;
            target.symbolic_formula = formula;
            target.origin = if target.origin == CombinationOrigin::Manual {
                CombinationOrigin::Manual
            } else {
                CombinationOrigin::UserEdited
            };
        } else {
            return Err(CanonicalError::new(
                "ACTIONS_COMBINATION_NOT_FOUND",
                format!("Δεν βρέθηκε ο συνδυασμός με αναγνωριστικό '{}'.", edit_id),
                format!("Combination ID '{}' not found in existing combinations.", edit_id),
            ));
        }
    } else {
        // Create new manual combination
        let mut max_idx = 0usize;
        for c in &combinations {
            if c.id.starts_with("CO") {
                if let Ok(idx) = c.id[2..].parse::<usize>() {
                    max_idx = max_idx.max(idx);
                }
            }
        }
        let new_id = format!("CO{}", max_idx + 1);

        combinations.push(Combination {
            id: new_id,
            name: draft.name.clone(),
            limit_state: draft.limit_state,
            origin: CombinationOrigin::Manual,
            equation: Some(CombinationEquation::Custom),
            leading_load_case_id: None,
            terms: valid_terms,
            symbolic_formula: formula,
            generation_signature: None,
        });
    }

    // Recompute envelopes
    let mut envelopes = Vec::new();
    for ls in [LimitState::ULS_STR, LimitState::SLS_CHAR, LimitState::SLS_FREQ, LimitState::SLS_QP, LimitState::SEISMIC] {
        let ids: Vec<String> = combinations.iter().filter(|c| c.limit_state == ls).map(|c| c.id.clone()).collect();
        if !ids.is_empty() {
            let (id_str, el, en) = match ls {
                LimitState::ULS_STR => ("ENV_ULS_STR", "Περιβάλλουσα ΟΚΑ (STR)", "ULS (STR) Envelope"),
                LimitState::SLS_CHAR => ("ENV_SLS_CHAR", "Περιβάλλουσα ΟΚΛ (Χαρακτηριστική)", "SLS (Characteristic) Envelope"),
                LimitState::SLS_FREQ => ("ENV_SLS_FREQ", "Περιβάλλουσα ΟΚΛ (Συχνή)", "SLS (Frequent) Envelope"),
                LimitState::SLS_QP => ("ENV_SLS_QP", "Περιβάλλουσα ΟΚΛ (Οιονεί Μόνιμη)", "SLS (Quasi-Permanent) Envelope"),
                LimitState::SEISMIC => ("ENV_SEISMIC", "Περιβάλλουσα Σεισμικών", "Seismic Envelope"),
                _ => ("ENV_OTHER", "Περιβάλλουσα", "Envelope"),
            };
            envelopes.push(EnvelopeSpecification {
                id: id_str.to_string(),
                name: BilingualText::new(el, en),
                limit_state: ls,
                combination_ids: ids,
            });
        }
    }

    Ok(CombinationSet {
        revision: 1,
        combinations,
        envelopes,
        warnings: Vec::new(),
    })
}

/// Deletes or resets a combination to its generated normative state
pub fn delete_or_reset_combination(
    request: &DeleteOrResetCombinationRequest,
) -> Result<CombinationSet, CanonicalError> {
    let mut combinations = request.existing_combinations.clone();
    let target_idx = combinations
        .iter()
        .position(|c| c.id == request.combination_id)
        .ok_or_else(|| {
            CanonicalError::new(
                "ACTIONS_COMBINATION_NOT_FOUND",
                format!("Δεν βρέθηκε ο συνδυασμός με αναγνωριστικό '{}'.", request.combination_id),
                format!("Combination ID '{}' not found in existing combinations.", request.combination_id),
            )
        })?;

    if request.action == "Delete" {
        combinations.remove(target_idx);
    } else if request.action == "ResetToGenerated" {
        // Regenerate everything with reset_user_edited: true for this slot
        let mut clean_existing = Vec::new();
        for c in &combinations {
            if c.id != request.combination_id {
                clean_existing.push(c.clone());
            }
        }
        let gen_req = GenerateCombinationsRequest {
            load_cases: request.load_cases.clone(),
            profile: request.profile.clone(),
            seismic_settings: request.seismic_settings.clone(),
            existing_combinations: clean_existing,
            include_uls_geo: false,
            reset_user_edited: false,
        };
        return generate_combinations(&gen_req);
    }

    // Recompute envelopes
    let mut envelopes = Vec::new();
    for ls in [LimitState::ULS_STR, LimitState::SLS_CHAR, LimitState::SLS_FREQ, LimitState::SLS_QP, LimitState::SEISMIC] {
        let ids: Vec<String> = combinations.iter().filter(|c| c.limit_state == ls).map(|c| c.id.clone()).collect();
        if !ids.is_empty() {
            let (id_str, el, en) = match ls {
                LimitState::ULS_STR => ("ENV_ULS_STR", "Περιβάλλουσα ΟΚΑ (STR)", "ULS (STR) Envelope"),
                LimitState::SLS_CHAR => ("ENV_SLS_CHAR", "Περιβάλλουσα ΟΚΛ (Χαρακτηριστική)", "SLS (Characteristic) Envelope"),
                LimitState::SLS_FREQ => ("ENV_SLS_FREQ", "Περιβάλλουσα ΟΚΛ (Συχνή)", "SLS (Frequent) Envelope"),
                LimitState::SLS_QP => ("ENV_SLS_QP", "Περιβάλλουσα ΟΚΛ (Οιονεί Μόνιμη)", "SLS (Quasi-Permanent) Envelope"),
                LimitState::SEISMIC => ("ENV_SEISMIC", "Περιβάλλουσα Σεισμικών", "Seismic Envelope"),
                _ => ("ENV_OTHER", "Περιβάλλουσα", "Envelope"),
            };
            envelopes.push(EnvelopeSpecification {
                id: id_str.to_string(),
                name: BilingualText::new(el, en),
                limit_state: ls,
                combination_ids: ids,
            });
        }
    }

    Ok(CombinationSet {
        revision: 1,
        combinations,
        envelopes,
        warnings: Vec::new(),
    })
}
