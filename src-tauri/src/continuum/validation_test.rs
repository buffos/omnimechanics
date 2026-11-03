#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::validation::*;

    #[test]
    fn test_sc002_self_intersecting_bowtie_polygon_rejected() {
        // V1(0,0), V2(4,4), V3(4,0), V4(0,4)
        let vertices = vec![
            Vertex2D { id: "V1".into(), x: 0.0, y: 0.0 },
            Vertex2D { id: "V2".into(), x: 4.0, y: 4.0 },
            Vertex2D { id: "V3".into(), x: 4.0, y: 0.0 },
            Vertex2D { id: "V4".into(), x: 0.0, y: 4.0 },
        ];

        // 1. Preview segment
        let preview = preview_segment(&vertices[0..3], &vertices[3], true);
        assert!(!preview.is_valid, "Bowtie closing segment must be marked invalid");
        assert!(!preview.intersecting_segment_indices.is_empty());

        // 2. Authoritative closure
        let err = close_and_normalize_loop("L_OUTER", &vertices, false).unwrap_err();
        assert_eq!(err.code, "SELF_INTERSECTING_LOOP");
        let details = err.details.expect("Details must be present");
        let int_pt = details.get("intersection_point").unwrap().as_array().unwrap();
        assert_eq!(int_pt[0].as_f64().unwrap(), 2.0);
        assert_eq!(int_pt[1].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_sc003_cutout_breach_and_clearance_rejected() {
        let outer_vertices = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: 5.0, y: -5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: -5.0, y: 5.0 },
        ];
        let outer = close_and_normalize_loop("L_OUTER", &outer_vertices, false).unwrap();

        // Case A: Cutout at (4.0, 0.0) with R = 1.5 extends to x = 5.5, breaching outer boundary (x = 5.0)
        let domain_breach = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer.clone(),
            cutouts: vec![Cutout {
                id: "H1".into(),
                shape: CutoutShape::Circle {
                    center_x: 4.0,
                    center_y: 0.0,
                    radius: 1.5,
                    angular_divisions: 32,
                },
                tag: "Hole".into(),
                local_element_size: None,
            }],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 0.8,
                hole_refinement_factor: 0.35,
                ..Default::default()
            },
        };

        let err_breach = validate_domain(&domain_breach).unwrap_err();
        assert_eq!(err_breach.code, "HOLE_OUTSIDE_OUTER_BOUNDARY");

        // Case B: Cutout at (3.49, 0.0) with R = 1.5 extends to x = 4.99, leaving clearance 0.01 < d_min = 0.028
        let domain_clearance = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![Cutout {
                id: "H1".into(),
                shape: CutoutShape::Circle {
                    center_x: 3.49,
                    center_y: 0.0,
                    radius: 1.5,
                    angular_divisions: 32,
                },
                tag: "Hole".into(),
                local_element_size: None,
            }],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 0.8,
                hole_refinement_factor: 0.35,
                ..Default::default()
            },
        };

        let err_clearance = validate_domain(&domain_clearance).unwrap_err();
        assert_eq!(err_clearance.code, "CLEARANCE_BELOW_MINIMUM");
        let details = err_clearance.details.expect("Details must be present");
        let act = details.get("actual_clearance").unwrap().as_f64().unwrap();
        let req = details.get("required_clearance").unwrap().as_f64().unwrap();
        assert!(act < req);
    }

    #[test]
    fn test_sc004_overlapping_interior_cutouts_rejected() {
        let outer_vertices = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: 5.0, y: -5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: -5.0, y: 5.0 },
        ];
        let outer = close_and_normalize_loop("L_OUTER", &outer_vertices, false).unwrap();

        // H1 at (-1.0, 0.0) R = 1.5, H2 at (1.0, 0.0) R = 1.5 -> Distance between centers = 2.0 < 3.0
        let domain = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![
                Cutout {
                    id: "H1".into(),
                    shape: CutoutShape::Circle {
                        center_x: -1.0,
                        center_y: 0.0,
                        radius: 1.5,
                        angular_divisions: 32,
                    },
                    tag: "Hole".into(),
                    local_element_size: None,
                },
                Cutout {
                    id: "H2".into(),
                    shape: CutoutShape::Circle {
                        center_x: 1.0,
                        center_y: 0.0,
                        radius: 1.5,
                        angular_divisions: 32,
                    },
                    tag: "Hole".into(),
                    local_element_size: None,
                },
            ],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: Default::default(),
        };

        let err = validate_domain(&domain).unwrap_err();
        assert_eq!(err.code, "HOLE_OVERLAPS_EXISTING_FEATURE");
        let details = err.details.expect("Details present");
        assert_eq!(details.get("feature_a").unwrap().as_str().unwrap(), "H1");
        assert_eq!(details.get("feature_b").unwrap().as_str().unwrap(), "H2");
    }

    #[test]
    fn test_sc005_winding_normalization_and_outward_normals() {
        // Outer rectangle drawn in CW order: (-5,-5) -> (-5,5) -> (5,5) -> (5,-5)
        let outer_cw = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: -5.0, y: 5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: 5.0, y: -5.0 },
        ];

        let outer_loop = close_and_normalize_loop("L_OUTER", &outer_cw, false).unwrap();
        assert_eq!(outer_loop.orientation, LoopOrientation::CounterClockwise);

        let coords: Vec<(f64, f64)> = outer_loop.vertices.iter().map(|v| (v.x, v.y)).collect();
        let area = compute_signed_area(&coords);
        assert!(area > 0.0, "Outer loop area must be positive (CCW)");
        assert_eq!(area, 100.0);

        // Check outward normals on outer segments
        for seg in &outer_loop.segments {
            match seg.tag.as_str() {
                "Bottom" => assert_eq!(seg.outward_normal, [0.0, -1.0, 0.0]),
                "Right" => assert_eq!(seg.outward_normal, [1.0, 0.0, 0.0]),
                "Top" => assert_eq!(seg.outward_normal, [0.0, 1.0, 0.0]),
                "Left" => assert_eq!(seg.outward_normal, [-1.0, 0.0, 0.0]),
                _ => {}
            }
        }

        // Cutout drawn in CCW order: (-1,-1) -> (1,-1) -> (1,1) -> (-1,1)
        let hole_ccw = vec![
            Vertex2D { id: "HV1".into(), x: -1.0, y: -1.0 },
            Vertex2D { id: "HV2".into(), x: 1.0, y: -1.0 },
            Vertex2D { id: "HV3".into(), x: 1.0, y: 1.0 },
            Vertex2D { id: "HV4".into(), x: -1.0, y: 1.0 },
        ];

        let hole_loop = close_and_normalize_loop("L_HOLE", &hole_ccw, true).unwrap();
        assert_eq!(hole_loop.orientation, LoopOrientation::Clockwise);

        let hole_coords: Vec<(f64, f64)> = hole_loop.vertices.iter().map(|v| (v.x, v.y)).collect();
        let hole_area = compute_signed_area(&hole_coords);
        assert!(hole_area < 0.0, "Hole loop area must be negative (CW)");
        assert_eq!(hole_area, -4.0);

        // Check that normals point into the hole cavity (away from solid material)
        // Bottom edge of hole at y = -1 must have outward normal pointing UP towards cavity center: [0, 1, 0]
        let bottom_seg = hole_loop.segments.iter().find(|s| {
            let v1 = hole_loop.vertices.iter().find(|v| v.id == s.start_vertex_id).unwrap();
            let v2 = hole_loop.vertices.iter().find(|v| v.id == s.end_vertex_id).unwrap();
            (v1.y - (-1.0)).abs() < 1e-4 && (v2.y - (-1.0)).abs() < 1e-4
        }).expect("Bottom edge of hole exists");
        assert_eq!(bottom_seg.outward_normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_vertex_on_non_incident_segment_and_intrinsic_feature_validation() {
        // 1. Vertex touching non-incident segment
        let vertices = vec![
            Vertex2D { id: "V1".into(), x: 0.0, y: 0.0 },
            Vertex2D { id: "V2".into(), x: 4.0, y: 0.0 },
            Vertex2D { id: "V3".into(), x: 4.0, y: 4.0 },
            Vertex2D { id: "V4".into(), x: 2.0, y: 0.0 }, // touches segment S1 (V1->V2)
            Vertex2D { id: "V5".into(), x: 0.0, y: 4.0 },
        ];
        let err = close_and_normalize_loop("L_TOUCH", &vertices, false).unwrap_err();
        assert_eq!(err.code, "SELF_INTERSECTING_LOOP");

        // 2. Inclusion with invalid Poisson ratio >= 0.5 rejected
        let outer_vertices = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: 5.0, y: -5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: -5.0, y: 5.0 },
        ];
        let outer = close_and_normalize_loop("L_OUTER", &outer_vertices, false).unwrap();
        let domain_bad_inc = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![],
            inclusions: vec![InclusionRegion {
                id: "INC_BAD".into(),
                shape: CutoutShape::Circle {
                    center_x: 0.0,
                    center_y: 0.0,
                    radius: 1.0,
                    angular_divisions: 16,
                },
                material_label: "BadNu".into(),
                youngs_modulus_override_gpa: Some(200.0),
                poisson_ratio_override: Some(0.55),
                local_element_size: None,
            }],
            tag_conditions: Default::default(),
            meshing_profile: Default::default(),
        };
        let err_inc = validate_domain(&domain_bad_inc).unwrap_err();
        assert_eq!(err_inc.code, "INVALID_BOUNDARY_CONDITION");
    }
}
