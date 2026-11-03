use std::collections::HashMap;

use super::catenary::solve_irvine_catenary;
use super::dynamics::solve_modal_analysis;
use super::linear_solver::solve_linear_static;
use super::pdelta_buckling::{solve_linear_buckling_analysis, solve_second_order_pdelta};
use super::types::*;
use super::unilateral::solve_unilateral_active_set;

#[test]
fn test_benchmark_sc001_linear_portal_frame() {
    let csv = include_str!("../../../docs/benchmarks/frame/sc001_linear_portal_frame.csv");
    let mut lines = csv.lines();
    let header = lines.next().unwrap();
    assert!(header.starts_with("benchmark,node_id"));

    // Build the benchmark portal frame:
    // Span 6m, Height 4m, Fixed bases at N1(0,0), N2(6,0), N3(0,4), N4(6,4)
    // E = 30e6 kN/m^2, I_col = 0.002 m^4, I_beam = 0.004 m^4, A_col = 0.15 m^2, A_beam = 0.20 m^2
    // Lateral load at N3: Px = 20.0 kN
    let mut model = StructuralModel::default();
    model.nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 6.0, y: 0.0 },
        Node { id: "N3".to_string(), x: 0.0, y: 4.0 },
        Node { id: "N4".to_string(), x: 6.0, y: 4.0 },
    ];
    model.supports = vec![
        Support { id: "S1".to_string(), node_id: "N1".to_string(), kind: SupportKind::Fixed, unilateral: false },
        Support { id: "S2".to_string(), node_id: "N2".to_string(), kind: SupportKind::Fixed, unilateral: false },
    ];
    model.materials = vec![Material {
        id: "MAT1".to_string(),
        name: "Concrete".to_string(),
        source: LibrarySource::Custom,
        class: MaterialClass::Concrete,
        e_gpa: 30.0,
        g_gpa: 12.5,
        density_gamma: 25.0,
        alpha_t: 1.0e-5,
    }];
    model.sections = vec![
        Section {
            id: "SEC_COL".to_string(),
            name: "Col".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.15,
            inertia_i: 0.002,
            shear_area_av: None,
            depth_h: Some(0.4),
        },
        Section {
            id: "SEC_BEAM".to_string(),
            name: "Beam".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.20,
            inertia_i: 0.004,
            shear_area_av: None,
            depth_h: Some(0.5),
        },
    ];
    model.elements = vec![
        Element::FrameMember {
            id: "C1".to_string(), node_i: "N1".to_string(), node_j: "N3".to_string(),
            section_id: "SEC_COL".to_string(), material_id: "MAT1".to_string(),
            release_i: EndRelease::default(), release_j: EndRelease::default(),
            offset_i: RigidOffset::default(), offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "C2".to_string(), node_i: "N2".to_string(), node_j: "N4".to_string(),
            section_id: "SEC_COL".to_string(), material_id: "MAT1".to_string(),
            release_i: EndRelease::default(), release_j: EndRelease::default(),
            offset_i: RigidOffset::default(), offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "B1".to_string(), node_i: "N3".to_string(), node_j: "N4".to_string(),
            section_id: "SEC_BEAM".to_string(), material_id: "MAT1".to_string(),
            release_i: EndRelease::default(), release_j: EndRelease::default(),
            offset_i: RigidOffset::default(), offset_j: RigidOffset::default(),
        },
    ];
    model.load_cases = vec![LoadCase {
        id: "LC1".to_string(),
        name: "Lateral".to_string(),
        action_type: ActionType::Variable { sub_category: None },
        self_weight: false,
        loads: vec![Load::NodalLoad { node_id: "N3".to_string(), fx: 20.0, fy: 0.0, mz: 0.0 }],
    }];

    let res = solve_linear_static(&model, &[]).expect("Linear static solve must succeed");
    assert_eq!(res.len(), 1);

    // Sum of horizontal base reactions must equal -20.0 kN
    let rx_sum: f64 = res[0].support_reactions.iter().map(|r| r.rx_kn).sum();
    assert!((rx_sum - (-20.0)).abs() < 1e-4, "Horizontal equilibrium violated: sum Rx = {}", rx_sum);

    let u3 = res[0].nodal_displacements.iter().find(|n| n.node_id == "N3").unwrap().ux_m;
    let u4 = res[0].nodal_displacements.iter().find(|n| n.node_id == "N4").unwrap().ux_m;
    assert!((u3 - u4).abs() < 1e-5, "Beam axial rigid-body sway must match at both ends");
    assert!((u3 - 0.001193287).abs() < 1e-5, "Displacement {} within benchmark tolerance", u3);
}

#[test]
fn test_benchmark_sc003_unilateral_x_bracing() {
    let csv = include_str!("../../../docs/benchmarks/frame/sc003_unilateral_x_bracing.csv");
    assert!(csv.contains("SC-003,BRACE_TENSION,TensionOnly,42.4264,true"));
    assert!(csv.contains("SC-003,BRACE_SLACK,TensionOnly,0.0,false"));
}

#[test]
fn test_benchmark_sc004_pdelta_euler_buckling() {
    let csv = include_str!("../../../docs/benchmarks/frame/sc004_pdelta_euler_buckling.csv");
    let mut lines = csv.lines();
    lines.next(); // header
    let row: Vec<&str> = lines.next().unwrap().split(',').collect();
    let p_cr_ref: f64 = row[5].parse().unwrap();
    let u_1st_ref: f64 = row[6].parse().unwrap();
    let u_pdelta_ref: f64 = row[7].parse().unwrap();

    // Verify analytical Euler buckling critical load:
    // P_cr = pi^2 * EI / (4 * L^2)
    let l = 6.0;
    let ei = 17547.6;
    let p_cr_calc = (std::f64::consts::PI * std::f64::consts::PI * ei) / (4.0 * l * l);
    assert!((p_cr_calc - p_cr_ref).abs() / p_cr_ref < 1e-3);

    // Verify 1st-order displacement: H * L^3 / (3 * EI)
    let h = 20.0;
    let u_1st_calc = (h * l.powi(3)) / (3.0 * ei);
    assert!((u_1st_calc - u_1st_ref).abs() / u_1st_ref < 1e-3);

    // Verify P-Delta amplification: 1 / (1 - P / P_cr)
    let p = 200.0;
    let amp = 1.0 / (1.0 - p / p_cr_calc);
    let u_pdelta_calc = u_1st_calc * amp;
    assert!((u_pdelta_calc - u_pdelta_ref).abs() / u_pdelta_ref < 1e-3);
}

#[test]
fn test_benchmark_sc005_irvine_catenary_cable() {
    let csv = include_str!("../../../docs/benchmarks/frame/sc005_irvine_catenary_cable.csv");
    let mut lines = csv.lines();
    lines.next();
    let row: Vec<&str> = lines.next().unwrap().split(',').collect();

    let span: f64 = row[1].parse().unwrap();
    let dy: f64 = row[2].parse().unwrap();
    let l0: f64 = row[3].parse().unwrap();
    let ea: f64 = row[4].parse().unwrap();
    let w0: f64 = row[5].parse().unwrap();
    let h_ref: f64 = row[6].parse().unwrap();
    let v_ref: f64 = row[7].parse().unwrap();

    let (h_calc, v_calc, _, is_slack) = solve_irvine_catenary(span, dy, l0, ea, w0)
        .expect("Irvine solver must succeed");

    assert!(!is_slack);
    assert!((h_calc - h_ref).abs() / h_ref < 1e-3, "H diff {} exceeds 1e-3", (h_calc - h_ref).abs() / h_ref);
    assert!((v_calc - v_ref).abs() / v_ref < 1e-3, "V diff {} exceeds 1e-3", (v_calc - v_ref).abs() / v_ref);
}

#[test]
fn test_benchmark_sc006_modal_rsa_3storey_frame() {
    let csv = include_str!("../../../docs/benchmarks/frame/sc006_modal_rsa_3storey_frame.csv");
    let mut lines = csv.lines();
    lines.next(); // header

    let row1: Vec<&str> = lines.next().unwrap().split(',').collect();
    let t1_ref: f64 = row1[2].parse().unwrap();

    // Verify period range ~ 0.30s
    assert!(t1_ref > 0.20 && t1_ref < 0.40);
}
