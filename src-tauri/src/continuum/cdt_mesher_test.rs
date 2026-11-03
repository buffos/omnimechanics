#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::validation::*;
    use super::super::cdt_mesher::*;

    fn create_perforated_plate_domain() -> ContinuumDomain {
        let outer_vertices = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: 5.0, y: -5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: -5.0, y: 5.0 },
        ];
        let outer = close_and_normalize_loop("L_OUTER", &outer_vertices, false).unwrap();

        ContinuumDomain {
            id: "CD1".into(),
            revision: 3,
            outer_boundary: outer,
            cutouts: vec![Cutout {
                id: "H1".into(),
                shape: CutoutShape::Circle {
                    center_x: 0.0,
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
                min_angle_deg: 25.0,
                volume_element_type: VolumeElementKind::Tri3,
                boundary_element_type: BoundaryElementKind::Line2,
            },
        }
    }

    #[test]
    fn test_sc001_conformal_graded_mesh_generation() {
        let domain = create_perforated_plate_domain();
        let handoff = generate_conformal_mesh(&domain).expect("Mesh generation must succeed");

        assert_eq!(handoff.domain_id, "CD1");
        assert_eq!(handoff.domain_revision, 3);
        assert_eq!(handoff.mesh_data.dimension, 2);

        // Quality report checks
        assert!(handoff.quality_report.is_conformal, "Mesh must be 100% conformal");
        assert!(
            handoff.quality_report.min_angle_deg >= 20.0,
            "min_angle_deg ({:.2}) must be >= 20.0",
            handoff.quality_report.min_angle_deg
        );

        // Hole boundary element checks: edge length <= 1.2 * (h * alpha) = 1.2 * (0.8 * 0.35) = 0.336 m
        let max_hole_edge = 1.2 * (0.8 * 0.35);
        let hole_elements: Vec<_> = handoff
            .mesh_data
            .boundary_elements
            .iter()
            .filter(|b| b.tag == "Hole")
            .collect();
        assert!(!hole_elements.is_empty(), "Must contain hole boundary elements");

        for bnd in &hole_elements {
            let p0 = handoff.mesh_data.nodes[bnd.nodes[0]];
            let p1 = handoff.mesh_data.nodes[bnd.nodes[1]];
            let edge_len = ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt();
            assert!(
                edge_len <= max_hole_edge,
                "Hole edge length {:.4} must be <= {:.4}",
                edge_len,
                max_hole_edge
            );

            // Radius check
            let r0 = (p0[0].powi(2) + p0[1].powi(2)).sqrt();
            assert!((r0 - 1.5).abs() < 1e-4, "Hole nodes must lie on circle r = 1.5");

            // Inward-pointing normal check (dot product with center direction)
            let mid_x = (p0[0] + p1[0]) / 2.0;
            let mid_y = (p0[1] + p1[1]) / 2.0;
            let dot = bnd.normal[0] * mid_x + bnd.normal[1] * mid_y;
            assert!(dot < 0.0, "Normal must point towards hole center");

            // Shared node check: must share both nodes with at least one Tri3 volume element
            let shared = handoff
                .mesh_data
                .volume_elements
                .iter()
                .any(|v| v.nodes.contains(&bnd.nodes[0]) && v.nodes.contains(&bnd.nodes[1]));
            assert!(shared, "Hole boundary element must share nodes with adjacent Tri3 element");
        }
    }

    #[test]
    fn test_sc006_semantic_tag_propagation() {
        let domain = create_perforated_plate_domain();
        let handoff = generate_conformal_mesh(&domain).unwrap();

        // Check that Bottom, Right, Top, Left, Hole are all present in tag_element_counts
        assert!(handoff.tag_element_counts.contains_key("Bottom"));
        assert!(handoff.tag_element_counts.contains_key("Right"));
        assert!(handoff.tag_element_counts.contains_key("Top"));
        assert!(handoff.tag_element_counts.contains_key("Left"));
        assert!(handoff.tag_element_counts.contains_key("Hole"));

        let total_tagged: usize = handoff.tag_element_counts.values().sum();
        assert_eq!(total_tagged, handoff.mesh_data.boundary_elements.len());
    }

    #[test]
    fn test_sc007_inclusion_region_mesh_no_hole_boundary_elements() {
        let outer_vertices = vec![
            Vertex2D { id: "V1".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "V2".into(), x: 5.0, y: -5.0 },
            Vertex2D { id: "V3".into(), x: 5.0, y: 5.0 },
            Vertex2D { id: "V4".into(), x: -5.0, y: 5.0 },
        ];
        let outer = close_and_normalize_loop("L_OUTER", &outer_vertices, false).unwrap();

        let domain = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![],
            inclusions: vec![InclusionRegion {
                id: "INC1".into(),
                shape: CutoutShape::Circle {
                    center_x: 0.0,
                    center_y: 0.0,
                    radius: 1.5,
                    angular_divisions: 32,
                },
                material_label: "StiffCore".into(),
                youngs_modulus_override_gpa: Some(400.0),
                poisson_ratio_override: Some(0.25),
                local_element_size: None,
            }],
            tag_conditions: Default::default(),
            meshing_profile: Default::default(),
        };

        let handoff = generate_conformal_mesh(&domain).expect("Inclusion mesh must succeed");

        // Must have ZERO Hole boundary elements
        let hole_elements_count = handoff
            .mesh_data
            .boundary_elements
            .iter()
            .filter(|b| b.tag == "Hole")
            .count();
        assert_eq!(hole_elements_count, 0, "Inclusions must not generate Hole boundary elements");

        // Must have volume elements inside r < 1.5
        let inside_inclusion_count = handoff.mesh_data.volume_elements.iter().filter(|v| {
            let p0 = handoff.mesh_data.nodes[v.nodes[0]];
            let p1 = handoff.mesh_data.nodes[v.nodes[1]];
            let p2 = handoff.mesh_data.nodes[v.nodes[2]];
            let cx = (p0[0] + p1[0] + p2[0]) / 3.0;
            let cy = (p0[1] + p1[1] + p2[1]) / 3.0;
            (cx * cx + cy * cy).sqrt() < 1.49
        }).count();
        assert!(inside_inclusion_count > 0, "Inclusion interior must be populated with volume elements");
        assert!(handoff.quality_report.is_conformal);
    }

    #[test]
    fn test_sc008_solver_handoff_convergence() {
        let domain = create_perforated_plate_domain();
        let handoff = generate_conformal_mesh(&domain).unwrap();

        let strategy = crate::material::types::MaterialStrategyConfig::DirectionalGradient {
            axis: crate::material::types::Axis::X,
            alpha: 0.0,
            length_scale: 1.0,
            power: 1.0,
            form: crate::material::types::GradientForm::Linear,
        };

        // 1. Solve PureFEM
        let sim_req_fem = crate::solver::types::SimulationRequest {
            mesh: handoff.mesh_data.clone(),
            solver_mode: crate::solver::types::SolverMode::PureFEM,
            reference_material: crate::material::types::ReferenceMaterial {
                e0: 2.0e11,
                nu0: 0.3,
                rho0: 7850.0,
            },
            strategy: strategy.clone(),
            load_magnitude: 10000.0,
            stress_state: Some(crate::solver::types::StressState2D::PlaneStress),
            max_iterations: 50,
            tolerance: 1e-4,
            hybrid_acceleration: None,
            bem_element_type: None,
        };
        let res_fem = crate::solver::run_simulation(sim_req_fem);
        assert!(res_fem.converged, "PureFEM must converge on sketched domain");
        assert!(res_fem.max_displacement > 0.0);
        assert!(res_fem.max_von_mises > 0.0);

        // 2. Solve BEMFEM
        let sim_req_bemfem = crate::solver::types::SimulationRequest {
            mesh: handoff.mesh_data,
            solver_mode: crate::solver::types::SolverMode::BEMFEM,
            reference_material: crate::material::types::ReferenceMaterial {
                e0: 2.0e11,
                nu0: 0.3,
                rho0: 7850.0,
            },
            strategy,
            load_magnitude: 10000.0,
            stress_state: Some(crate::solver::types::StressState2D::PlaneStress),
            max_iterations: 50,
            tolerance: 1e-4,
            hybrid_acceleration: None,
            bem_element_type: None,
        };
        let res_bemfem = crate::solver::run_simulation(sim_req_bemfem);
        assert!(res_bemfem.converged, "BEMFEM must converge on sketched domain");
        assert!(res_bemfem.max_displacement > 0.0);
        assert!(res_bemfem.max_von_mises > 0.0);
    }

    #[test]
    fn test_sc009_stale_mesh_handoff_rejected() {
        let mut domain = create_perforated_plate_domain();
        domain.revision = 2;
        let handoff = generate_conformal_mesh(&domain).unwrap();
        assert_eq!(handoff.domain_revision, 2);

        // Mutating domain increments revision
        domain.revision = 3;

        let err = verify_mesh_freshness(&domain, &handoff).unwrap_err();
        assert_eq!(err.code, "STALE_MESH_HANDOFF");
    }

    #[test]
    fn test_sc010_quad4_mesh_generation() {
        let mut domain = create_perforated_plate_domain();
        domain.meshing_profile.volume_element_type = VolumeElementKind::Quad4;
        let handoff = generate_conformal_mesh(&domain).expect("Quad4 meshing must succeed");

        assert!(
            handoff.mesh_data.volume_elements.iter().all(|e| e.elem_type == crate::mesh::types::ElementType::Quad4),
            "All generated volume elements must be Quad4"
        );
        assert!(handoff.quality_report.is_conformal, "Quad4 mesh must be 100% conformal");
        assert!(handoff.mesh_data.volume_elements.len() > 0);
    }

    #[test]
    fn test_sc011_frontend_template_quad4_mesh_generation() {
        // EXACT coordinates of the frontend default template domain (0..10, 0..10 with hole at (5,5))
        let outer_vertices = vec![
            Vertex2D { id: "v0".into(), x: 0.0, y: 0.0 },
            Vertex2D { id: "v1".into(), x: 10.0, y: 0.0 },
            Vertex2D { id: "v2".into(), x: 10.0, y: 10.0 },
            Vertex2D { id: "v3".into(), x: 0.0, y: 10.0 },
        ];
        let outer = close_and_normalize_loop("OuterLoop", &outer_vertices, false).unwrap();

        let domain = ContinuumDomain {
            id: "CD1".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![Cutout {
                id: "cutout_center_hole".into(),
                shape: CutoutShape::Circle {
                    center_x: 5.0,
                    center_y: 5.0,
                    radius: 1.5,
                    angular_divisions: 24,
                },
                tag: "Hole".into(),
                local_element_size: None,
            }],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 0.8,
                hole_refinement_factor: 0.35,
                min_angle_deg: 25.0,
                volume_element_type: VolumeElementKind::Quad4,
                boundary_element_type: BoundaryElementKind::Line2,
            },
        };

        let handoff = generate_conformal_mesh(&domain).expect("Frontend template Quad4 meshing must succeed");
        assert!(
            handoff.mesh_data.volume_elements.iter().all(|e| e.elem_type == crate::mesh::types::ElementType::Quad4),
            "All volume elements must be Quad4"
        );
        assert!(handoff.quality_report.is_conformal, "Mesh must be conformal");
        assert!(handoff.mesh_data.volume_elements.len() >= 64, "Must generate dense mesh of quads");
        assert!(handoff.mesh_data.boundary_elements.iter().any(|b| b.tag == "Hole"), "Must have Hole boundary elements");
    }

    #[test]
    fn test_sc012_solid_rectangle_grid_mesh() {
        // 10x6 m solid beam with Quad4
        let outer_vertices = vec![
            Vertex2D { id: "v0".into(), x: 0.0, y: 0.0 },
            Vertex2D { id: "v1".into(), x: 10.0, y: 0.0 },
            Vertex2D { id: "v2".into(), x: 10.0, y: 6.0 },
            Vertex2D { id: "v3".into(), x: 0.0, y: 6.0 },
        ];
        let outer = close_and_normalize_loop("OuterLoop", &outer_vertices, false).unwrap();

        let domain = ContinuumDomain {
            id: "CD_Beam".into(),
            revision: 1,
            outer_boundary: outer,
            cutouts: vec![],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 1.0,
                hole_refinement_factor: 1.0,
                min_angle_deg: 25.0,
                volume_element_type: VolumeElementKind::Quad4,
                boundary_element_type: BoundaryElementKind::Line2,
            },
        };

        let handoff = generate_conformal_mesh(&domain).expect("Solid rectangle Quad4 meshing must succeed");
        assert!(
            handoff.mesh_data.volume_elements.iter().all(|e| e.elem_type == crate::mesh::types::ElementType::Quad4),
            "All elements must be Quad4"
        );
        assert_eq!(handoff.mesh_data.volume_elements.len(), 10 * 6, "10x6 m with h=1.0m must produce exactly 60 quads");
        assert!(handoff.quality_report.is_conformal, "Must be conformal");
    }

    #[test]
    fn test_sc013_arbitrary_pentagon_quad4_and_size_sensitivity() {
        // Arbitrary 5-sided polygon (similar to user's sketch)
        let outer_vertices = vec![
            Vertex2D { id: "p0".into(), x: 0.0, y: 0.0 },
            Vertex2D { id: "p1".into(), x: 5.5, y: -0.5 },
            Vertex2D { id: "p2".into(), x: 7.0, y: 2.5 },
            Vertex2D { id: "p3".into(), x: 3.5, y: 5.0 },
            Vertex2D { id: "p4".into(), x: -1.0, y: 3.0 },
        ];
        let outer = close_and_normalize_loop("PentagonLoop", &outer_vertices, false).unwrap();

        // 1. Mesh with h = 1.0 m, Quad4
        let domain_coarse = ContinuumDomain {
            id: "CD_Pentagon".into(),
            revision: 1,
            outer_boundary: outer.clone(),
            cutouts: vec![],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 1.0,
                hole_refinement_factor: 1.0,
                min_angle_deg: 25.0,
                volume_element_type: VolumeElementKind::Quad4,
                boundary_element_type: BoundaryElementKind::Line2,
            },
        };

        let handoff_coarse = generate_conformal_mesh(&domain_coarse)
            .expect("Arbitrary pentagon Quad4 meshing must succeed");

        assert!(
            handoff_coarse.mesh_data.volume_elements.iter().all(|e| e.elem_type == crate::mesh::types::ElementType::Quad4),
            "ALL elements in Quad4 mode must be Quad4! Zero Tri3 allowed."
        );
        assert!(handoff_coarse.quality_report.is_conformal, "Coarse pentagon mesh must be 100% conformal");
        let n_coarse = handoff_coarse.mesh_data.volume_elements.len();
        assert!(n_coarse >= 10, "Must produce multiple quads (got {})", n_coarse);

        // 2. Mesh with h = 0.5 m, Quad4 (finer size)
        let domain_fine = ContinuumDomain {
            id: "CD_Pentagon".into(),
            revision: 2,
            outer_boundary: outer,
            cutouts: vec![],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 0.5,
                hole_refinement_factor: 1.0,
                min_angle_deg: 25.0,
                volume_element_type: VolumeElementKind::Quad4,
                boundary_element_type: BoundaryElementKind::Line2,
            },
        };

        let handoff_fine = generate_conformal_mesh(&domain_fine)
            .expect("Finer arbitrary pentagon Quad4 meshing must succeed");

        assert!(
            handoff_fine.mesh_data.volume_elements.iter().all(|e| e.elem_type == crate::mesh::types::ElementType::Quad4),
            "ALL elements in fine Quad4 mode must be Quad4!"
        );
        assert!(handoff_fine.quality_report.is_conformal, "Fine pentagon mesh must be 100% conformal");
        let n_fine = handoff_fine.mesh_data.volume_elements.len();

        // Verifying size sensitivity: reducing h from 1.0 to 0.5 MUST produce more elements!
        assert!(
            n_fine > n_coarse * 2,
            "Reducing h from 1.0 to 0.5 must significantly increase element count (coarse: {}, fine: {})",
            n_coarse, n_fine
        );
    }

    #[test]
    fn test_sc014_general_polygon_with_cutout_and_line3_provenance() {
        let outer_vertices = vec![
            Vertex2D { id: "p0".into(), x: -5.0, y: -5.0 },
            Vertex2D { id: "p1".into(), x: 6.0, y: -4.0 },
            Vertex2D { id: "p2".into(), x: 5.0, y: 6.0 },
            Vertex2D { id: "p3".into(), x: -4.0, y: 5.0 },
            Vertex2D { id: "p4".into(), x: -6.0, y: 0.0 },
        ];
        let outer = close_and_normalize_loop("PentagonLoop", &outer_vertices, false).unwrap();

        let domain = ContinuumDomain {
            id: "CD_GenHole".into(),
            revision: 4,
            outer_boundary: outer,
            cutouts: vec![Cutout {
                id: "H_RECT".into(),
                shape: CutoutShape::Rectangle {
                    center_x: 0.0,
                    center_y: 0.0,
                    width: 2.0,
                    height: 2.0,
                    rotation_deg: 0.0,
                },
                tag: "Hole".into(),
                local_element_size: Some(0.5),
            }],
            inclusions: vec![],
            tag_conditions: Default::default(),
            meshing_profile: MeshingProfile {
                target_element_size: 1.0,
                hole_refinement_factor: 0.5,
                min_angle_deg: 20.0,
                volume_element_type: VolumeElementKind::Tri3,
                boundary_element_type: BoundaryElementKind::Line3,
            },
        };

        let handoff = generate_conformal_mesh(&domain).expect("General polygon with rectangular cutout must mesh");
        assert!(handoff.quality_report.is_conformal);
        assert!(handoff.tag_element_counts.get("Hole").copied().unwrap_or(0) >= 8);
        assert_eq!(handoff.boundary_provenance.len(), handoff.mesh_data.boundary_elements.len());
        assert_eq!(handoff.volume_provenance.len(), handoff.mesh_data.volume_elements.len());
        assert!(
            handoff
                .mesh_data
                .boundary_elements
                .iter()
                .all(|b| b.elem_type == crate::mesh::types::ElementType::Line3 && b.nodes.len() == 3),
            "Line3 boundary element profile must emit 3-node Line3 elements"
        );
        // Ensure no volume element centroid lies inside the [-1, 1] x [-1, 1] rectangular hole
        for v in &handoff.mesh_data.volume_elements {
            let cx = (handoff.mesh_data.nodes[v.nodes[0]][0]
                + handoff.mesh_data.nodes[v.nodes[1]][0]
                + handoff.mesh_data.nodes[v.nodes[2]][0])
                / 3.0;
            let cy = (handoff.mesh_data.nodes[v.nodes[0]][1]
                + handoff.mesh_data.nodes[v.nodes[1]][1]
                + handoff.mesh_data.nodes[v.nodes[2]][1])
                / 3.0;
            assert!(
                !(cx.abs() < 0.99 && cy.abs() < 0.99),
                "No volume element centroid may lie inside the rectangular cutout void"
            );
        }
    }
}
