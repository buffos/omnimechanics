use super::catalog::*;
use super::types::*;
use super::validation::validate_model;

fn create_valid_model() -> StructuralModel {
    let cat = get_default_catalog();
    StructuralModel {
        id: "test-model".to_string(),
        name: "Test Model".to_string(),
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
                x: 4.0,
                y: 0.0,
            },
        ],
        materials: vec![cat.materials[0].clone()],
        sections: vec![cat.sections[0].clone()],
        elements: vec![Element::FrameMember {
            id: "E1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N2".to_string(),
            section_id: cat.sections[0].id.clone(),
            material_id: cat.materials[0].id.clone(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        }],
        supports: vec![Support {
            id: "S1".to_string(),
            node_id: "N1".to_string(),
            kind: SupportKind::Pinned,
            unilateral: false,
        }],
        load_cases: vec![],
        lumped_masses: vec![],
    }
}

#[test]
fn test_valid_model_passes() {
    let model = create_valid_model();
    assert!(validate_model(&model).is_ok());
}

#[test]
fn test_missing_node_reference() {
    let mut model = create_valid_model();
    if let Element::FrameMember { node_i, .. } = &mut model.elements[0] {
        *node_i = "UNKNOWN_NODE".to_string();
    }
    let err = validate_model(&model).unwrap_err();
    assert_eq!(err.code, "MISSING_MODEL_REFERENCE");
}

#[test]
fn test_duplicate_support_on_same_node() {
    let mut model = create_valid_model();
    model.supports.push(Support {
        id: "S2".to_string(),
        node_id: "N1".to_string(),
        kind: SupportKind::Roller { angle_deg: 0.0 },
        unilateral: false,
    });
    let err = validate_model(&model).unwrap_err();
    assert_eq!(err.code, "DUPLICATE_NODE_SUPPORT");
}

#[test]
fn test_zero_length_element() {
    let mut model = create_valid_model();
    if let Element::FrameMember { node_j, .. } = &mut model.elements[0] {
        *node_j = "N1".to_string();
    }
    let err = validate_model(&model).unwrap_err();
    assert_eq!(err.code, "INVALID_ELEMENT_GEOMETRY");
}

#[test]
fn test_excessive_rigid_offsets() {
    let mut model = create_valid_model();
    if let Element::FrameMember {
        offset_i, offset_j, ..
    } = &mut model.elements[0]
    {
        offset_i.dx = 2.0;
        offset_j.dx = 2.0; // 2.0 + 2.0 = 4.0 >= 0.95 * 4.0 = 3.8
    }
    let err = validate_model(&model).unwrap_err();
    assert_eq!(err.code, "INVALID_ELEMENT_GEOMETRY");
}

#[test]
fn test_transverse_load_on_cable_rejected() {
    let mut model = create_valid_model();
    model.elements.push(Element::Cable {
        id: "C1".to_string(),
        node_i: "N1".to_string(),
        node_j: "N2".to_string(),
        section_id: model.sections[0].id.clone(),
        material_id: model.materials[0].id.clone(),
        prestress: CablePrestress::Pretension { t0: 10.0 },
    });
    model.load_cases.push(LoadCase {
        id: "LC1".to_string(),
        name: "Test LC".to_string(),
        action_type: ActionType::Permanent,
        self_weight: false,
        loads: vec![Load::MemberDistributedLoad {
            element_id: "C1".to_string(),
            qx1: 0.0,
            qy1: -10.0,
            qx2: 0.0,
            qy2: -10.0,
            a_ratio: 0.0,
            b_ratio: 1.0,
            frame: LoadFrame::Global,
        }],
    });
    let err = validate_model(&model).unwrap_err();
    assert_eq!(err.code, "INVALID_LOAD_ASSIGNMENT");
}
