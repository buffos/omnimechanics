use super::types::*;

#[test]
fn test_canonical_json_roundtrip() {
    let json_sample = r#"{
  "id": "model-01",
  "name": "Two-Bay Portal Frame with Tie Cable",
  "revision": 12,
  "gravity_g": 9.81,
  "nodes": [
    { "id": "N1", "x": 0.0, "y": 0.0 },
    { "id": "N2", "x": 0.0, "y": 4.0 },
    { "id": "N3", "x": 6.0, "y": 4.0 },
    { "id": "N4", "x": 6.0, "y": 0.0 }
  ],
  "materials": [
    {
      "id": "M1",
      "name": "Steel S355",
      "source": { "kind": "Preset", "catalog_id": "EN1993_S355" },
      "class": "Steel",
      "e_gpa": 210.0,
      "g_gpa": 80.77,
      "density_gamma": 78.5,
      "alpha_t": 0.000012
    }
  ],
  "sections": [
    {
      "id": "S1",
      "name": "IPE 300",
      "source": { "kind": "Preset", "catalog_id": "IPE_300" },
      "shape": "Ipe",
      "area_a": 0.00538,
      "inertia_i": 0.00008356,
      "shear_area_av": 0.00257,
      "depth_h": 0.300
    }
  ],
  "elements": [
    {
      "id": "E1",
      "kind": "FrameMember",
      "node_i": "N1",
      "node_j": "N2",
      "section_id": "S1",
      "material_id": "M1",
      "release_i": { "axial": false, "shear": false, "moment": false },
      "release_j": { "axial": false, "shear": false, "moment": false },
      "offset_i": { "dx": 0.0, "dy": 0.0, "frame": "Local" },
      "offset_j": { "dx": 0.0, "dy": 0.0, "frame": "Local" }
    },
    {
      "id": "E2",
      "kind": "FrameMember",
      "node_i": "N2",
      "node_j": "N3",
      "section_id": "S1",
      "material_id": "M1",
      "release_i": { "axial": false, "shear": false, "moment": false },
      "release_j": { "axial": false, "shear": false, "moment": true },
      "offset_i": { "dx": 0.15, "dy": 0.0, "frame": "Local" },
      "offset_j": { "dx": -0.15, "dy": 0.0, "frame": "Local" }
    },
    {
      "id": "E3",
      "kind": "Cable",
      "node_i": "N1",
      "node_j": "N3",
      "section_id": "S1",
      "material_id": "M1",
      "prestress": { "mode": "Pretension", "t0": 25.0 }
    }
  ],
  "supports": [
    { "id": "SU1", "node_id": "N1", "kind": { "type": "Fixed" }, "unilateral": false },
    { "id": "SU2", "node_id": "N4", "kind": { "type": "Roller", "angle_deg": 15.0 }, "unilateral": true }
  ],
  "load_cases": [
    {
      "id": "LC1",
      "name": "Dead Load G",
      "action_type": { "kind": "Permanent" },
      "self_weight": true,
      "loads": [
        {
          "kind": "MemberDistributedLoad",
          "element_id": "E2",
          "qx1": 0.0,
          "qy1": -18.0,
          "qx2": 0.0,
          "qy2": -24.0,
          "a_ratio": 0.0,
          "b_ratio": 1.0,
          "frame": "Local"
        },
        {
          "kind": "ThermalLoad",
          "element_id": "E2",
          "dt_uniform": 20.0,
          "dt_gradient": 10.0
        }
      ]
    }
  ],
  "lumped_masses": [
    { "node_id": "N2", "mass_x_t": 12.5, "mass_y_t": 12.5, "inertia_theta_tm2": 0.0 }
  ]
}"#;

    let model: StructuralModel = serde_json::from_str(json_sample).expect("Failed to deserialize model");
    assert_eq!(model.id, "model-01");
    assert_eq!(model.revision, 12);
    assert_eq!(model.nodes.len(), 4);
    assert_eq!(model.materials.len(), 1);
    assert_eq!(model.sections.len(), 1);
    assert_eq!(model.elements.len(), 3);
    assert_eq!(model.supports.len(), 2);
    assert_eq!(model.load_cases.len(), 1);
    assert_eq!(model.lumped_masses.len(), 1);

    // Verify cable prestress variant
    if let Element::Cable { prestress, .. } = &model.elements[2] {
        assert_eq!(*prestress, CablePrestress::Pretension { t0: 25.0 });
    } else {
        panic!("Expected Element::Cable");
    }

    // Round-trip check
    let serialized = serde_json::to_string(&model).expect("Failed to serialize model");
    let model_back: StructuralModel =
        serde_json::from_str(&serialized).expect("Failed to deserialize serialized model");
    assert_eq!(model, model_back);
}
