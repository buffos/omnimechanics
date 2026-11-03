use super::combinations::generate_combinations;
use super::na_profile::get_default_gr_profile;
use super::seismic::*;
use super::types::*;

#[test]
fn test_sc005_en1998_type1_design_spectrum_evaluation() {
    // SC-005 Preconditions:
    // ZoneII (agR = 0.24g), GroundType::B (S = 1.20, TB = 0.15s, TC = 0.50s, TD = 2.00s)
    // ClassIII (gamma_I = 1.20 => ag = 0.288g = 2.82528 m/s2)
    // q = 3.90, zeta = 5%, beta = 0.20, include_vertical_component = true, qv = 1.50
    let settings = SeismicSettings {
        zone: HazardZone::ZoneII,
        ag_reference_g: 0.24,
        ground_type: GroundType::B,
        soil_factor_s: 1.20,
        tb_s: 0.15,
        tc_s: 0.50,
        td_s: 2.00,
        importance_class: ImportanceClass::ClassIII,
        gamma_i: 1.20,
        system_guidance: Some("RcMomentFrameDcm".to_string()),
        behavior_factor_q: 3.90,
        damping_ratio_percent: 5.0,
        beta_lower_bound: 0.20,
        modal_combination_rule: ModalCombinationRule::CQC,
        include_vertical_component: true,
        vertical_behavior_factor_qv: 1.50,
        psi_e_phi: 1.00,
    };

    let req = ComputeDesignSpectrumRequest {
        settings,
        max_period_s: Some(4.0),
        num_points: Some(50),
        query_periods_s: Some(vec![0.0, 0.15, 0.50, 1.00, 3.00]),
    };

    let res = compute_design_spectrum(&req).expect("Spectrum evaluation failed");

    // 1. ag check: 0.288g = 2.82528 m/s2
    let g = 9.81;
    assert!((res.spectrum.ag_g - 0.288).abs() < 1e-9);
    assert!((res.spectrum.ag_m_s2 - 2.82528).abs() < 1e-6);

    // Check queried ordinates
    let pts = &res.queried_ordinates;
    assert_eq!(pts.len(), 5);

    // 2. T = 0.0s: Se(0)/g = 0.288 * 1.20 = 0.3456; Sd(0)/g = 0.288 * 1.20 * (2/3) = 0.2304
    let pt0 = &pts[0];
    assert!((pt0.period_s - 0.0).abs() < 1e-9);
    assert!((pt0.se_g - 0.3456).abs() < 1e-6);
    assert!((pt0.sd_g - 0.2304).abs() < 1e-6);
    assert!((pt0.se_m_s2 - 0.3456 * g).abs() < 1e-5);
    assert!((pt0.sd_m_s2 - 0.2304 * g).abs() < 1e-5);

    // 3. Plateau T in [0.15, 0.50]s: Se(T)/g = 0.288 * 1.20 * 2.5 = 0.864; Sd(T)/g = 0.864 / 3.90 = 0.221538...
    let pt_tb = &pts[1];
    let pt_tc = &pts[2];
    assert!((pt_tb.se_g - 0.864).abs() < 1e-6);
    assert!((pt_tc.se_g - 0.864).abs() < 1e-6);
    let expected_sd_plat = 0.864 / 3.90;
    assert!((pt_tb.sd_g - expected_sd_plat).abs() < 1e-6);
    assert!((pt_tc.sd_g - expected_sd_plat).abs() < 1e-6);

    // 4. T = 1.00s in (TC, TD]: Sd(1.0)/g = max(0.221538 * (0.50 / 1.00), 0.20 * 0.288) = max(0.110769, 0.0576) = 0.110769
    let pt1 = &pts[3];
    assert!((pt1.sd_g - 0.11076923).abs() < 1e-5);

    // 5. T = 3.00s > TD: unconstrained = 0.024615 < beta * ag/g = 0.0576, clamped to lower bound 0.0576
    let pt3 = &pts[4];
    assert!((pt3.sd_g - 0.0576).abs() < 1e-6);
    assert!((pt3.sd_m_s2 - 0.0576 * g).abs() < 1e-5);

    // 6. Vertical spectrum populated with avg = 0.90 * ag = 0.2592g
    assert!(res.spectrum.vertical_curve.is_some());
    let v_curve = res.spectrum.vertical_curve.unwrap();
    assert!(!v_curve.is_empty());
    assert!((res.spectrum.avg_m_s2.unwrap() - 0.2592 * g).abs() < 1e-5);
}

#[test]
fn test_sc006_seismic_mass_participation_and_signed_combinations() {
    let profile = get_default_gr_profile();
    let settings = SeismicSettings {
        zone: HazardZone::ZoneII,
        ag_reference_g: 0.24,
        ground_type: GroundType::B,
        soil_factor_s: 1.20,
        tb_s: 0.15,
        tc_s: 0.50,
        td_s: 2.00,
        importance_class: ImportanceClass::ClassII,
        gamma_i: 1.00,
        system_guidance: Some("RcMomentFrameDcm".to_string()),
        behavior_factor_q: 3.90,
        damping_ratio_percent: 5.0,
        beta_lower_bound: 0.20,
        modal_combination_rule: ModalCombinationRule::CQC,
        include_vertical_component: false,
        vertical_behavior_factor_qv: 1.50,
        psi_e_phi: 1.00,
    };

    let load_cases = vec![
        LoadCaseDescriptor {
            id: "LC1".to_string(),
            code: "G1".to_string(),
            name: BilingualText::new("Ίδιο Βάρος", "Self Weight"),
            action_type: ActionType::Permanent,
        },
        LoadCaseDescriptor {
            id: "LC2".to_string(),
            code: "Q1".to_string(),
            name: BilingualText::new("Ωφέλιμο Κατοικίας", "Imposed Domestic"),
            action_type: ActionType::Imposed {
                category: "A".to_string(),
            },
        },
        LoadCaseDescriptor {
            id: "LC3".to_string(),
            code: "Q2".to_string(),
            name: BilingualText::new("Ωφέλιμο Αποθήκης", "Imposed Storage"),
            action_type: ActionType::Imposed {
                category: "E".to_string(),
            },
        },
        LoadCaseDescriptor {
            id: "LC4".to_string(),
            code: "W1".to_string(),
            name: BilingualText::new("Άνεμος", "Wind"),
            action_type: ActionType::Wind,
        },
        LoadCaseDescriptor {
            id: "LC5".to_string(),
            code: "Ex".to_string(),
            name: BilingualText::new("Σεισμός X", "Seismic X"),
            action_type: ActionType::Seismic,
        },
    ];

    // 1. SeismicMassCombinationRule check: 1.00 LC1, 0.30 LC2, 0.80 LC3, omits LC4 and LC5
    let mass_req = ComputeSeismicMassRuleRequest {
        load_cases: load_cases.clone(),
        profile: profile.clone(),
        settings: settings.clone(),
    };
    let mass_rule = compute_seismic_mass_rule(&mass_req).expect("Mass rule computation failed");
    assert_eq!(mass_rule.terms.len(), 3);
    let t_g = mass_rule.terms.iter().find(|t| t.load_case_id == "LC1").unwrap();
    let t_q1 = mass_rule.terms.iter().find(|t| t.load_case_id == "LC2").unwrap();
    let t_q2 = mass_rule.terms.iter().find(|t| t.load_case_id == "LC3").unwrap();
    assert_eq!(t_g.factor, 1.00);
    assert_eq!(t_q1.factor, 0.30);
    assert_eq!(t_q2.factor, 0.80);
    assert!(mass_rule.terms.iter().all(|t| t.load_case_id != "LC4" && t.load_case_id != "LC5"));

    // 2. GenerateCombinations with horizontal-only seismic: emits 2 SEISMIC combinations (+Ex and -Ex)
    let gen_req = GenerateCombinationsRequest {
        load_cases: load_cases.clone(),
        profile: profile.clone(),
        seismic_settings: Some(settings.clone()),
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    };
    let combs = generate_combinations(&gen_req).expect("Combinations generation failed");
    let seismic_combs: Vec<&Combination> = combs
        .combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SEISMIC)
        .collect();
    assert_eq!(seismic_combs.len(), 2);

    let comb_pos = seismic_combs
        .iter()
        .find(|c| c.generation_signature.as_deref() == Some("SEISMIC:Eq6_12b:LC5:+E"))
        .unwrap();
    let comb_neg = seismic_combs
        .iter()
        .find(|c| c.generation_signature.as_deref() == Some("SEISMIC:Eq6_12b:LC5:-E"))
        .unwrap();

    let ex_pos = comb_pos.terms.iter().find(|t| t.load_case_id == "LC5").unwrap();
    let ex_neg = comb_neg.terms.iter().find(|t| t.load_case_id == "LC5").unwrap();
    assert_eq!(ex_pos.factor, 1.00);
    assert_eq!(ex_neg.factor, -1.00);

    // Verify ENV_SEISMIC exists and includes both
    let env_seismic = combs
        .envelopes
        .iter()
        .find(|e| e.id == "ENV_SEISMIC")
        .expect("ENV_SEISMIC not found");
    assert_eq!(env_seismic.combination_ids.len(), 2);

    // 3. Enable vertical seismic component and add LC6 (Ez, Seismic)
    let mut lc_with_v = load_cases.clone();
    lc_with_v.push(LoadCaseDescriptor {
        id: "LC6".to_string(),
        code: "Ez".to_string(),
        name: BilingualText::new("Σεισμός Z", "Seismic Z"),
        action_type: ActionType::Seismic,
    });
    let mut settings_v = settings.clone();
    settings_v.include_vertical_component = true;

    let gen_req_v = GenerateCombinationsRequest {
        load_cases: lc_with_v,
        profile,
        seismic_settings: Some(settings_v),
        existing_combinations: Vec::new(),
        include_uls_geo: false,
        reset_user_edited: false,
    };
    let combs_v = generate_combinations(&gen_req_v).expect("Combinations generation with vertical failed");
    let seismic_v: Vec<&Combination> = combs_v
        .combinations
        .iter()
        .filter(|c| c.limit_state == LimitState::SEISMIC)
        .collect();
    // 8 signed 30% combinations
    assert_eq!(seismic_v.len(), 8);
}

#[test]
fn test_sc007_en1998_lateral_force_base_shear_and_story_distribution() {
    // SC-007 Preconditions:
    // SeismicSettings from SC-005 where TC = 0.50s, T1 = 0.40s <= 2 TC
    // 3-storey frame (n_storeys = 3 > 2 => lambda = 0.85)
    // Level 1: z1 = 3.0 m, m1 = 100 t (z1*m1 = 300)
    // Level 2: z2 = 6.0 m, m2 = 100 t (z2*m2 = 600)
    // Level 3: z3 = 9.0 m, m3 = 80 t  (z3*m3 = 720)
    // Total mass = 280 t, Sum zm = 1620 t*m
    let settings = SeismicSettings {
        zone: HazardZone::ZoneII,
        ag_reference_g: 0.24,
        ground_type: GroundType::B,
        soil_factor_s: 1.20,
        tb_s: 0.15,
        tc_s: 0.50,
        td_s: 2.00,
        importance_class: ImportanceClass::ClassIII,
        gamma_i: 1.20,
        system_guidance: Some("RcMomentFrameDcm".to_string()),
        behavior_factor_q: 3.90,
        damping_ratio_percent: 5.0,
        beta_lower_bound: 0.20,
        modal_combination_rule: ModalCombinationRule::CQC,
        include_vertical_component: false,
        vertical_behavior_factor_qv: 1.50,
        psi_e_phi: 1.00,
    };

    let levels = vec![
        MassLevelInput {
            level_name: "Story 1".to_string(),
            elevation_z_m: 3.0,
            mass_tonnes: 100.0,
        },
        MassLevelInput {
            level_name: "Story 2".to_string(),
            elevation_z_m: 6.0,
            mass_tonnes: 100.0,
        },
        MassLevelInput {
            level_name: "Story 3".to_string(),
            elevation_z_m: 9.0,
            mass_tonnes: 80.0,
        },
    ];

    let req = ComputeLateralForcesRequest {
        settings,
        fundamental_period_t1_s: 0.40,
        n_storeys: 3,
        levels,
    };

    let res = compute_lateral_forces(&req).expect("Lateral force computation failed");

    // 1. lambda = 0.85
    assert_eq!(res.lambda_correction, 0.85);

    // 2. Base shear Fb = 2.1732923 * 280 * 0.85 = 517.2435 kN
    assert!((res.base_shear_kn - 517.243567).abs() < 1e-2);

    // 3. Level forces: F1 = 95.7858 kN, F2 = 191.5717 kN, F3 = 229.8860 kN
    assert_eq!(res.story_forces.len(), 3);
    assert!((res.story_forces[0].force_kn - 95.7858).abs() < 1e-2);
    assert!((res.story_forces[1].force_kn - 191.5717).abs() < 1e-2);
    assert!((res.story_forces[2].force_kn - 229.8860).abs() < 1e-2);

    // Sum of forces equals Fb
    let sum_f: f64 = res.story_forces.iter().map(|s| s.force_kn).sum();
    assert!((sum_f - res.base_shear_kn).abs() < 1e-5);
}

#[test]
fn test_sc008_cqc_modal_correlation_matrix() {
    // SC-008 Preconditions:
    // Modal periods T = [0.50s, 0.51s, 0.15s], damping = 5%
    let periods = vec![0.50, 0.51, 0.15];
    let req = ComputeCqcMatrixRequest {
        rule: "CQC".to_string(),
        modal_periods_s: periods.clone(),
        damping_ratio_percent: 5.0,
    };

    let res = compute_cqc_matrix(&req).expect("CQC computation failed");

    // 1. Diagonal entries = 1.0
    for i in 0..3 {
        assert!((res.matrix[i][i] - 1.0).abs() < 1e-9);
    }

    // 2. Symmetric
    for i in 0..3 {
        for j in 0..3 {
            assert!((res.matrix[i][j] - res.matrix[j][i]).abs() < 1e-9);
        }
    }

    // 3. Closely spaced modes 1 and 2 (0.50s, 0.51s): strong correlation rho_12 > 0.90 (~0.962..0.963)
    let rho_12 = res.matrix[0][1];
    assert!(rho_12 > 0.95 && rho_12 < 0.98);

    // 4. Well-separated modes 1 and 3 (0.50s, 0.15s): small correlation rho_13 < 0.02 (~0.005..0.008)
    let rho_13 = res.matrix[0][2];
    assert!(rho_13 < 0.02 && rho_13 > 0.001);

    // 5. Test SRSS produces identity matrix
    let req_srss = ComputeCqcMatrixRequest {
        rule: "SRSS".to_string(),
        modal_periods_s: periods,
        damping_ratio_percent: 5.0,
    };
    let res_srss = compute_cqc_matrix(&req_srss).expect("SRSS failed");
    for i in 0..3 {
        for j in 0..3 {
            if i == j {
                assert_eq!(res_srss.matrix[i][j], 1.0);
            } else {
                assert_eq!(res_srss.matrix[i][j], 0.0);
            }
        }
    }
}

#[test]
fn test_sc009_seismic_normative_validation_rejections() {
    // 1. q < 1.0
    let mut bad_settings = SeismicSettings::default();
    bad_settings.behavior_factor_q = 0.50;
    let err = validate_seismic_settings(&bad_settings).unwrap_err();
    assert_eq!(err.code, "ACTIONS_INVALID_SEISMIC_PARAMETERS");

    // 2. TB >= TC
    let mut bad_tb_tc = SeismicSettings::default();
    bad_tb_tc.tb_s = 0.60;
    bad_tb_tc.tc_s = 0.50;
    let err2 = validate_seismic_settings(&bad_tb_tc).unwrap_err();
    assert_eq!(err2.code, "ACTIONS_INVALID_SEISMIC_PARAMETERS");

    // 3. Negative period query
    let req = ComputeDesignSpectrumRequest {
        settings: SeismicSettings::default(),
        max_period_s: Some(4.0),
        num_points: Some(20),
        query_periods_s: Some(vec![-0.1]),
    };
    let err3 = compute_design_spectrum(&req).unwrap_err();
    assert_eq!(err3.code, "ACTIONS_INVALID_SPECTRUM_PERIOD");

    // 4. Invalid lateral force input: empty levels
    let req_lf = ComputeLateralForcesRequest {
        settings: SeismicSettings::default(),
        fundamental_period_t1_s: 0.5,
        n_storeys: 1,
        levels: Vec::new(),
    };
    let err4 = compute_lateral_forces(&req_lf).unwrap_err();
    assert_eq!(err4.code, "ACTIONS_INVALID_LATERAL_FORCE_INPUT");
}
