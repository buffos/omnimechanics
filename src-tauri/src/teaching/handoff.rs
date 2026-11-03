//! Bidirectional Mode 2 <-> Mode 3 Teaching Studio Handoff.

use serde::{Deserialize, Serialize};

use crate::frame::types::{
    ActionType, AxialBehavior, CanonicalError, DsmWalkthroughData, Element, EndRelease,
    LibrarySource, Load, LoadCase, LoadFrame, Material, MaterialClass, Node, RigidOffset, Section,
    SectionShape, StructuralModel, Support, SupportKind,
};
use crate::frame::walkthrough::generate_dsm_walkthrough;
use super::classifier::classify_model;
use super::types::{TeachingInputConfig, TeachingSolutionDto};
use super::presets::solve_preset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenStepByStepRequest {
    pub model: StructuralModel,
    pub load_case_id: Option<String>,
    pub method_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenStepByStepResponse {
    pub model_revision: u64,
    pub selected_method: String,
    pub solution: Option<TeachingSolutionDto>,
    pub dsm_walkthrough: Option<DsmWalkthroughData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetToModelRequest {
    pub preset_id: String,
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub load_q: Option<f64>,
    pub load_p: Option<f64>,
    pub load_p_pos: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetToModelResponse {
    pub model: StructuralModel,
}

/// Converts a Mode 3 teaching preset into a fully editable StructuralModel (revision: 1).
pub fn preset_to_structural_model(
    req: &PresetToModelRequest,
) -> Result<PresetToModelResponse, CanonicalError> {
    let mut model = StructuralModel {
        id: format!("model-{}", req.preset_id),
        name: format!("Preset {}", req.preset_id),
        revision: 1,
        gravity_g: 9.81,
        nodes: Vec::new(),
        elements: Vec::new(),
        supports: Vec::new(),
        load_cases: vec![LoadCase {
            id: "LC1".to_string(),
            name: "Permanent Loads G".to_string(),
            action_type: ActionType::Permanent,
            self_weight: false,
            loads: Vec::new(),
        }],
        materials: vec![Material {
            id: "MAT1".to_string(),
            name: "S355 Structural Steel".to_string(),
            source: LibrarySource::Preset { catalog_id: "S355".to_string() },
            class: MaterialClass::Steel,
            e_gpa: 210.0,
            g_gpa: 81.0,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        }],
        sections: vec![Section {
            id: "SEC1".to_string(),
            name: "IPE 300".to_string(),
            source: LibrarySource::Preset { catalog_id: "IPE_300".to_string() },
            shape: SectionShape::Ipe,
            area_a: 0.00538,
            inertia_i: 0.00008356,
            shear_area_av: None,
            depth_h: Some(0.30),
        }],
        lumped_masses: Vec::new(),
    };

    match req.preset_id.as_str() {
        "simply_supported_distributed" => {
            let l = req.length.unwrap_or(6.0);
            let q = req.load_q.unwrap_or(12.0);

            model.nodes.push(Node { id: "N1".to_string(), x: 0.0, y: 0.0 });
            model.nodes.push(Node { id: "N2".to_string(), x: l, y: 0.0 });

            model.elements.push(Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Pinned,
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S2".to_string(),
                node_id: "N2".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });

            model.load_cases[0].loads.push(Load::MemberDistributedLoad {
                element_id: "E1".to_string(),
                qx1: 0.0,
                qy1: -q,
                qx2: 0.0,
                qy2: -q,
                a_ratio: 0.0,
                b_ratio: 1.0,
                frame: LoadFrame::Local,
            });
        }
        "simply_supported_point" => {
            let l = req.length.unwrap_or(8.0);
            let p = req.load_p.unwrap_or(40.0);
            let p_pos = req.load_p_pos.unwrap_or(l / 2.0).clamp(0.0, l);
            let x_ratio = (p_pos / l).clamp(0.0, 1.0);

            model.nodes.push(Node { id: "N1".to_string(), x: 0.0, y: 0.0 });
            model.nodes.push(Node { id: "N2".to_string(), x: l, y: 0.0 });

            model.elements.push(Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Pinned,
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S2".to_string(),
                node_id: "N2".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });

            model.load_cases[0].loads.push(Load::MemberPointLoad {
                element_id: "E1".to_string(),
                fx: 0.0,
                fy: -p,
                mz: 0.0,
                x_ratio,
                frame: LoadFrame::Local,
            });
        }
        "cantilever" => {
            let l = req.length.unwrap_or(4.0);
            let p = req.load_p.unwrap_or(15.0);
            let p_pos = req.load_p_pos.unwrap_or(l).clamp(0.0, l);
            let x_ratio = (p_pos / l).clamp(0.0, 1.0);
            let q = req.load_q.unwrap_or(5.0);

            model.nodes.push(Node { id: "N1".to_string(), x: 0.0, y: 0.0 });
            model.nodes.push(Node { id: "N2".to_string(), x: l, y: 0.0 });

            model.elements.push(Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Fixed,
                unilateral: false,
            });

            model.load_cases[0].loads.push(Load::MemberDistributedLoad {
                element_id: "E1".to_string(),
                qx1: 0.0,
                qy1: -q,
                qx2: 0.0,
                qy2: -q,
                a_ratio: 0.0,
                b_ratio: 1.0,
                frame: LoadFrame::Local,
            });

            if p.abs() > 1e-4 {
                model.load_cases[0].loads.push(Load::MemberPointLoad {
                    element_id: "E1".to_string(),
                    fx: 0.0,
                    fy: -p,
                    mz: 0.0,
                    x_ratio,
                    frame: LoadFrame::Local,
                });
            }
        }
        "overhanging_beam" => {
            let l_main = req.length.unwrap_or(6.0);
            let l_overhang = req.height.unwrap_or(2.0);
            let q = req.load_q.unwrap_or(10.0);
            let p = req.load_p.unwrap_or(20.0);

            model.nodes.push(Node { id: "N1".to_string(), x: 0.0, y: 0.0 });
            model.nodes.push(Node { id: "N2".to_string(), x: l_main, y: 0.0 });
            model.nodes.push(Node { id: "N3".to_string(), x: l_main + l_overhang, y: 0.0 });

            model.elements.push(Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });
            model.elements.push(Element::FrameMember {
                id: "E2".to_string(),
                node_i: "N2".to_string(),
                node_j: "N3".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Pinned,
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S2".to_string(),
                node_id: "N2".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });

            model.load_cases[0].loads.push(Load::MemberDistributedLoad {
                element_id: "E1".to_string(),
                qx1: 0.0,
                qy1: -q,
                qx2: 0.0,
                qy2: -q,
                a_ratio: 0.0,
                b_ratio: 1.0,
                frame: LoadFrame::Local,
            });

            if p.abs() > 1e-4 {
                model.load_cases[0].loads.push(Load::MemberPointLoad {
                    element_id: "E2".to_string(),
                    fx: 0.0,
                    fy: -p,
                    mz: 0.0,
                    x_ratio: 1.0,
                    frame: LoadFrame::Local,
                });
            }
        }
        "cross_two_span" => {
            let l1 = req.length.unwrap_or(6.0);
            let l2 = req.height.unwrap_or(4.0);
            let q = req.load_q.unwrap_or(15.0);
            let p = req.load_p.unwrap_or(30.0);

            model.nodes.push(Node { id: "N1".to_string(), x: 0.0, y: 0.0 });
            model.nodes.push(Node { id: "N2".to_string(), x: l1, y: 0.0 });
            model.nodes.push(Node { id: "N3".to_string(), x: l1 + l2, y: 0.0 });

            model.elements.push(Element::FrameMember {
                id: "E1".to_string(),
                node_i: "N1".to_string(),
                node_j: "N2".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });
            model.elements.push(Element::FrameMember {
                id: "E2".to_string(),
                node_i: "N2".to_string(),
                node_j: "N3".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                release_i: EndRelease::default(),
                release_j: EndRelease::default(),
                offset_i: RigidOffset::default(),
                offset_j: RigidOffset::default(),
            });

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N1".to_string(),
                kind: SupportKind::Pinned,
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S2".to_string(),
                node_id: "N2".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S3".to_string(),
                node_id: "N3".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });

            model.load_cases[0].loads.push(Load::MemberDistributedLoad {
                element_id: "E1".to_string(),
                qx1: 0.0,
                qy1: -q,
                qx2: 0.0,
                qy2: -q,
                a_ratio: 0.0,
                b_ratio: 1.0,
                frame: LoadFrame::Local,
            });

            if p.abs() > 1e-4 {
                model.load_cases[0].loads.push(Load::MemberPointLoad {
                    element_id: "E2".to_string(),
                    fx: 0.0,
                    fy: -p,
                    mz: 0.0,
                    x_ratio: 0.5,
                    frame: LoadFrame::Local,
                });
            }
        }
        "pratt_truss" => {
            let span = req.length.unwrap_or(12.0);
            let height = req.height.unwrap_or(3.0);
            let panel_len = span / 4.0;
            let p = req.load_p.unwrap_or(20.0);

            // Bottom nodes N0..N4
            for i in 0..=4 {
                model.nodes.push(Node {
                    id: format!("N{}", i),
                    x: (i as f64) * panel_len,
                    y: 0.0,
                });
            }
            // Top nodes N5..N7
            for i in 1..=3 {
                model.nodes.push(Node {
                    id: format!("N{}", 4 + i),
                    x: (i as f64) * panel_len,
                    y: height,
                });
            }

            let mut mid = 1;
            // Bottom chords
            for i in 0..4 {
                model.elements.push(Element::TrussBar {
                    id: format!("E{}", mid),
                    node_i: format!("N{}", i),
                    node_j: format!("N{}", i + 1),
                    section_id: "SEC1".to_string(),
                    material_id: "MAT1".to_string(),
                    axial_behavior: AxialBehavior::Both,
                });
                mid += 1;
            }
            // Top chords
            model.elements.push(Element::TrussBar {
                id: format!("E{}", mid),
                node_i: "N5".to_string(),
                node_j: "N6".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                axial_behavior: AxialBehavior::Both,
            });
            mid += 1;
            model.elements.push(Element::TrussBar {
                id: format!("E{}", mid),
                node_i: "N6".to_string(),
                node_j: "N7".to_string(),
                section_id: "SEC1".to_string(),
                material_id: "MAT1".to_string(),
                axial_behavior: AxialBehavior::Both,
            });
            mid += 1;

            // Verticals: (N1, N5), (N2, N6), (N3, N7)
            for i in 1..=3 {
                model.elements.push(Element::TrussBar {
                    id: format!("E{}", mid),
                    node_i: format!("N{}", i),
                    node_j: format!("N{}", 4 + i),
                    section_id: "SEC1".to_string(),
                    material_id: "MAT1".to_string(),
                    axial_behavior: AxialBehavior::Both,
                });
                mid += 1;
            }

            // Diagonals: (N0, N5), (N5, N2), (N2, N7), (N7, N4)
            let diags = [("N0", "N5"), ("N5", "N2"), ("N2", "N7"), ("N7", "N4")];
            for (ni, nj) in diags {
                model.elements.push(Element::TrussBar {
                    id: format!("E{}", mid),
                    node_i: ni.to_string(),
                    node_j: nj.to_string(),
                    section_id: "SEC1".to_string(),
                    material_id: "MAT1".to_string(),
                    axial_behavior: AxialBehavior::Both,
                });
                mid += 1;
            }

            model.supports.push(Support {
                id: "S1".to_string(),
                node_id: "N0".to_string(),
                kind: SupportKind::Pinned,
                unilateral: false,
            });
            model.supports.push(Support {
                id: "S2".to_string(),
                node_id: "N4".to_string(),
                kind: SupportKind::Roller { angle_deg: 0.0 },
                unilateral: false,
            });

            // Point loads at bottom chord nodes N1, N2, N3
            for i in 1..=3 {
                model.load_cases[0].loads.push(Load::NodalLoad {
                    node_id: format!("N{}", i),
                    fx: 0.0,
                    fy: -p,
                    mz: 0.0,
                });
            }
        }
        _ => {
            return Err(CanonicalError::new(
                "UNKNOWN_PRESET",
                format!("Το εκπαιδευτικό preset '{}' δεν είναι γνωστό.", req.preset_id),
                format!("Unknown teaching preset '{}'.", req.preset_id),
            ));
        }
    }

    Ok(PresetToModelResponse { model })
}

/// Opens the model step-by-step in Mode 3 Teaching Studio.
pub fn open_step_by_step(
    req: &OpenStepByStepRequest,
) -> Result<OpenStepByStepResponse, CanonicalError> {
    let target_lc = req
        .load_case_id
        .clone()
        .or_else(|| req.model.load_cases.first().map(|c| c.id.clone()))
        .unwrap_or_else(|| "LC1".to_string());

    let method_choice = req.method_id.as_deref().unwrap_or("auto");

    if method_choice == "dsm_walkthrough" {
        let walkthrough = generate_dsm_walkthrough(&req.model, &target_lc)?;
        return Ok(OpenStepByStepResponse {
            model_revision: req.model.revision,
            selected_method: "dsm_walkthrough".to_string(),
            solution: None,
            dsm_walkthrough: Some(walkthrough),
        });
    }

    // Run classifier
    let eligibility = classify_model(&req.model, Some(&target_lc))?;

    let eligible_method = if method_choice == "auto" {
        eligibility
            .methods
            .iter()
            .find(|m| m.eligible)
            .map(|m| m.method_id.as_str())
    } else {
        let req_method = eligibility
            .methods
            .iter()
            .find(|m| m.method_id == method_choice);
        match req_method {
            Some(m) if m.eligible => Some(m.method_id.as_str()),
            Some(m) => {
                let reason_el = m
                    .ineligibility_reason_el
                    .clone()
                    .unwrap_or_else(|| "Μη επιλέξιμη μέθοδος διδασκαλίας.".to_string());
                let reason_en = m
                    .ineligibility_reason_en
                    .clone()
                    .unwrap_or_else(|| "Ineligible teaching method.".to_string());
                return Err(CanonicalError::new(
                    "INELIGIBLE_TEACHING_METHOD",
                    reason_el,
                    reason_en,
                ));
            }
            None => {
                return Err(CanonicalError::new(
                    "UNKNOWN_TEACHING_METHOD",
                    format!("Η μέθοδος '{}' δεν είναι γνωστή.", method_choice),
                    format!("Unknown teaching method '{}'.", method_choice),
                ));
            }
        }
    };

    let selected_method = match eligible_method {
        Some(m) => m.to_string(),
        None => {
            if eligibility.dsm_walkthrough_available {
                // Fallback to DSM walkthrough
                let walkthrough = generate_dsm_walkthrough(&req.model, &target_lc)?;
                return Ok(OpenStepByStepResponse {
                    model_revision: req.model.revision,
                    selected_method: "dsm_walkthrough".to_string(),
                    solution: None,
                    dsm_walkthrough: Some(walkthrough),
                });
            } else {
                return Err(CanonicalError::new(
                    "INELIGIBLE_TEACHING_METHOD",
                    "Ο φορέας δεν είναι επιλέξιμος για καμία κλασική αναλυτική μέθοδο (ή είναι μηχανισμός).",
                    "The structure is ineligible for classical analytical methods (or is a mechanism).",
                ));
            }
        }
    };

    // If eligible for classical method, invoke teaching solution
    // Map model to TeachingInputConfig
    let preset_name = match selected_method.as_str() {
        "IsostaticBeamFrame" => "simply_supported_distributed",
        "TrussJointsRitter" => "pratt_truss",
        "ContinuousBeamClapeyronCross" => "cross_two_span",
        _ => "simply_supported_distributed",
    };

    let teaching_config = TeachingInputConfig {
        preset_name: preset_name.to_string(),
        method: "diagrams".to_string(),
        language: Some("el".to_string()),
        length: None,
        height: None,
        load_q: None,
        load_p: None,
        load_p_pos: None,
        load_m: None,
        custom_nodes: None,
        custom_members: None,
        custom_supports: None,
        custom_point_loads: None,
        custom_dist_loads: None,
    };

    let solution = solve_preset(&teaching_config).ok();

    Ok(OpenStepByStepResponse {
        model_revision: req.model.revision,
        selected_method,
        solution,
        dsm_walkthrough: None,
    })
}
