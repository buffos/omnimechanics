use super::catenary::{
    compute_unstressed_length, irvine_spans, solve_catenary_cable, solve_irvine_catenary,
};
use super::types::*;

#[test]
fn test_sc005_irvine_transcendental_closed_form() {
    // Single suspended cable between (0, 0) and (100.0, 10.0)
    // Span = 100 m, dy = 10 m
    // ISO Cable Steel: E = 160 GPa = 160e6 kN/m^2, A = 0.002 m^2, gamma = 78.5 kN/m^3
    // w0 = 78.5 * 0.002 = 0.157 kN/m
    // EA = 160e6 * 0.002 = 320,000 kN
    let l_span = 100.0;
    let dy = 10.0;
    let ea = 320000.0;
    let w0 = 0.157;
    let l0 = 100.8; // slightly longer than straight chord sqrt(100^2+10^2)=100.498m => sag

    // Solve Irvine catenary
    let (h, v, _kt, is_slack) = solve_irvine_catenary(l_span, dy, l0, ea, w0)
        .expect("Irvine catenary solve should succeed");

    assert!(!is_slack, "Cable under self-weight should be taut");
    assert!(h > 0.0, "Horizontal tension must be positive");

    // Check compatibility back-substitution: irvine_spans(h, v) must equal (l_span, dy)
    let (dx_check, dy_check) = irvine_spans(h, v, l0, ea, w0);

    let err_x = (dx_check - l_span).abs() / l_span;
    let err_y = (dy_check - dy).abs() / dy.abs().max(1.0);

    assert!(
        err_x < 1e-4,
        "Irvine dx relative error {} exceeds 1e-4 (calc: {}, target: {})",
        err_x,
        dx_check,
        l_span
    );
    assert!(
        err_y < 1e-4,
        "Irvine dy error {} exceeds 1e-4 (calc: {}, target: {})",
        err_y,
        dy_check,
        dy
    );
}

#[test]
fn test_sc005_prestress_inputs_equivalence() {
    // Compare UnstressedLength vs Pretension vs SagRatio
    let dx = 80.0;
    let dy = 0.0;
    let ea = 200000.0;
    let w0 = 0.1;

    // Cable with T0 = 200 kN
    let t0 = 200.0;
    let l0_pretension = compute_unstressed_length(
        &CablePrestress::Pretension { t0 },
        dx,
        dy,
        ea,
        w0,
    );

    // Cable with sag ratio r = 0.02 (f = 1.6 m)
    let sag_to_span = 0.02;
    let l0_sag = compute_unstressed_length(
        &CablePrestress::SagRatio { sag_to_span },
        dx,
        dy,
        ea,
        w0,
    );

    assert!(l0_pretension > 0.0 && l0_pretension < dx);
    assert!(l0_sag > 0.0);

    // Direct UnstressedLength
    let l0_direct = compute_unstressed_length(
        &CablePrestress::UnstressedLength { l0: 79.92 },
        dx,
        dy,
        ea,
        w0,
    );
    assert_eq!(l0_direct, 79.92);
}

#[test]
fn test_sc005_guyed_mast_cable_stayed_and_slack_detection() {
    // Guyed mast:
    // Vertical column N1 (0, 0) to N2 (0, 10)
    // Windward cable C1 from N3 (-10, 0) to N2 (0, 10)
    // Leeward cable C2 from N4 (10, 0) to N2 (0, 10)
    // Column base N1 is Pinned
    // Cable ground anchors N3 and N4 are Fixed/Pinned
    let mut model = StructuralModel::default();
    model.nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 0.0, y: 10.0 },
        Node { id: "N3".to_string(), x: -10.0, y: 0.0 },
        Node { id: "N4".to_string(), x: 10.0, y: 0.0 },
    ];

    model.supports = vec![
        Support {
            id: "S1".to_string(),
            node_id: "N1".to_string(),
            kind: SupportKind::Pinned,
            unilateral: false,
        },
        Support {
            id: "S3".to_string(),
            node_id: "N3".to_string(),
            kind: SupportKind::Fixed,
            unilateral: false,
        },
        Support {
            id: "S4".to_string(),
            node_id: "N4".to_string(),
            kind: SupportKind::Fixed,
            unilateral: false,
        },
    ];

    model.materials = vec![
        Material {
            id: "MAT_STEEL".to_string(),
            name: "ColumnSteel".to_string(),
            source: LibrarySource::Custom,
            class: MaterialClass::Steel,
            e_gpa: 210.0,
            g_gpa: 80.0,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        },
        Material {
            id: "MAT_CABLE".to_string(),
            name: "CableSteel".to_string(),
            source: LibrarySource::Custom,
            class: MaterialClass::CableSteel,
            e_gpa: 160.0,
            g_gpa: 60.0,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        },
    ];

    model.sections = vec![
        Section {
            id: "SEC_COL".to_string(),
            name: "Column".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.01,
            inertia_i: 1e-4,
            shear_area_av: None,
            depth_h: Some(0.2),
        },
        Section {
            id: "SEC_CABLE".to_string(),
            name: "CableSec".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.001,
            inertia_i: 1e-7,
            shear_area_av: None,
            depth_h: Some(0.03),
        },
    ];

    // Column member N1-N2
    model.elements = vec![
        Element::FrameMember {
            id: "COL".to_string(),
            node_i: "N1".to_string(),
            node_j: "N2".to_string(),
            material_id: "MAT_STEEL".to_string(),
            section_id: "SEC_COL".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Stay 1 (Windward when wind blows right +X): N3 to N2
        // Chord = sqrt(10^2 + 10^2) = 14.142 m
        Element::Cable {
            id: "C1".to_string(),
            node_i: "N3".to_string(),
            node_j: "N2".to_string(),
            material_id: "MAT_CABLE".to_string(),
            section_id: "SEC_CABLE".to_string(),
            prestress: CablePrestress::Pretension { t0: 50.0 }, // 50 kN pretension
        },
        // Stay 2 (Leeward when wind blows right +X): N2 to N4
        Element::Cable {
            id: "C2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N4".to_string(),
            material_id: "MAT_CABLE".to_string(),
            section_id: "SEC_CABLE".to_string(),
            prestress: CablePrestress::Pretension { t0: 50.0 },
        },
    ];

    // High lateral wind load at mast top N2: Px = +120 kN
    // This pushes mast right, tightening C1 and slacking C2
    model.load_cases = vec![LoadCase {
        id: "WIND".to_string(),
        name: "Wind Load".to_string(),
        action_type: ActionType::Variable { sub_category: None },
        self_weight: false,
        loads: vec![Load::NodalLoad {
            node_id: "N2".to_string(),
            fx: 120.0,
            fy: 0.0,
            mz: 0.0,
        }],
    }];

    let comb = Combination {
        id: "CO1".to_string(),
        name: "ComboWind".to_string(),
        factors: vec![LoadCaseFactor {
            load_case_id: "WIND".to_string(),
            factor: 1.0,
        }],
    };

    let res = solve_catenary_cable(&model, &[comb]).expect("Catenary solve should succeed");
    assert_eq!(res.len(), 1);
    let r = &res[0];

    assert!(!r.superposition_valid);
    assert_eq!(r.cable_results.len(), 2);

    let c1 = r.cable_results.iter().find(|c| c.element_id == "C1").unwrap();
    let c2 = r.cable_results.iter().find(|c| c.element_id == "C2").unwrap();

    // Windward cable C1 must be in high tension (> 50 kN)
    assert!(
        c1.max_tension_kn > 50.0,
        "Windward cable tension should increase under wind load (got {})",
        c1.max_tension_kn
    );
    assert!(!c1.is_slack, "Windward cable must not be slack");

    // Leeward cable C2 must lose tension and be slack (or low tension)
    assert!(
        c2.max_tension_kn < c1.max_tension_kn,
        "Leeward cable tension ({}) should be much less than windward ({})",
        c2.max_tension_kn,
        c1.max_tension_kn
    );

    // Verify curve points generation
    assert_eq!(c1.curve_points_global.len(), 21);
    assert_eq!(c2.curve_points_global.len(), 21);
    assert!((c1.curve_points_global[0].x - (-10.0)).abs() < 1e-4);
    assert!((c1.curve_points_global[0].y - 0.0).abs() < 1e-4);
}
