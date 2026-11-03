use super::catalog::*;
use super::diagnostics::evaluate_diagnostics;
use super::types::*;

fn create_base_portal_frame() -> StructuralModel {
    let cat = get_default_catalog();
    StructuralModel {
        id: "portal-frame".to_string(),
        name: "Portal Frame".to_string(),
        revision: 1,
        gravity_g: 9.81,
        nodes: vec![
            Node {
                id: "N1".to_string(),
                x: 0.0,
                y: 0.0,
            },
            Node {
                id: "N2".to_string(),
                x: 0.0,
                y: 4.0,
            },
            Node {
                id: "N3".to_string(),
                x: 6.0,
                y: 4.0,
            },
            Node {
                id: "N4".to_string(),
                x: 6.0,
                y: 0.0,
            },
        ],
        materials: vec![cat.materials[0].clone()],
        sections: vec![cat.sections[0].clone()],
        elements: vec![
            Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: cat.sections[0].id.clone(),
                material_id: cat.materials[0].id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            },
            Element::FrameMember {
                id: "E2".to_string(),
                node_i: "N2".to_string(),
                node_j: "N3".to_string(),
                section_id: cat.sections[0].id.clone(),
                material_id: cat.materials[0].id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            },
            Element::FrameMember {
                id: "E3".to_string(),
                node_i: "N4".to_string(),
                node_j: "N3".to_string(),
                section_id: cat.sections[0].id.clone(),
                material_id: cat.materials[0].id.clone(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            },
        ],
        supports: vec![
            Support {
                id: "SU1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Fixed,
                unilateral: false,
            },
            Support {
                id: "SU2".to_string(),
                node_id: "N4".to_string(),
                kind: SupportKind::Fixed,
                unilateral: false,
            },
        ],
        load_cases: vec![],
        lumped_masses: vec![],
    }
}

#[test]
fn test_stable_fixed_portal_frame() {
    let model = create_base_portal_frame();
    let diag = evaluate_diagnostics(&model).expect("Diagnostics evaluation failed");
    assert!(diag.is_stable);
    assert_eq!(diag.kinematic_instability_degree, 0);
    // Indeterminacy: R = 6, B = 9, 3N = 12, H = 0 => 6 + 9 - 12 = 3
    assert_eq!(diag.static_indeterminacy_degree, 3);
    assert!(diag.mechanism_modes.is_empty());
}

#[test]
fn test_sc002_4hinge_portal_sway_mechanism() {
    let mut model = create_base_portal_frame();
    // Change supports to pinned (R = 2 + 2 = 4)
    model.supports[0].kind = SupportKind::Pinned;
    model.supports[1].kind = SupportKind::Pinned;

    // Add moment releases to both ends of beam E2
    if let Element::FrameMember {
        release_i, release_j, ..
    } = &mut model.elements[1]
    {
        release_i.moment = true;
        release_j.moment = true;
    }

    let diag = evaluate_diagnostics(&model).expect("Diagnostics evaluation failed");
    assert!(!diag.is_stable);
    assert_eq!(diag.kinematic_instability_degree, 1);
    assert_eq!(diag.static_indeterminacy_degree, -1);
    assert_eq!(diag.mechanism_modes.len(), 1);

    let mode = &diag.mechanism_modes[0];
    assert_eq!(
        mode.classification,
        MechanismClassification::GlobalSwayOrHingeChain
    );

    // Verify normalization max(|dux|, |duy|) == 1.0
    let mut max_trans = 0.0f64;
    for disp in &mode.node_displacements {
        max_trans = max_trans.max(disp.dux.abs()).max(disp.duy.abs());
    }
    assert!((max_trans - 1.0).abs() < 1e-6);

    // Nodes N2 and N3 sway horizontally
    let disp_n2 = mode.node_displacements.iter().find(|d| d.node_id == "N2").unwrap();
    let disp_n3 = mode.node_displacements.iter().find(|d| d.node_id == "N3").unwrap();
    assert!(disp_n2.dux.abs() > 0.5);
    assert!(disp_n3.dux.abs() > 0.5);
    // Node N1 and N4 are pinned (no displacement)
    let disp_n1 = mode.node_displacements.iter().find(|d| d.node_id == "N1").unwrap();
    let disp_n4 = mode.node_displacements.iter().find(|d| d.node_id == "N4").unwrap();
    assert!(disp_n1.dux.abs() < 1e-6);
    assert!(disp_n4.dux.abs() < 1e-6);
}
