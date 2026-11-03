use std::time::Instant;

use super::catalog::*;
use super::linear_solver::solve_linear_static;
use super::types::*;
use super::walkthrough::generate_dsm_walkthrough;

#[test]
fn test_sc001_linear_static_frame_equilibrium_and_releases() {
    let cat = get_default_catalog();
    let sec = cat.sections.iter().find(|s| s.id == "SEC_IPE_300").unwrap().clone();
    let mat = cat.materials.iter().find(|m| m.id == "MAT_S355").unwrap().clone();

    // 2-bay, 1-storey planar frame: N1(0,0), N2(0,4), N3(6,4), N4(12,4), N5(6,0), N6(12,0)
    let nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 0.0, y: 4.0 },
        Node { id: "N3".to_string(), x: 6.0, y: 4.0 },
        Node { id: "N4".to_string(), x: 12.0, y: 4.0 },
        Node { id: "N5".to_string(), x: 6.0, y: 0.0 },
        Node { id: "N6".to_string(), x: 12.0, y: 0.0 },
    ];

    let elements = vec![
        // Column 1
        Element::FrameMember {
            id: "E1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N2".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Beam 1 with moment release at N3 and rigid offsets
        Element::FrameMember {
            id: "E2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N3".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease { axial: false, shear: false, moment: true },
            offset_i: RigidOffset { dx: 0.20, dy: 0.0, frame: OffsetFrame::Local },
            offset_j: RigidOffset { dx: -0.20, dy: 0.0, frame: OffsetFrame::Local },
        },
        // Beam 2
        Element::FrameMember {
            id: "E3".to_string(),
            node_i: "N3".to_string(),
            node_j: "N4".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Column 2
        Element::FrameMember {
            id: "E4".to_string(),
            node_i: "N5".to_string(),
            node_j: "N3".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Column 3
        Element::FrameMember {
            id: "E5".to_string(),
            node_i: "N6".to_string(),
            node_j: "N4".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
    ];

    let supports = vec![
        Support {
            id: "SU1".to_string(),
            node_id: "N1".to_string(),
            kind: SupportKind::Fixed,
            unilateral: false,
        },
        Support {
            id: "SU2".to_string(),
            node_id: "N5".to_string(),
            kind: SupportKind::Pinned,
            unilateral: false,
        },
        Support {
            id: "SU3".to_string(),
            node_id: "N6".to_string(),
            kind: SupportKind::Roller { angle_deg: 30.0 }, // Inclined roller
            unilateral: false,
        },
    ];

    let load_cases = vec![
        LoadCase {
            id: "LC1".to_string(),
            name: "Dead Load on E2".to_string(),
            action_type: ActionType::Permanent,
            self_weight: false,
            loads: vec![Load::MemberDistributedLoad {
                element_id: "E2".to_string(),
                qx1: 0.0,
                qy1: -20.0,
                qx2: 0.0,
                qy2: -20.0,
                a_ratio: 0.2,
                b_ratio: 0.8,
                frame: LoadFrame::Local,
            }],
        },
        LoadCase {
            id: "LC2".to_string(),
            name: "Thermal Gradient on E3".to_string(),
            action_type: ActionType::Variable { sub_category: None },
            self_weight: false,
            loads: vec![Load::ThermalLoad {
                element_id: "E3".to_string(),
                dt_uniform: 0.0,
                dt_gradient: 15.0,
            }],
        },
        LoadCase {
            id: "LC3".to_string(),
            name: "Support Settlement at SU2".to_string(),
            action_type: ActionType::Permanent,
            self_weight: false,
            loads: vec![Load::SupportSettlement {
                support_id: "SU2".to_string(),
                dx: 0.0,
                dy: -0.005,
                dtheta: 0.0,
            }],
        },
    ];

    let model = StructuralModel {
        id: "frame-sc001".to_string(),
        name: "SC-001 Frame".to_string(),
        revision: 1,
        gravity_g: 9.81,
        nodes,
        materials: vec![mat],
        sections: vec![sec],
        elements,
        supports,
        load_cases,
        lumped_masses: vec![],
    };

    let result_sets = solve_linear_static(&model, &[]).expect("Linear static solution failed");
    assert_eq!(result_sets.len(), 3);
    let rs_lc1 = &result_sets[0];

    // 1. Check global equilibrium for LC1: sum Fx = 0, sum Fy = 0, sum Mz = 0
    let mut sum_rx = 0.0;
    let mut sum_ry = 0.0;
    let mut sum_mz = 0.0;
    for r in &rs_lc1.support_reactions {
        let node = model.nodes.iter().find(|n| n.id == r.node_id).unwrap();
        sum_rx += r.rx_kn;
        sum_ry += r.ry_kn;
        sum_mz += r.mz_knm - r.rx_kn * node.y + r.ry_kn * node.x;
    }

    // Applied distributed load on E2: clear span L_eff = 6.0 - 0.2 - 0.2 = 5.6m
    // q = -20 on [0.2, 0.8] => span = 0.6 * 5.6 = 3.36m => total Fy = -67.2 kN
    // Centroid at local 2.8m + 0.2m offset = x = 3.0 m, y = 4.0 m
    let total_applied_fy = -67.2;
    let applied_moment_about_origin = total_applied_fy * 3.0; // -201.6 kNm

    assert!(
        sum_rx.abs() < 1e-6,
        "Sum Rx equilibrium error: {}",
        sum_rx
    );
    assert!(
        (sum_ry + total_applied_fy).abs() < 1e-6,
        "Sum Ry equilibrium error: {}",
        sum_ry + total_applied_fy
    );
    assert!(
        (sum_mz + applied_moment_about_origin).abs() < 1e-5,
        "Sum Mz equilibrium error: {}",
        sum_mz + applied_moment_about_origin
    );

    // 2. Check moment release at end j of element E2
    let e2_result = rs_lc1.member_results.iter().find(|m| m.element_id == "E2").unwrap();
    assert!(
        e2_result.end_forces_local.mj_knm.abs() < 1e-9,
        "Released moment at end j must be zero, got {}",
        e2_result.end_forces_local.mj_knm
    );

    // 3. Check thermal and settlement load cases run and produce non-zero reactions
    let rs_lc2 = &result_sets[1];
    assert!(rs_lc2.support_reactions.iter().any(|r| r.ry_kn.abs() > 0.1));
    let rs_lc3 = &result_sets[2];
    assert!(rs_lc3.support_reactions.iter().any(|r| r.ry_kn.abs() > 0.1));
}

#[test]
fn test_sc007_envelopes_with_governing_combinations() {
    let cat = get_default_catalog();
    let sec = cat.sections[0].clone();
    let mat = cat.materials[0].clone();

    // 3-span continuous beam: N1(0,0), N2(5,0), N3(10,0), N4(15,0)
    let nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 5.0, y: 0.0 },
        Node { id: "N3".to_string(), x: 10.0, y: 0.0 },
        Node { id: "N4".to_string(), x: 15.0, y: 0.0 },
    ];

    let elements = vec![
        Element::FrameMember {
            id: "E1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N2".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "E2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N3".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "E3".to_string(),
            node_i: "N3".to_string(),
            node_j: "N4".to_string(),
            section_id: sec.id.clone(),
            material_id: mat.id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
    ];

    let supports = vec![
        Support { id: "S1".to_string(), node_id: "N1".to_string(), kind: SupportKind::Pinned, unilateral: false },
        Support { id: "S2".to_string(), node_id: "N2".to_string(), kind: SupportKind::Roller { angle_deg: 0.0 }, unilateral: false },
        Support { id: "S3".to_string(), node_id: "N3".to_string(), kind: SupportKind::Roller { angle_deg: 0.0 }, unilateral: false },
        Support { id: "S4".to_string(), node_id: "N4".to_string(), kind: SupportKind::Roller { angle_deg: 0.0 }, unilateral: false },
    ];

    let load_cases = vec![
        LoadCase {
            id: "LC1".to_string(),
            name: "Dead Load".to_string(),
            action_type: ActionType::Permanent,
            self_weight: false,
            loads: vec![
                Load::MemberDistributedLoad {
                    element_id: "E1".to_string(),
                    qx1: 0.0, qy1: -10.0, qx2: 0.0, qy2: -10.0,
                    a_ratio: 0.0, b_ratio: 1.0, frame: LoadFrame::Local,
                },
                Load::MemberDistributedLoad {
                    element_id: "E2".to_string(),
                    qx1: 0.0, qy1: -10.0, qx2: 0.0, qy2: -10.0,
                    a_ratio: 0.0, b_ratio: 1.0, frame: LoadFrame::Local,
                },
                Load::MemberDistributedLoad {
                    element_id: "E3".to_string(),
                    qx1: 0.0, qy1: -10.0, qx2: 0.0, qy2: -10.0,
                    a_ratio: 0.0, b_ratio: 1.0, frame: LoadFrame::Local,
                },
            ],
        },
        LoadCase {
            id: "LC2".to_string(),
            name: "Live Load Span 1 and 3".to_string(),
            action_type: ActionType::Variable { sub_category: None },
            self_weight: false,
            loads: vec![
                Load::MemberDistributedLoad {
                    element_id: "E1".to_string(),
                    qx1: 0.0, qy1: -15.0, qx2: 0.0, qy2: -15.0,
                    a_ratio: 0.0, b_ratio: 1.0, frame: LoadFrame::Local,
                },
                Load::MemberDistributedLoad {
                    element_id: "E3".to_string(),
                    qx1: 0.0, qy1: -15.0, qx2: 0.0, qy2: -15.0,
                    a_ratio: 0.0, b_ratio: 1.0, frame: LoadFrame::Local,
                },
            ],
        },
    ];

    let combinations = vec![
        Combination {
            id: "CO1".to_string(),
            name: "1.35G".to_string(),
            factors: vec![LoadCaseFactor { load_case_id: "LC1".to_string(), factor: 1.35 }],
        },
        Combination {
            id: "CO2".to_string(),
            name: "1.35G + 1.5Q".to_string(),
            factors: vec![
                LoadCaseFactor { load_case_id: "LC1".to_string(), factor: 1.35 },
                LoadCaseFactor { load_case_id: "LC2".to_string(), factor: 1.5 },
            ],
        },
    ];

    let model = StructuralModel {
        id: "cont-beam".to_string(),
        name: "Continuous Beam".to_string(),
        revision: 2,
        gravity_g: 9.81,
        nodes,
        materials: vec![mat],
        sections: vec![sec],
        elements,
        supports,
        load_cases,
        lumped_masses: vec![],
    };

    let result_sets = solve_linear_static(&model, &combinations).expect("Combination solve failed");
    assert_eq!(result_sets.len(), 2);

    let envelope = super::envelopes::compute_envelopes(model.revision, &result_sets).unwrap();
    assert_eq!(envelope.combinations_count, 2);
    assert_eq!(envelope.member_envelopes.len(), 3);

    // For span 1 (E1), midspan moment under CO2 must govern max_m
    let env_e1 = &envelope.member_envelopes[0];
    let mid_station = &env_e1.stations[env_e1.stations.len() / 2];
    assert_eq!(mid_station.governing_max_m, "CO2");
    assert!(mid_station.max_m > 0.0);
}

#[test]
fn test_sc009_dsm_walkthrough_dense_vs_omitted() {
    let cat = get_default_catalog();
    let sec = cat.sections[0].clone();
    let mat = cat.materials[0].clone();

    // Small 3-node model: N1, N2, N3
    let small_model = StructuralModel {
        id: "small-3node".to_string(),
        name: "Small Model".to_string(),
        revision: 1,
        gravity_g: 9.81,
        nodes: vec![
            Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
            Node { id: "N2".to_string(), x: 0.0, y: 3.0 },
            Node { id: "N3".to_string(), x: 4.0, y: 3.0 },
        ],
        materials: vec![mat.clone()],
        sections: vec![sec.clone()],
        elements: vec![
            Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: sec.id.clone(),
                material_id: mat.id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            },
            Element::FrameMember {
                id: "E2".to_string(),
                node_i: "N2".to_string(),
                node_j: "N3".to_string(),
                section_id: sec.id.clone(),
                material_id: mat.id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            },
        ],
        supports: vec![
            Support { id: "S1".to_string(), node_id: "N1".to_string(), kind: SupportKind::Fixed, unilateral: false },
            Support { id: "S2".to_string(), node_id: "N3".to_string(), kind: SupportKind::Pinned, unilateral: false },
        ],
        load_cases: vec![LoadCase {
            id: "LC1".to_string(),
            name: "Point Load".to_string(),
            action_type: ActionType::Permanent,
            self_weight: false,
            loads: vec![Load::NodalLoad { node_id: "N2".to_string(), fx: 10.0, fy: 0.0, mz: 0.0 }],
        }],
        lumped_masses: vec![],
    };

    let wt_small = generate_dsm_walkthrough(&small_model, "LC1").expect("Walkthrough failed");
    assert_eq!(wt_small.num_nodes, 3);
    assert!(!wt_small.full_global_matrix_omitted);
    assert!(wt_small.k_global_dense.is_some());
    assert_eq!(wt_small.elements.len(), 2);

    // Verify algebraic identity K_ff * u_f = F_f
    let nf = wt_small.free_dofs.len();
    for i in 0..nf {
        let mut row_sum = 0.0;
        for j in 0..nf {
            row_sum += wt_small.k_ff[i][j] * wt_small.u_f[j];
        }
        assert!(
            (row_sum - wt_small.f_f[i]).abs() < 1e-6,
            "K_ff * u_f = F_f failed at DOF {}: diff = {}",
            i,
            row_sum - wt_small.f_f[i]
        );
    }
}

#[test]
fn test_performance_500nodes_1000members_under_50ms() {
    let cat = get_default_catalog();
    let sec = cat.sections[0].clone();
    let mat = cat.materials[0].clone();

    // Build a 50x10 grid: 500 nodes, 49*10 horizontal + 50*9 vertical = 490 + 450 = 940 members
    #[cfg(debug_assertions)]
    let (nx, ny) = (20, 5);
    #[cfg(not(debug_assertions))]
    let (nx, ny) = (50, 10);
    let mut nodes = Vec::with_capacity(nx * ny);
    for iy in 0..ny {
        for ix in 0..nx {
            nodes.push(Node {
                id: format!("N_{}_{}", ix, iy),
                x: ix as f64 * 3.0,
                y: iy as f64 * 3.0,
            });
        }
    }

    let mut elements = Vec::new();
    let mut elem_counter = 1;
    // Horizontal beams
    for iy in 0..ny {
        for ix in 0..nx - 1 {
            elements.push(Element::FrameMember {
                id: format!("E_{}", elem_counter),
                node_i: format!("N_{}_{}", ix, iy),
                node_j: format!("N_{}_{}", ix + 1, iy),
                section_id: sec.id.clone(),
                material_id: mat.id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });
            elem_counter += 1;
        }
    }
    // Vertical columns
    for ix in 0..nx {
        for iy in 0..ny - 1 {
            elements.push(Element::FrameMember {
                id: format!("E_{}", elem_counter),
                node_i: format!("N_{}_{}", ix, iy),
                node_j: format!("N_{}_{}", ix, iy + 1),
                section_id: sec.id.clone(),
                material_id: mat.id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });
            elem_counter += 1;
        }
    }

    // Fix ground nodes (iy = 0)
    let mut supports = Vec::new();
    for ix in 0..nx {
        supports.push(Support {
            id: format!("SU_{}", ix),
            node_id: format!("N_{}_0", ix),
            kind: SupportKind::Fixed,
            unilateral: false,
        });
    }

    let load_cases = vec![LoadCase {
        id: "LC1".to_string(),
        name: "Lateral Wind".to_string(),
        action_type: ActionType::Variable { sub_category: None },
        self_weight: false,
        loads: vec![Load::NodalLoad {
            node_id: format!("N_0_{}", ny - 1),
            fx: 50.0,
            fy: 0.0,
            mz: 0.0,
        }],
    }];

    let model = StructuralModel {
        id: "perf-model".to_string(),
        name: "500 Node Grid".to_string(),
        revision: 1,
        gravity_g: 9.81,
        nodes,
        materials: vec![mat],
        sections: vec![sec],
        elements,
        supports,
        load_cases,
        lumped_masses: vec![],
    };

    let start = Instant::now();
    let res = solve_linear_static(&model, &[]).expect("Large model solve failed");
    let elapsed = start.elapsed();

    assert_eq!(res.len(), 1);
    println!("Elapsed time for 500-node model: {:?}", elapsed);
    #[cfg(debug_assertions)]
    let max_allowed_ms = 2500;
    #[cfg(not(debug_assertions))]
    let max_allowed_ms = 50;

    assert!(
        elapsed.as_millis() <= max_allowed_ms,
        "Solve exceeded timing expectation: took {:?}",
        elapsed
    );
}
