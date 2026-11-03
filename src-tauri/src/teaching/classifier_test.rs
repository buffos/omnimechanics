#[cfg(test)]
mod tests {
    use crate::teaching::classifier::*;
    use crate::teaching::handoff::*;
    use crate::teaching::persistence::*;
    use crate::frame::types::*;

    #[test]
    fn test_classify_simply_supported_beam() {
        let req = PresetToModelRequest {
            preset_id: "simply_supported_distributed".to_string(),
            length: Some(6.0),
            height: None,
            load_q: Some(12.0),
            load_p: None,
            load_p_pos: None,
        };
        let res = preset_to_structural_model(&req).expect("preset conversion failed");
        assert_eq!(res.model.revision, 1);
        assert_eq!(res.model.nodes.len(), 2);
        assert_eq!(res.model.elements.len(), 1);

        let eligibility = classify_model(&res.model, None).expect("classification failed");
        assert!(eligibility.is_stable);
        assert_eq!(eligibility.indeterminacy_degree, 0);
        assert!(eligibility.overall_eligible);
        assert!(eligibility.dsm_walkthrough_available);

        let isostatic = eligibility.methods.iter().find(|m| m.method_id == "IsostaticBeamFrame").unwrap();
        assert!(isostatic.eligible);

        let force_method = eligibility.methods.iter().find(|m| m.method_id == "ForceMethod").unwrap();
        assert!(!force_method.eligible);
    }

    #[test]
    fn test_classify_cable_bearing_structure_ineligible_for_classical() {
        let model = StructuralModel {
            id: "model-cable".to_string(),
            name: "Cable Model".to_string(),
            revision: 5,
            gravity_g: 9.81,
            nodes: vec![
                Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
                Node { id: "N2".to_string(), x: 6.0, y: 0.0 },
            ],
            materials: vec![Material {
                id: "MAT1".to_string(),
                name: "Steel".to_string(),
                source: LibrarySource::Custom,
                class: MaterialClass::Steel,
                e_gpa: 210.0,
                g_gpa: 81.0,
                density_gamma: 78.5,
                alpha_t: 1.2e-5,
            }],
            sections: vec![
                Section {
                    id: "SEC1".to_string(),
                    name: "BeamSection".to_string(),
                    source: LibrarySource::Custom,
                    shape: SectionShape::Ipe,
                    area_a: 0.005,
                    inertia_i: 8e-5,
                    shear_area_av: None,
                    depth_h: Some(0.30),
                },
                Section {
                    id: "SEC2".to_string(),
                    name: "CableSection".to_string(),
                    source: LibrarySource::Custom,
                    shape: SectionShape::Generic,
                    area_a: 0.001,
                    inertia_i: 1e-8,
                    shear_area_av: None,
                    depth_h: Some(0.02),
                },
            ],
            elements: vec![
                Element::FrameMember {
                    id: "E1".to_string(),
                    node_i: "N1".to_string(),
                    node_j: "N2".to_string(),
                    section_id: "SEC1".to_string(),
                    material_id: "MAT1".to_string(),
                    release_i: EndRelease::default(),
                    release_j: EndRelease::default(),
                    offset_i: RigidOffset::default(),
                    offset_j: RigidOffset::default(),
                },
                Element::Cable {
                    id: "E2".to_string(),
                    node_i: "N1".to_string(),
                    node_j: "N2".to_string(),
                    section_id: "SEC2".to_string(),
                    material_id: "MAT1".to_string(),
                    prestress: CablePrestress::Pretension { t0: 50.0 },
                },
            ],
            supports: vec![
                Support {
                    id: "S1".to_string(),
                    node_id: "N1".to_string(),
                    kind: SupportKind::Fixed,
                    unilateral: false,
                },
                Support {
                    id: "S2".to_string(),
                    node_id: "N2".to_string(),
                    kind: SupportKind::Fixed,
                    unilateral: false,
                },
            ],
            load_cases: vec![LoadCase {
                id: "LC1".to_string(),
                name: "G".to_string(),
                action_type: ActionType::Permanent,
                self_weight: false,
                loads: vec![],
            }],
            lumped_masses: vec![],
        };

        let eligibility = classify_model(&model, None).expect("classification failed");
        assert!(eligibility.is_stable);
        assert!(!eligibility.overall_eligible);
        assert!(eligibility.dsm_walkthrough_available);
        for m in &eligibility.methods {
            assert!(!m.eligible);
            assert!(m.ineligibility_reason_el.as_ref().unwrap().contains("καλωδίων"));
        }

        // Attempting to open step by step fails or falls back to DSM walkthrough
        let step_req = OpenStepByStepRequest {
            model: model.clone(),
            load_case_id: None,
            method_id: Some("IsostaticBeamFrame".to_string()),
        };
        let step_res = open_step_by_step(&step_req);
        assert!(step_res.is_err());
        assert_eq!(step_res.err().unwrap().code, "INELIGIBLE_TEACHING_METHOD");

        // Requesting DSM walkthrough succeeds
        let dsm_req = OpenStepByStepRequest {
            model,
            load_case_id: None,
            method_id: Some("dsm_walkthrough".to_string()),
        };
        let dsm_res = open_step_by_step(&dsm_req).expect("dsm walkthrough failed");
        assert_eq!(dsm_res.selected_method, "dsm_walkthrough");
        assert!(dsm_res.dsm_walkthrough.is_some());
    }

    #[test]
    fn test_classify_continuous_two_span_beam() {
        let req = PresetToModelRequest {
            preset_id: "cross_two_span".to_string(),
            length: Some(6.0),
            height: Some(4.0),
            load_q: Some(15.0),
            load_p: Some(30.0),
            load_p_pos: None,
        };
        let res = preset_to_structural_model(&req).expect("preset conversion failed");
        let eligibility = classify_model(&res.model, None).expect("classification failed");

        assert!(eligibility.is_stable);
        assert_eq!(eligibility.indeterminacy_degree, 1);
        assert!(eligibility.overall_eligible);

        let cross = eligibility.methods.iter().find(|m| m.method_id == "ContinuousBeamClapeyronCross").unwrap();
        assert!(cross.eligible);

        let force = eligibility.methods.iter().find(|m| m.method_id == "ForceMethod").unwrap();
        assert!(force.eligible);
    }

    #[test]
    fn test_persistence_round_trip() {
        let req = PresetToModelRequest {
            preset_id: "simply_supported_point".to_string(),
            length: Some(8.0),
            height: None,
            load_q: None,
            load_p: Some(40.0),
            load_p_pos: Some(4.0),
        };
        let res = preset_to_structural_model(&req).expect("preset conversion failed");

        let doc = ProjectDocument {
            version: "2.0.0".to_string(),
            structural: res.model,
            combinations: vec![],
            seismic_settings: None,
            viewport: Some(serde_json::json!({ "zoom": 1.5, "panX": 100, "panY": 200 })),
        };

        let save_req = SaveWorkspaceRequest {
            path: None,
            document: doc.clone(),
        };
        let json_str = save_workspace(&save_req).expect("save failed");
        assert!(json_str.contains("\"version\": \"2.0.0\""));

        let load_req = LoadWorkspaceRequest {
            path_or_json: json_str,
        };
        let loaded_doc = load_workspace(&load_req).expect("load failed");
        assert_eq!(loaded_doc.version, doc.version);
        assert_eq!(loaded_doc.structural.nodes.len(), doc.structural.nodes.len());
        assert_eq!(loaded_doc.structural.elements.len(), doc.structural.elements.len());
    }
}
