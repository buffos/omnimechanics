use std::sync::Arc;

use super::job_manager::JobManager;
use super::pdelta_buckling::{solve_linear_buckling, solve_second_order_pdelta};
use super::types::*;
use super::unilateral::solve_unilateral_active_set;

fn create_cantilever_column(l: f64, p_axial: f64, h_lateral: f64) -> StructuralModel {
    let mut model = StructuralModel::default();
    let n_segs = 8;
    let dy = l / n_segs as f64;

    for i in 0..=n_segs {
        model.nodes.push(Node {
            id: format!("N{}", i + 1),
            x: 0.0,
            y: i as f64 * dy,
        });
    }

    model.supports = vec![Support {
        id: "S1".to_string(),
        node_id: "N1".to_string(),
        kind: SupportKind::Fixed,
        unilateral: false,
    }];

    // Simple steel column: E = 210 GPa, I = 1e-4 m^4, A = 0.01 m^2
    let e_gpa = 210.0;
    let a = 0.01;
    let i = 1e-4;

    model.materials = vec![Material {
        id: "MAT1".to_string(),
        name: "Steel".to_string(),
        source: LibrarySource::Custom,
        class: MaterialClass::Steel,
        e_gpa,
        g_gpa: 80.0,
        density_gamma: 78.5,
        alpha_t: 1.2e-5,
    }];

    model.sections = vec![Section {
        id: "SEC1".to_string(),
        name: "ColumnSec".to_string(),
        source: LibrarySource::Custom,
        shape: SectionShape::Generic,
        area_a: a,
        inertia_i: i,
        shear_area_av: None,
        depth_h: Some(0.2),
    }];

    for i in 0..n_segs {
        model.elements.push(Element::FrameMember {
            id: format!("M{}", i + 1),
            node_i: format!("N{}", i + 1),
            node_j: format!("N{}", i + 2),
            material_id: "MAT1".to_string(),
            section_id: "SEC1".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        });
    }

    let top_node_id = format!("N{}", n_segs + 1);
    let mut loads = Vec::new();
    if p_axial.abs() > 1e-6 {
        loads.push(Load::NodalLoad {
            node_id: top_node_id.clone(),
            fx: 0.0,
            fy: -p_axial,
            mz: 0.0,
        });
    }
    if h_lateral.abs() > 1e-6 {
        loads.push(Load::NodalLoad {
            node_id: top_node_id,
            fx: h_lateral,
            fy: 0.0,
            mz: 0.0,
        });
    }

    model.load_cases = vec![LoadCase {
        id: "LC1".to_string(),
        name: "ColumnLoad".to_string(),
        action_type: ActionType::Variable { sub_category: None },
        self_weight: false,
        loads,
    }];

    model
}

#[test]
fn test_sc004_pdelta_cantilever_amplification() {
    // Cantilever of L = 4.0 m
    // EI = 210e6 kN/m^2 * 1e-4 m^4 = 21000 kNm^2
    // Euler Pcr = pi^2 * EI / (4 * L^2) = pi^2 * 21000 / 64 = 3238.28 kN
    let l = 4.0;
    let ei = 210.0 * 1e6 * 1e-4;
    let p_cr_analytical = std::f64::consts::PI.powi(2) * ei / (4.0 * l * l);

    // Apply P = 0.3 * Pcr, H = 20 kN
    let p_applied = 0.3 * p_cr_analytical;
    let h_applied = 20.0;
    let model = create_cantilever_column(l, p_applied, h_applied);

    let comb = Combination {
        id: "CO1".to_string(),
        name: "Combo1".to_string(),
        factors: vec![LoadCaseFactor {
            load_case_id: "LC1".to_string(),
            factor: 1.0,
        }],
    };

    let res = solve_second_order_pdelta(&model, &[comb]).expect("P-Delta should converge");
    assert_eq!(res.len(), 1);

    // Analytical base moment M_base = H * L * tan(kL) / (kL) where k = sqrt(P / EI)
    let k = (p_applied / ei).sqrt();
    let kl = k * l;
    let amplification = kl.tan() / kl;
    let m_base_analytical = h_applied * l * amplification;

    let computed_m_base = res[0].support_reactions[0].mz_knm.abs();
    let rel_err = (computed_m_base - m_base_analytical).abs() / m_base_analytical;

    assert!(
        rel_err < 1e-3,
        "P-Delta base moment relative error {} exceeds 1e-3 (computed: {}, analytical: {})",
        rel_err,
        computed_m_base,
        m_base_analytical
    );
}

#[test]
fn test_sc004_linear_buckling_euler_critical_load() {
    let l = 4.0;
    let ei = 210.0 * 1e6 * 1e-4;
    let p_cr_analytical = std::f64::consts::PI.powi(2) * ei / (4.0 * l * l);

    // Reference load P_ref = 1000 kN
    let p_ref = 1000.0;
    let model = create_cantilever_column(l, p_ref, 0.0);

    let comb = Combination {
        id: "CO1".to_string(),
        name: "Combo1".to_string(),
        factors: vec![LoadCaseFactor {
            load_case_id: "LC1".to_string(),
            factor: 1.0,
        }],
    };

    let mode1 = solve_linear_buckling(&model, "CO1").expect("Buckling solve should succeed");
    let alpha_cr_1 = mode1.alpha_cr;
    let p_cr_computed = alpha_cr_1 * p_ref;

    let rel_err = (p_cr_computed - p_cr_analytical).abs() / p_cr_analytical;
    assert!(
        rel_err < 1e-4,
        "Euler critical load factor error {} exceeds 1e-4 (computed: {}, analytical: {})",
        rel_err,
        p_cr_computed,
        p_cr_analytical
    );

    // Verify mode normalization
    let max_disp = mode1
        .node_displacements
        .iter()
        .map(|d| d.dux.abs().max(d.duy.abs()).max(d.dtheta.abs()))
        .fold(0.0, f64::max);
    assert!(
        (max_disp - 1.0).abs() < 1e-6,
        "Buckling mode should be normalized to unit maximum displacement"
    );
}

#[test]
fn test_sc004_pdelta_buckling_divergence_error() {
    let l = 4.0;
    let ei = 210.0 * 1e6 * 1e-4;
    let p_cr_analytical = std::f64::consts::PI.powi(2) * ei / (4.0 * l * l);

    // P >= P_cr
    let p_excessive = 1.05 * p_cr_analytical;
    let model = create_cantilever_column(l, p_excessive, 10.0);

    let comb = Combination {
        id: "CO1".to_string(),
        name: "Combo1".to_string(),
        factors: vec![LoadCaseFactor {
            load_case_id: "LC1".to_string(),
            factor: 1.0,
        }],
    };

    let err = solve_second_order_pdelta(&model, &[comb]).unwrap_err();
    assert_eq!(err.code, "PDELTA_BUCKLING_DIVERGENCE");
}

#[test]
fn test_sc003_unilateral_active_set_x_bracing() {
    let mut model = StructuralModel::default();
    model.nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 4.0, y: 0.0 },
        Node { id: "N3".to_string(), x: 0.0, y: 3.0 },
        Node { id: "N4".to_string(), x: 4.0, y: 3.0 },
    ];

    model.supports = vec![
        Support {
            id: "S1".to_string(),
            node_id: "N1".to_string(),
            kind: SupportKind::Pinned,
            unilateral: false,
        },
        Support {
            id: "S2".to_string(),
            node_id: "N2".to_string(),
            kind: SupportKind::Pinned,
            unilateral: true,
        },
    ];

    model.materials = vec![Material {
        id: "MAT1".to_string(),
        name: "Steel".to_string(),
        source: LibrarySource::Custom,
        class: MaterialClass::Steel,
        e_gpa: 210.0,
        g_gpa: 80.0,
        density_gamma: 78.5,
        alpha_t: 1.2e-5,
    }];

    model.sections = vec![
        Section {
            id: "SEC_BEAM".to_string(),
            name: "Beam".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.01,
            inertia_i: 1e-4,
            shear_area_av: None,
            depth_h: Some(0.2),
        },
        Section {
            id: "SEC_BRACE".to_string(),
            name: "Brace".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.002,
            inertia_i: 1e-6,
            shear_area_av: None,
            depth_h: Some(0.05),
        },
    ];

    model.elements = vec![
        Element::FrameMember {
            id: "COL1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N3".to_string(),
            material_id: "MAT1".to_string(),
            section_id: "SEC_BEAM".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "COL2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N4".to_string(),
            material_id: "MAT1".to_string(),
            section_id: "SEC_BEAM".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "BEAM".to_string(),
            node_i: "N3".to_string(),
            node_j: "N4".to_string(),
            material_id: "MAT1".to_string(),
            section_id: "SEC_BEAM".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::TrussBar {
            id: "BRACE1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N4".to_string(),
            material_id: "MAT1".to_string(),
            section_id: "SEC_BRACE".to_string(),
            axial_behavior: AxialBehavior::TensionOnly,
        },
        Element::TrussBar {
            id: "BRACE2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N3".to_string(),
            material_id: "MAT1".to_string(),
            section_id: "SEC_BRACE".to_string(),
            axial_behavior: AxialBehavior::TensionOnly,
        },
    ];

    model.load_cases = vec![LoadCase {
        id: "WIND_RIGHT".to_string(),
        name: "Wind Right".to_string(),
        action_type: ActionType::Variable { sub_category: None },
        self_weight: false,
        loads: vec![Load::NodalLoad {
            node_id: "N3".to_string(),
            fx: 50.0,
            fy: 0.0,
            mz: 0.0,
        }],
    }];

    let comb = Combination {
        id: "CO1".to_string(),
        name: "ComboWind".to_string(),
        factors: vec![LoadCaseFactor {
            load_case_id: "WIND_RIGHT".to_string(),
            factor: 1.0,
        }],
    };

    let res = solve_unilateral_active_set(&model, &[comb]).expect("Active set should converge");
    assert_eq!(res.len(), 1);
    let r = &res[0];

    assert!(!r.superposition_valid);

    let b1_res = r.member_results.iter().find(|m| m.element_id == "BRACE1");
    assert!(b1_res.is_some(), "Active tension brace must have results");
    assert!(
        b1_res.unwrap().end_forces_local.ni_kn < 0.0,
        "BRACE1 should be active in tension"
    );

    let b2_res = r.member_results.iter().find(|m| m.element_id == "BRACE2");
    if let Some(b2) = b2_res {
        println!("b2 ni_kn = {}, nj_kn = {}", b2.end_forces_local.ni_kn, b2.end_forces_local.nj_kn);
        assert!(
            b2.end_forces_local.ni_kn.abs() < 1e-6,
            "Inactive compression brace should carry zero axial load"
        );
    }
}

#[test]
fn test_sc008_job_cancellation_and_revision_superseding() {
    let l = 4.0;
    let model = create_cantilever_column(l, 500.0, 10.0);
    let job_manager = Arc::new(JobManager::new());

    // 1. Start job
    let start_req = StartJobRequest {
        model_revision: model.revision,
        model: None,
        kind: AnalysisKind::SecondOrderPDelta,
        target_ids: Vec::new(),
        combinations: vec![Combination {
            id: "CO1".to_string(),
            name: "Combo1".to_string(),
            factors: vec![LoadCaseFactor {
                load_case_id: "LC1".to_string(),
                factor: 1.0,
            }],
        }],
        compute_envelope: false,
        damping_ratio: None,
        combination_rule: None,
    };

    let resp = job_manager
        .start_job(&model, start_req.clone(), None)
        .expect("Job should start");

    // Cancel job
    let status = job_manager.cancel_job(&resp.job_id).expect("Cancel should succeed");
    assert!(status == JobStatus::Cancelled || status == JobStatus::Converged);

    // 2. Test revision superseding
    let resp2 = job_manager
        .start_job(&model, start_req, None)
        .expect("Job should start");

    // Advance model revision
    job_manager.supersede_jobs_older_than(model.revision + 1);

    // Verify job is Superseded or Converged
    let job_res = job_manager
        .get_job_result(&resp2.job_id)
        .expect("Query should succeed");
    let status = job_res.as_ref().unwrap().status;
    assert!(status == JobStatus::Superseded || status == JobStatus::Converged);

    // 3. Stale revision rejection
    let stale_req = StartJobRequest {
        model_revision: model.revision + 99,
        model: None,
        kind: AnalysisKind::LinearStatic,
        target_ids: Vec::new(),
        combinations: Vec::new(),
        compute_envelope: false,
        damping_ratio: None,
        combination_rule: None,
    };
    let err = job_manager
        .start_job(&model, stale_req, None)
        .unwrap_err();
    assert_eq!(err.code, "STALE_MODEL_REVISION");
}
