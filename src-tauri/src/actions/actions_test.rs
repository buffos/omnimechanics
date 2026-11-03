use super::combinations::{
    delete_or_reset_combination, generate_combinations, save_custom_combination,
};
use super::na_profile::{get_default_na_profile, validate_na_profile};
use super::types::*;

#[test]
fn test_sc001_greek_national_annex_retrieval() {
    let res = get_default_na_profile(Some("GR")).expect("GR profile retrieval should succeed");
    let p = res.profile;

    // Normative defaults
    assert_eq!(p.code, "GR");
    assert_eq!(p.gamma_g_sup, 1.35);
    assert_eq!(p.gamma_g_inf, 1.00);
    assert_eq!(p.gamma_q_sup, 1.50);
    assert_eq!(p.gamma_q_inf, 0.00);
    assert_eq!(p.xi_reduction, 0.85);
    assert_eq!(p.uls_equation_policy, UlsEquationPolicy::Single6_10);
    assert_eq!(p.permanent_favorability_policy, PermanentFavorabilityPolicy::BothBranches);

    // Psi factors
    let psi_a = p.psi_factors.get("Imposed_A").unwrap();
    assert_eq!(psi_a.psi_0, 0.70);
    assert_eq!(psi_a.psi_1, 0.50);
    assert_eq!(psi_a.psi_2, 0.30);

    let psi_wind = p.psi_factors.get("Wind").unwrap();
    assert_eq!(psi_wind.psi_0, 0.60);
    assert_eq!(psi_wind.psi_1, 0.20);
    assert_eq!(psi_wind.psi_2, 0.00);

    // Guidance catalog
    assert!(!res.system_guidance_catalog.is_empty());
    let dcm = res.system_guidance_catalog.iter().find(|g| g.id == "RcMomentFrameDcm").unwrap();
    assert_eq!(dcm.default_q, 3.90);
}

#[test]
fn test_sc002_uls_str_eq6_10_and_sls_permutations() {
    let profile = get_default_na_profile(Some("GR")).unwrap().profile;

    let lc1 = LoadCaseDescriptor {
        id: "LC1".to_string(),
        code: "G1".to_string(),
        name: BilingualText::new("Μόνιμο Ίδιο Βάρος", "Permanent Self-Weight"),
        action_type: ActionType::Permanent,
    };
    let lc2 = LoadCaseDescriptor {
        id: "LC2".to_string(),
        code: "Q1".to_string(),
        name: BilingualText::new("Κινητό Κατοικιών", "Residential Imposed"),
        action_type: ActionType::Imposed { category: "A".to_string() },
    };
    let lc3 = LoadCaseDescriptor {
        id: "LC3".to_string(),
        code: "W1".to_string(),
        name: BilingualText::new("Άνεμος", "Wind"),
        action_type: ActionType::Wind,
    };

    let req = GenerateCombinationsRequest {
        load_cases: vec![lc1, lc2, lc3],
        profile,
        seismic_settings: None,
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    };

    let comb_set = generate_combinations(&req).expect("Generation should succeed");
    let combs = &comb_set.combinations;

    // 1. ULS STR: 4 leading permutations
    let uls_str: Vec<&Combination> = combs.iter().filter(|c| c.limit_state == LimitState::ULS_STR).collect();
    assert_eq!(uls_str.len(), 4);

    // Check Leading Q1, unfavorable G1: 1.35 G1 + 1.50 Q1 + 0.90 W1
    let q1_unfav = uls_str.iter().find(|c| c.generation_signature.as_deref() == Some("ULS_STR:Eq6_10:G_SUP:LEAD_LC2")).unwrap();
    let g_term = q1_unfav.terms.iter().find(|t| t.load_case_id == "LC1").unwrap();
    let q_term = q1_unfav.terms.iter().find(|t| t.load_case_id == "LC2").unwrap();
    let w_term = q1_unfav.terms.iter().find(|t| t.load_case_id == "LC3").unwrap();
    assert_eq!(g_term.factor, 1.35);
    assert_eq!(q_term.factor, 1.50);
    assert_eq!(w_term.factor, 0.90); // 1.50 * 0.60 = 0.90

    // Check Leading W1, unfavorable G1: 1.35 G1 + 1.50 W1 + 1.05 Q1
    let w1_unfav = uls_str.iter().find(|c| c.generation_signature.as_deref() == Some("ULS_STR:Eq6_10:G_SUP:LEAD_LC3")).unwrap();
    let w_lead = w1_unfav.terms.iter().find(|t| t.load_case_id == "LC3").unwrap();
    let q_acc = w1_unfav.terms.iter().find(|t| t.load_case_id == "LC2").unwrap();
    assert_eq!(w_lead.factor, 1.50);
    assert_eq!(q_acc.factor, 1.05); // 1.50 * 0.70 = 1.05

    // 2. SLS Characteristic: 2 combinations
    let sls_char: Vec<&Combination> = combs.iter().filter(|c| c.limit_state == LimitState::SLS_CHAR).collect();
    assert_eq!(sls_char.len(), 2);

    // 3. SLS Frequent: 2 combinations
    let sls_freq: Vec<&Combination> = combs.iter().filter(|c| c.limit_state == LimitState::SLS_FREQ).collect();
    assert_eq!(sls_freq.len(), 2);
    // Leading Q1: psi1(Q1) = 0.50, psi2(W1) = 0.0 => W1 omitted
    let q1_freq = sls_freq.iter().find(|c| c.generation_signature.as_deref() == Some("SLS_FREQ:Eq6_15b:LEAD_LC2")).unwrap();
    assert_eq!(q1_freq.terms.len(), 2);
    assert_eq!(q1_freq.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 0.50);

    // 4. SLS Quasi-Permanent: 1 combination (1.0 G1 + 0.30 Q1)
    let sls_qp: Vec<&Combination> = combs.iter().filter(|c| c.limit_state == LimitState::SLS_QP).collect();
    assert_eq!(sls_qp.len(), 1);
    assert_eq!(sls_qp[0].terms.len(), 2);
    assert_eq!(sls_qp[0].terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 0.30);

    // Envelopes
    assert_eq!(comb_set.envelopes.len(), 4);
    assert!(comb_set.envelopes.iter().any(|e| e.id == "ENV_ULS_STR" && e.combination_ids.len() == 4));
}

#[test]
fn test_sc003_twin_equations_6_10a_and_6_10b() {
    let mut profile = get_default_na_profile(Some("GR")).unwrap().profile;
    profile.uls_equation_policy = UlsEquationPolicy::Twin6_10a_6_10b;
    profile.permanent_favorability_policy = PermanentFavorabilityPolicy::UnfavorableOnly;
    profile.xi_reduction = 0.85;

    let lc1 = LoadCaseDescriptor {
        id: "LC1".to_string(),
        code: "G1".to_string(),
        name: BilingualText::new("Μόνιμο", "Permanent"),
        action_type: ActionType::Permanent,
    };
    let lc2 = LoadCaseDescriptor {
        id: "LC2".to_string(),
        code: "Q1".to_string(),
        name: BilingualText::new("Γραφεία", "Offices"),
        action_type: ActionType::Imposed { category: "B".to_string() }, // psi0 = 0.70
    };

    let req = GenerateCombinationsRequest {
        load_cases: vec![lc1, lc2],
        profile,
        seismic_settings: None,
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    };

    let comb_set = generate_combinations(&req).unwrap();
    let uls_str: Vec<&Combination> = comb_set.combinations.iter().filter(|c| c.limit_state == LimitState::ULS_STR).collect();

    // Must have Eq6_10a and Eq6_10b
    let c_10a = uls_str.iter().find(|c| c.equation == Some(CombinationEquation::Eq6_10a)).unwrap();
    let c_10b = uls_str.iter().find(|c| c.equation == Some(CombinationEquation::Eq6_10b)).unwrap();

    // 10a: 1.35 G1 + 1.05 Q1
    assert_eq!(c_10a.terms.iter().find(|t| t.load_case_id == "LC1").unwrap().factor, 1.35);
    assert_eq!(c_10a.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 1.05);

    // 10b: (0.85 * 1.35 = 1.1475) G1 + 1.50 Q1
    let g_factor = c_10b.terms.iter().find(|t| t.load_case_id == "LC1").unwrap().factor;
    assert!((g_factor - 1.1475).abs() < 1e-6);
    assert_eq!(c_10b.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 1.50);
}

#[test]
fn test_sc004_user_edited_and_manual_survive_regeneration() {
    let profile = get_default_na_profile(Some("GR")).unwrap().profile;

    let lc1 = LoadCaseDescriptor {
        id: "LC1".to_string(),
        code: "G1".to_string(),
        name: BilingualText::new("G1", "G1"),
        action_type: ActionType::Permanent,
    };
    let lc2 = LoadCaseDescriptor {
        id: "LC2".to_string(),
        code: "Q1".to_string(),
        name: BilingualText::new("Q1", "Q1"),
        action_type: ActionType::Imposed { category: "A".to_string() },
    };

    // 1. Initial generation
    let gen1 = generate_combinations(&GenerateCombinationsRequest {
        load_cases: vec![lc1.clone(), lc2.clone()],
        profile: profile.clone(),
        seismic_settings: None,
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    }).unwrap();

    let co1 = gen1.combinations[0].clone();
    assert_eq!(co1.origin, CombinationOrigin::Generated);

    // 2. Edit CO1: change Q1 factor from 1.50 to 1.65
    let mut edited_terms = co1.terms.clone();
    for t in &mut edited_terms {
        if t.load_case_id == "LC2" {
            t.factor = 1.65;
        }
    }
    let save_req = SaveCustomCombinationRequest {
        combination: CombinationDraft {
            id: Some(co1.id.clone()),
            name: co1.name.clone(),
            limit_state: co1.limit_state,
            terms: edited_terms,
        },
        load_cases: vec![lc1.clone(), lc2.clone()],
        existing_combinations: gen1.combinations.clone(),
    };
    let comb_set_edited = save_custom_combination(&save_req).unwrap();
    let co1_edited = comb_set_edited.combinations.iter().find(|c| c.id == co1.id).unwrap();
    assert_eq!(co1_edited.origin, CombinationOrigin::UserEdited);
    assert_eq!(co1_edited.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 1.65);

    // 3. Add a manual combination
    let manual_req = SaveCustomCombinationRequest {
        combination: CombinationDraft {
            id: None,
            name: BilingualText::new("Manual Test", "Manual Test"),
            limit_state: LimitState::ULS_STR,
            terms: vec![
                CombinationTerm { load_case_id: "LC1".to_string(), factor: 1.20, role: CombinationRole::ManualFactor },
                CombinationTerm { load_case_id: "LC2".to_string(), factor: 1.20, role: CombinationRole::ManualFactor },
            ],
        },
        load_cases: vec![lc1.clone(), lc2.clone()],
        existing_combinations: comb_set_edited.combinations.clone(),
    };
    let comb_set_with_manual = save_custom_combination(&manual_req).unwrap();
    assert!(comb_set_with_manual.combinations.iter().any(|c| c.origin == CombinationOrigin::Manual));

    // 4. Add LC3 (Snow) and regenerate with reset_user_edited: false
    let lc3 = LoadCaseDescriptor {
        id: "LC3".to_string(),
        code: "S1".to_string(),
        name: BilingualText::new("S1", "S1"),
        action_type: ActionType::Snow,
    };
    let regen_set = generate_combinations(&GenerateCombinationsRequest {
        load_cases: vec![lc1.clone(), lc2.clone(), lc3],
        profile: profile.clone(),
        seismic_settings: None,
        existing_combinations: comb_set_with_manual.combinations.clone(),
        include_uls_geo: false,
        reset_user_edited: false,
    }).unwrap();

    // CO1 must retain 1.65 and UserEdited origin
    let co1_retained = regen_set.combinations.iter().find(|c| c.id == co1.id).unwrap();
    assert_eq!(co1_retained.origin, CombinationOrigin::UserEdited);
    assert_eq!(co1_retained.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 1.65);

    // Manual combination must still exist
    assert!(regen_set.combinations.iter().any(|c| c.origin == CombinationOrigin::Manual));

    // 5. Reset CO1 to generated
    let reset_req = DeleteOrResetCombinationRequest {
        combination_id: co1.id.clone(),
        action: "ResetToGenerated".to_string(),
        load_cases: vec![lc1, lc2],
        profile,
        seismic_settings: None,
        existing_combinations: regen_set.combinations,
    };
    let reset_set = delete_or_reset_combination(&reset_req).unwrap();
    let co1_reset = reset_set.combinations.iter().find(|c| c.id == co1.id).unwrap();
    assert_eq!(co1_reset.origin, CombinationOrigin::Generated);
    assert_eq!(co1_reset.terms.iter().find(|t| t.load_case_id == "LC2").unwrap().factor, 1.50);
}

#[test]
fn test_sc009_normative_validation_rejections() {
    let mut profile = get_default_na_profile(Some("GR")).unwrap().profile;

    // 1. Invalid psi hierarchy: psi1 > psi0
    profile.psi_factors.insert("Imposed_A".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.85, psi_2: 0.30 });
    let err = validate_na_profile(&profile).unwrap_err();
    assert_eq!(err.code, "ACTIONS_INVALID_PSI_FACTOR");

    // Restore profile
    profile = get_default_na_profile(Some("GR")).unwrap().profile;

    // 2. Missing permanent load case
    let lc_imposed = LoadCaseDescriptor {
        id: "LC1".to_string(),
        code: "Q1".to_string(),
        name: BilingualText::new("Q1", "Q1"),
        action_type: ActionType::Imposed { category: "A".to_string() },
    };
    let err2 = generate_combinations(&GenerateCombinationsRequest {
        load_cases: vec![lc_imposed],
        profile: profile.clone(),
        seismic_settings: None,
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    }).unwrap_err();
    assert_eq!(err2.code, "ACTIONS_NO_PERMANENT_LOAD_CASE");

    // 3. Accidental load case rejected
    let lc_perm = LoadCaseDescriptor {
        id: "LC1".to_string(),
        code: "G1".to_string(),
        name: BilingualText::new("G1", "G1"),
        action_type: ActionType::Permanent,
    };
    let lc_acc = LoadCaseDescriptor {
        id: "LC2".to_string(),
        code: "A1".to_string(),
        name: BilingualText::new("A1", "A1"),
        action_type: ActionType::Accidental,
    };
    let err3 = generate_combinations(&GenerateCombinationsRequest {
        load_cases: vec![lc_perm, lc_acc],
        profile,
        seismic_settings: None,
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    }).unwrap_err();
    assert_eq!(err3.code, "ACTIONS_ACCIDENTAL_NOT_SUPPORTED");
}
