use std::collections::{HashMap, HashSet};

use super::types::{
    CanonicalError, Element, Load, OffsetFrame, StructuralModel,
};

pub fn validate_model(model: &StructuralModel) -> Result<(), CanonicalError> {
    // 1. Build lookup sets
    let mut node_map = HashMap::new();
    for n in &model.nodes {
        if node_map.insert(n.id.as_str(), (n.x, n.y)).is_some() {
            return Err(CanonicalError::new(
                "DUPLICATE_NODE_ID",
                format!("Διπλότυπο αναγνωριστικό κόμβου: {}", n.id),
                format!("Duplicate node identifier: {}", n.id),
            ));
        }
    }

    let material_set: HashSet<&str> = model.materials.iter().map(|m| m.id.as_str()).collect();
    let section_map: HashMap<&str, &super::types::Section> =
        model.sections.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut support_nodes = HashSet::new();
    let mut support_set = HashSet::new();

    // 2. Validate Supports (INV-02)
    for sup in &model.supports {
        if !node_map.contains_key(sup.node_id.as_str()) {
            return Err(CanonicalError::new(
                "MISSING_MODEL_REFERENCE",
                format!("Ο κόμβος στήριξης '{}' δεν υπάρχει στο μοντέλο.", sup.node_id),
                format!("Support node '{}' does not exist in model.", sup.node_id),
            ));
        }
        if !support_nodes.insert(sup.node_id.as_str()) {
            return Err(CanonicalError::new(
                "DUPLICATE_NODE_SUPPORT",
                format!("Περισσότερες από μία στηρίξεις στον ίδιο κόμβο '{}'.", sup.node_id),
                format!("Multiple supports assigned to the same node '{}'.", sup.node_id),
            ));
        }
        support_set.insert(sup.id.as_str());
    }

    // 3. Validate Elements (INV-01)
    let mut element_map = HashMap::new();
    for el in &model.elements {
        let el_id = el.id();
        if element_map.insert(el_id, el).is_some() {
            return Err(CanonicalError::new(
                "DUPLICATE_ELEMENT_ID",
                format!("Διπλότυπο αναγνωριστικό στοιχείου: {}", el_id),
                format!("Duplicate element identifier: {}", el_id),
            ));
        }

        match el {
            Element::FrameMember {
                id,
                node_i,
                node_j,
                section_id,
                material_id,
                offset_i,
                offset_j,
                ..
            } => {
                let p_i = node_map.get(node_i.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' του στοιχείου '{}' δεν βρέθηκε.", node_i, id),
                        format!("Node '{}' of element '{}' not found.", node_i, id),
                    )
                })?;
                let p_j = node_map.get(node_j.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' του στοιχείου '{}' δεν βρέθηκε.", node_j, id),
                        format!("Node '{}' of element '{}' not found.", node_j, id),
                    )
                })?;

                if node_i == node_j {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Το στοιχείο '{}' συνδέει τον ίδιο κόμβο (μηδενικό μήκος).", id),
                        format!("Element '{}' connects the same node (self-loop / zero length).", id),
                    ));
                }

                if !material_set.contains(material_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Το υλικό '{}' του στοιχείου '{}' δεν βρέθηκε.", material_id, id),
                        format!("Material '{}' of element '{}' not found.", material_id, id),
                    ));
                }

                if !section_map.contains_key(section_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Η διατομή '{}' του στοιχείου '{}' δεν βρέθηκε.", section_id, id),
                        format!("Section '{}' of element '{}' not found.", section_id, id),
                    ));
                }

                let dx_global = p_j.0 - p_i.0;
                let dy_global = p_j.1 - p_i.1;
                let nominal_length = dx_global.hypot(dy_global);
                if nominal_length < 1e-4 {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Το μήκος του στοιχείου '{}' είναι μικρότερο από 0.1 mm.", id),
                        format!("Length of element '{}' is smaller than 0.1 mm.", id),
                    ));
                }

                // Check rigid offsets
                let off_i_len = match offset_i.frame {
                    OffsetFrame::Local => offset_i.dx.abs(),
                    OffsetFrame::Global => offset_i.dx.hypot(offset_i.dy),
                };
                let off_j_len = match offset_j.frame {
                    OffsetFrame::Local => offset_j.dx.abs(),
                    OffsetFrame::Global => offset_j.dx.hypot(offset_j.dy),
                };

                if off_i_len + off_j_len >= 0.95 * nominal_length {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!(
                            "Οι δύσκαμπτες εκκεντρότητες του στοιχείου '{}' υπερβαίνουν το 95% του μήκους.",
                            id
                        ),
                        format!(
                            "Rigid offsets of element '{}' exceed 95% of nominal length.",
                            id
                        ),
                    ));
                }
            }
            Element::TrussBar {
                id,
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } => {
                let p_i = node_map.get(node_i.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' της ράβδου δικτυώματος '{}' δεν βρέθηκε.", node_i, id),
                        format!("Node '{}' of truss bar '{}' not found.", node_i, id),
                    )
                })?;
                let p_j = node_map.get(node_j.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' της ράβδου δικτυώματος '{}' δεν βρέθηκε.", node_j, id),
                        format!("Node '{}' of truss bar '{}' not found.", node_j, id),
                    )
                })?;

                if node_i == node_j {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Η ράβδος δικτυώματος '{}' έχει μηδενικό μήκος.", id),
                        format!("Truss bar '{}' has zero length.", id),
                    ));
                }

                let length = (p_j.0 - p_i.0).hypot(p_j.1 - p_i.1);
                if length < 1e-4 {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Το μήκος της ράβδου '{}' είναι μικρότερο από 0.1 mm.", id),
                        format!("Length of truss bar '{}' is smaller than 0.1 mm.", id),
                    ));
                }

                if !material_set.contains(material_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Το υλικό '{}' της ράβδου '{}' δεν βρέθηκε.", material_id, id),
                        format!("Material '{}' of truss bar '{}' not found.", material_id, id),
                    ));
                }

                if !section_map.contains_key(section_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Η διατομή '{}' της ράβδου '{}' δεν βρέθηκε.", section_id, id),
                        format!("Section '{}' of truss bar '{}' not found.", section_id, id),
                    ));
                }
            }
            Element::Spring {
                id,
                node_i,
                node_j,
                ..
            } => {
                if !node_map.contains_key(node_i.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' του ελατηρίου '{}' δεν βρέθηκε.", node_i, id),
                        format!("Node '{}' of spring '{}' not found.", node_i, id),
                    ));
                }
                if let Some(nj) = node_j {
                    if !node_map.contains_key(nj.as_str()) {
                        return Err(CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Ο 2ος κόμβος '{}' του ελατηρίου '{}' δεν βρέθηκε.", nj, id),
                            format!("Second node '{}' of spring '{}' not found.", nj, id),
                        ));
                    }
                    if node_i == nj {
                        return Err(CanonicalError::new(
                            "INVALID_ELEMENT_GEOMETRY",
                            format!("Το ελατήριο '{}' συνδέει τον ίδιο κόμβο.", id),
                            format!("Spring '{}' connects the same node.", id),
                        ));
                    }
                }
            }
            Element::Cable {
                id,
                node_i,
                node_j,
                section_id,
                material_id,
                ..
            } => {
                let p_i = node_map.get(node_i.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' του καλωδίου '{}' δεν βρέθηκε.", node_i, id),
                        format!("Node '{}' of cable '{}' not found.", node_i, id),
                    )
                })?;
                let p_j = node_map.get(node_j.as_str()).ok_or_else(|| {
                    CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Ο κόμβος '{}' του καλωδίου '{}' δεν βρέθηκε.", node_j, id),
                        format!("Node '{}' of cable '{}' not found.", node_j, id),
                    )
                })?;

                if node_i == node_j {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Το καλώδιο '{}' έχει μηδενικό μήκος.", id),
                        format!("Cable '{}' has zero length.", id),
                    ));
                }

                let chord = (p_j.0 - p_i.0).hypot(p_j.1 - p_i.1);
                if chord < 1e-4 {
                    return Err(CanonicalError::new(
                        "INVALID_ELEMENT_GEOMETRY",
                        format!("Το μήκος χορδής του καλωδίου '{}' είναι μικρότερο από 0.1 mm.", id),
                        format!("Chord length of cable '{}' is smaller than 0.1 mm.", id),
                    ));
                }

                if !material_set.contains(material_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Το υλικό '{}' του καλωδίου '{}' δεν βρέθηκε.", material_id, id),
                        format!("Material '{}' of cable '{}' not found.", material_id, id),
                    ));
                }

                if !section_map.contains_key(section_id.as_str()) {
                    return Err(CanonicalError::new(
                        "MISSING_MODEL_REFERENCE",
                        format!("Η διατομή '{}' του καλωδίου '{}' δεν βρέθηκε.", section_id, id),
                        format!("Section '{}' of cable '{}' not found.", section_id, id),
                    ));
                }
            }
        }
    }

    // 4. Validate Loads inside LoadCases (INV-03)
    for lc in &model.load_cases {
        for load in &lc.loads {
            match load {
                Load::NodalLoad { node_id, .. } => {
                    if !node_map.contains_key(node_id.as_str()) {
                        return Err(CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Το φορτίο κόμβου αναφέρεται σε άγνωστο κόμβο '{}'.", node_id),
                            format!("Nodal load references unknown node '{}'.", node_id),
                        ));
                    }
                }
                Load::MemberPointLoad {
                    element_id,
                    x_ratio,
                    ..
                } => {
                    let el = element_map.get(element_id.as_str()).ok_or_else(|| {
                        CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Το σημειακό φορτίο αναφέρεται σε άγνωστο στοιχείο '{}'.", element_id),
                            format!("Point load references unknown element '{}'.", element_id),
                        )
                    })?;

                    if matches!(el, Element::Cable { .. }) {
                        return Err(CanonicalError::new(
                            "INVALID_LOAD_ASSIGNMENT",
                            format!("Δεν επιτρέπονται εγκάρσια σημειακά φορτία σε καλώδιο '{}'.", element_id),
                            format!("Transverse point loads are not allowed on cable '{}'.", element_id),
                        ));
                    }

                    if *x_ratio < 0.0 || *x_ratio > 1.0 {
                        return Err(CanonicalError::new(
                            "INVALID_LOAD_ASSIGNMENT",
                            format!("Ο λόγος x_ratio = {} πρέπει να είναι μεταξύ 0 και 1.", x_ratio),
                            format!("Ratio x_ratio = {} must be between 0 and 1.", x_ratio),
                        ));
                    }
                }
                Load::MemberDistributedLoad {
                    element_id,
                    a_ratio,
                    b_ratio,
                    ..
                } => {
                    let el = element_map.get(element_id.as_str()).ok_or_else(|| {
                        CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Το κατανεμημένο φορτίο αναφέρεται σε άγνωστο στοιχείο '{}'.", element_id),
                            format!("Distributed load references unknown element '{}'.", element_id),
                        )
                    })?;

                    if matches!(el, Element::Cable { .. }) {
                        return Err(CanonicalError::new(
                            "INVALID_LOAD_ASSIGNMENT",
                            format!("Δεν επιτρέπονται κατανεμημένα φορτία σε καλώδιο '{}'.", element_id),
                            format!("Distributed loads are not allowed on cable '{}'.", element_id),
                        ));
                    }

                    if *a_ratio < 0.0 || *b_ratio > 1.0 || *a_ratio >= *b_ratio {
                        return Err(CanonicalError::new(
                            "INVALID_LOAD_ASSIGNMENT",
                            format!("Μη έγκυρα όρια κατανεμημένου φορτίου a_ratio={}, b_ratio={}.", a_ratio, b_ratio),
                            format!("Invalid distributed load bounds a_ratio={}, b_ratio={}.", a_ratio, b_ratio),
                        ));
                    }
                }
                Load::SupportSettlement { support_id, .. } => {
                    if !support_set.contains(support_id.as_str()) {
                        return Err(CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Η καθίζηση αναφέρεται σε άγνωστη στήριξη '{}'.", support_id),
                            format!("Support settlement references unknown support '{}'.", support_id),
                        ));
                    }
                }
                Load::ThermalLoad {
                    element_id,
                    dt_gradient,
                    ..
                } => {
                    let el = element_map.get(element_id.as_str()).ok_or_else(|| {
                        CanonicalError::new(
                            "MISSING_MODEL_REFERENCE",
                            format!("Το θερμοκρασιακό φορτίο αναφέρεται σε άγνωστο στοιχείο '{}'.", element_id),
                            format!("Thermal load references unknown element '{}'.", element_id),
                        )
                    })?;

                    if let Element::FrameMember { section_id, .. } = el {
                        let sec = section_map.get(section_id.as_str()).unwrap();
                        if dt_gradient.abs() > 1e-9 && sec.depth_h.is_none() {
                            return Err(CanonicalError::new(
                                "INVALID_LOAD_ASSIGNMENT",
                                format!(
                                    "Η διατομή '{}' του στοιχείου '{}' δεν ορίζει ύψος depth_h για θερμοκρασιακή βαθμίδα.",
                                    section_id, element_id
                                ),
                                format!(
                                    "Section '{}' of element '{}' does not define depth_h required for thermal gradient.",
                                    section_id, element_id
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // 5. Validate LumpedMasses
    for mass in &model.lumped_masses {
        if !node_map.contains_key(mass.node_id.as_str()) {
            return Err(CanonicalError::new(
                "MISSING_MODEL_REFERENCE",
                format!("Η συγκεντρωμένη μάζα αναφέρεται σε άγνωστο κόμβο '{}'.", mass.node_id),
                format!("Lumped mass references unknown node '{}'.", mass.node_id),
            ));
        }
    }

    Ok(())
}
