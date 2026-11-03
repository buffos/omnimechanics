//! Structural Method Classifier for Mode 2 -> Mode 3 Handoff.
//! Evaluates eligibility of a StructuralModel for classical teaching methods.

use serde::{Deserialize, Serialize};
use crate::frame::diagnostics::evaluate_diagnostics;
use crate::frame::types::{CanonicalError, Element, StructuralModel};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodStatusDto {
    pub method_id: String,
    pub name_el: String,
    pub name_en: String,
    pub eligible: bool,
    pub ineligibility_reason_el: Option<String>,
    pub ineligibility_reason_en: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodEligibility {
    pub model_revision: u64,
    pub methods: Vec<MethodStatusDto>,
    pub dsm_walkthrough_available: bool,
    pub indeterminacy_degree: i32,
    pub is_stable: bool,
    pub overall_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyModelRequest {
    pub model: StructuralModel,
    pub load_case_id: Option<String>,
}

/// Classifies a structural model to evaluate eligibility for classical pedagogical methods.
pub fn classify_model(
    model: &StructuralModel,
    _load_case_id: Option<&str>,
) -> Result<MethodEligibility, CanonicalError> {
    crate::frame::validation::validate_model(model)?;

    // 1. Check for cables (BR-5)
    let has_cables = model
        .elements
        .iter()
        .any(|e| matches!(e, Element::Cable { .. }));

    // 2. Check for unilateral supports
    let has_unilateral = model.supports.iter().any(|s| s.unilateral);

    // 3. Evaluate kinematic diagnostics
    let diag = evaluate_diagnostics(model)?;
    let is_stable = diag.is_stable;
    let ns = diag.static_indeterminacy_degree;

    if !is_stable {
        let reason_el = "Ο φορέας είναι κινηματικά ασταθής (μηχανισμός).".to_string();
        let reason_en = "The structure is kinematically unstable (mechanism).".to_string();
        let methods = vec![
            MethodStatusDto {
                method_id: "IsostaticBeamFrame".to_string(),
                name_el: "Ισοστατικοί Φορείς & Δοκοί".to_string(),
                name_en: "Isostatic Beams & Frames".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "TrussJointsRitter".to_string(),
                name_el: "Δικτυώματα - Μέθοδος Κόμβων & Ritter".to_string(),
                name_en: "Trusses - Method of Joints & Ritter".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ContinuousBeamClapeyronCross".to_string(),
                name_el: "Συνεχείς Δοκοί - Clapeyron & Cross".to_string(),
                name_en: "Continuous Beams - Clapeyron & Cross".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ForceMethod".to_string(),
                name_el: "Μέθοδος Δυνάμεων".to_string(),
                name_en: "Force Method".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el),
                ineligibility_reason_en: Some(reason_en),
            },
        ];

        return Ok(MethodEligibility {
            model_revision: model.revision,
            methods,
            dsm_walkthrough_available: false,
            indeterminacy_degree: ns,
            is_stable: false,
            overall_eligible: false,
        });
    }

    // If cables are present: BR-5 unconditionally disables classical methods
    if has_cables {
        let reason_el = "Ο φορέας περιέχει μη-γραμμικά στοιχεία καλωδίων αλυσοειδούς (BR-5).".to_string();
        let reason_en = "The structure contains nonlinear catenary cable elements (BR-5).".to_string();
        let methods = vec![
            MethodStatusDto {
                method_id: "IsostaticBeamFrame".to_string(),
                name_el: "Ισοστατικοί Φορείς & Δοκοί".to_string(),
                name_en: "Isostatic Beams & Frames".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "TrussJointsRitter".to_string(),
                name_el: "Δικτυώματα - Μέθοδος Κόμβων & Ritter".to_string(),
                name_en: "Trusses - Method of Joints & Ritter".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ContinuousBeamClapeyronCross".to_string(),
                name_el: "Συνεχείς Δοκοί - Clapeyron & Cross".to_string(),
                name_en: "Continuous Beams - Clapeyron & Cross".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ForceMethod".to_string(),
                name_el: "Μέθοδος Δυνάμεων".to_string(),
                name_en: "Force Method".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el),
                ineligibility_reason_en: Some(reason_en),
            },
        ];

        return Ok(MethodEligibility {
            model_revision: model.revision,
            methods,
            dsm_walkthrough_available: true,
            indeterminacy_degree: ns,
            is_stable: true,
            overall_eligible: false,
        });
    }

    // If unilateral supports are present
    if has_unilateral {
        let reason_el = "Ο φορέας περιέχει μη-γραμμικές μονόπλευρες στηρίξεις επαφής.".to_string();
        let reason_en = "The structure contains nonlinear unilateral contact supports.".to_string();
        let methods = vec![
            MethodStatusDto {
                method_id: "IsostaticBeamFrame".to_string(),
                name_el: "Ισοστατικοί Φορείς & Δοκοί".to_string(),
                name_en: "Isostatic Beams & Frames".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "TrussJointsRitter".to_string(),
                name_el: "Δικτυώματα - Μέθοδος Κόμβων & Ritter".to_string(),
                name_en: "Trusses - Method of Joints & Ritter".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ContinuousBeamClapeyronCross".to_string(),
                name_el: "Συνεχείς Δοκοί - Clapeyron & Cross".to_string(),
                name_en: "Continuous Beams - Clapeyron & Cross".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el.clone()),
                ineligibility_reason_en: Some(reason_en.clone()),
            },
            MethodStatusDto {
                method_id: "ForceMethod".to_string(),
                name_el: "Μέθοδος Δυνάμεων".to_string(),
                name_en: "Force Method".to_string(),
                eligible: false,
                ineligibility_reason_el: Some(reason_el),
                ineligibility_reason_en: Some(reason_en),
            },
        ];

        return Ok(MethodEligibility {
            model_revision: model.revision,
            methods,
            dsm_walkthrough_available: true,
            indeterminacy_degree: ns,
            is_stable: true,
            overall_eligible: false,
        });
    }

    // Determine structural topology:
    // Is it a pure truss?
    let is_pure_truss = !model.elements.is_empty()
        && model
            .elements
            .iter()
            .all(|e| matches!(e, Element::TrussBar { .. }));

    // Is it a collinear horizontal continuous beam?
    let is_continuous_beam = check_continuous_beam(model);

    let mut methods = Vec::new();

    // 1. Isostatic Beams & Frames
    if ns == 0 && !is_pure_truss {
        methods.push(MethodStatusDto {
            method_id: "IsostaticBeamFrame".to_string(),
            name_el: "Ισοστατικοί Φορείς & Δοκοί".to_string(),
            name_en: "Isostatic Beams & Frames".to_string(),
            eligible: true,
            ineligibility_reason_el: None,
            ineligibility_reason_en: None,
        });
    } else {
        let reason_el = if ns != 0 {
            format!("Απαιτείται βαθμός υπερστατικότητας ns = 0 (τρέχων: ns = {}).", ns)
        } else {
            "Ο φορέας αποτελείται αποκλειστικά από ράβδους δικτυώματος.".to_string()
        };
        let reason_en = if ns != 0 {
            format!("Requires static indeterminacy ns = 0 (current: ns = {}).", ns)
        } else {
            "Structure consists exclusively of truss bars.".to_string()
        };
        methods.push(MethodStatusDto {
            method_id: "IsostaticBeamFrame".to_string(),
            name_el: "Ισοστατικοί Φορείς & Δοκοί".to_string(),
            name_en: "Isostatic Beams & Frames".to_string(),
            eligible: false,
            ineligibility_reason_el: Some(reason_el),
            ineligibility_reason_en: Some(reason_en),
        });
    }

    // 2. Truss Joints & Ritter
    if is_pure_truss && ns == 0 {
        methods.push(MethodStatusDto {
            method_id: "TrussJointsRitter".to_string(),
            name_el: "Δικτυώματα - Μέθοδος Κόμβων & Ritter".to_string(),
            name_en: "Trusses - Method of Joints & Ritter".to_string(),
            eligible: true,
            ineligibility_reason_el: None,
            ineligibility_reason_en: None,
        });
    } else {
        let reason_el = if !is_pure_truss {
            "Απαιτείται αμιγές δικτύωμα (μόνο ράβδοι δικτυώματος TrussBar).".to_string()
        } else {
            format!("Απαιτείται ισοστατικό δικτύωμα με ns = 0 (τρέχων: ns = {}).", ns)
        };
        let reason_en = if !is_pure_truss {
            "Requires a pure truss (TrussBar elements only).".to_string()
        } else {
            format!("Requires an isostatic truss with ns = 0 (current: ns = {}).", ns)
        };
        methods.push(MethodStatusDto {
            method_id: "TrussJointsRitter".to_string(),
            name_el: "Δικτυώματα - Μέθοδος Κόμβων & Ritter".to_string(),
            name_en: "Trusses - Method of Joints & Ritter".to_string(),
            eligible: false,
            ineligibility_reason_el: Some(reason_el),
            ineligibility_reason_en: Some(reason_en),
        });
    }

    // 3. Continuous Beams - Clapeyron & Cross
    if is_continuous_beam && ns >= 1 && ns <= 3 {
        methods.push(MethodStatusDto {
            method_id: "ContinuousBeamClapeyronCross".to_string(),
            name_el: "Συνεχείς Δοκοί - Clapeyron & Cross".to_string(),
            name_en: "Continuous Beams - Clapeyron & Cross".to_string(),
            eligible: true,
            ineligibility_reason_el: None,
            ineligibility_reason_en: None,
        });
    } else {
        let reason_el = if !is_continuous_beam {
            "Εφαρμόζεται σε οριζόντιες συνεχείς δοκούς πολλαπλών ανοιγμάτων.".to_string()
        } else if ns < 1 {
            "Η δοκός είναι ισοστατική (ns = 0). Δεν απαιτείται μέθοδος Cross/Clapeyron.".to_string()
        } else {
            format!("Ο βαθμός υπερστατικότητας (ns = {}) υπερβαίνει το όριο (ns <= 3).", ns)
        };
        let reason_en = if !is_continuous_beam {
            "Applies to horizontal continuous beams across multiple spans.".to_string()
        } else if ns < 1 {
            "Beam is statically determinate (ns = 0). Cross/Clapeyron not needed.".to_string()
        } else {
            format!("Degree of indeterminacy (ns = {}) exceeds limit (ns <= 3).", ns)
        };
        methods.push(MethodStatusDto {
            method_id: "ContinuousBeamClapeyronCross".to_string(),
            name_el: "Συνεχείς Δοκοί - Clapeyron & Cross".to_string(),
            name_en: "Continuous Beams - Clapeyron & Cross".to_string(),
            eligible: false,
            ineligibility_reason_el: Some(reason_el),
            ineligibility_reason_en: Some(reason_en),
        });
    }

    // 4. Force Method
    if ns >= 1 && ns <= 3 {
        methods.push(MethodStatusDto {
            method_id: "ForceMethod".to_string(),
            name_el: "Μέθοδος Δυνάμεων".to_string(),
            name_en: "Force Method".to_string(),
            eligible: true,
            ineligibility_reason_el: None,
            ineligibility_reason_en: None,
        });
    } else {
        let reason_el = if ns < 1 {
            "Ο φορέας είναι ισοστατικός (ns = 0). Δεν απαιτούνται υπερστατικές άγνωστες δυνάμεις.".to_string()
        } else {
            format!("Ο βαθμός υπερστατικότητας (ns = {}) υπερβαίνει το όριο ns <= 3 για αναλυτική μέθοδο δυνάμεων.", ns)
        };
        let reason_en = if ns < 1 {
            "Structure is statically determinate (ns = 0). No redundant forces required.".to_string()
        } else {
            format!("Degree of indeterminacy (ns = {}) exceeds limit ns <= 3 for analytical force method.", ns)
        };
        methods.push(MethodStatusDto {
            method_id: "ForceMethod".to_string(),
            name_el: "Μέθοδος Δυνάμεων".to_string(),
            name_en: "Force Method".to_string(),
            eligible: false,
            ineligibility_reason_el: Some(reason_el),
            ineligibility_reason_en: Some(reason_en),
        });
    }

    let overall_eligible = methods.iter().any(|m| m.eligible);

    Ok(MethodEligibility {
        model_revision: model.revision,
        methods,
        dsm_walkthrough_available: true,
        indeterminacy_degree: ns,
        is_stable: true,
        overall_eligible,
    })
}

/// Helper function to check if a model represents a collinear horizontal continuous beam.
fn check_continuous_beam(model: &StructuralModel) -> bool {
    if model.elements.is_empty() {
        return false;
    }

    // Must consist only of FrameMembers
    for el in &model.elements {
        if !matches!(el, Element::FrameMember { .. }) {
            return false;
        }
    }

    // All nodes must have approximately the same y-coordinate (horizontal beam)
    let y0 = model.nodes[0].y;
    for node in &model.nodes {
        if (node.y - y0).abs() > 1e-4 {
            return false;
        }
    }

    // Must have at least 2 members or 3 supports
    model.elements.len() >= 2 || model.supports.len() >= 3
}
