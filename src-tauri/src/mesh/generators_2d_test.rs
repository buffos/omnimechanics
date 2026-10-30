use super::*;

#[test]
    fn test_beam_2d_quads() {
        let mesh = generate_beam_2d(10.0, 2.0, 5, 2, false);
        assert_eq!(mesh.stats.dimension, 2);
        assert_eq!(mesh.nodes.len(), (5 + 1) * (2 + 1));
        assert_eq!(mesh.volume_elements.len(), 5 * 2);
        assert_eq!(mesh.boundary_elements.len(), 2 * 5 + 2 * 2);

        // Verify boundary tags
        let tags: std::collections::HashSet<_> = mesh.boundary_elements.iter().map(|b| b.tag.as_str()).collect();
        assert!(tags.contains("Left"));
        assert!(tags.contains("Right"));
        assert!(tags.contains("Top"));
        assert!(tags.contains("Bottom"));

        // Verify nodes bounds
        for node in &mesh.nodes {
            assert!(node[0] >= 0.0 && node[0] <= 10.0);
            assert!(node[1] >= 0.0 && node[1] <= 2.0);
            assert_eq!(node[2], 0.0);
        }
    }

    #[test]
    fn test_beam_2d_triangles() {
        let mesh = generate_beam_2d(10.0, 2.0, 4, 3, true);
        assert_eq!(mesh.volume_elements.len(), 4 * 3 * 2);
        for elem in &mesh.volume_elements {
            assert_eq!(elem.elem_type, ElementType::Tri3);
            assert_eq!(elem.nodes.len(), 3);
        }
    }

    #[test]
    fn test_plate_with_hole_2d() {
        let mesh = generate_plate_with_hole_2d(20.0, 20.0, 2.0, 4, 16);
        assert_eq!(mesh.stats.dimension, 2);
        assert!(!mesh.nodes.is_empty());
        assert!(!mesh.volume_elements.is_empty());
        assert!(!mesh.boundary_elements.is_empty());

        let tags: std::collections::HashSet<_> = mesh.boundary_elements.iter().map(|b| b.tag.as_str()).collect();
        assert!(tags.contains("Hole"));
        assert!(tags.contains("Left"));
        assert!(tags.contains("Right"));
        assert!(tags.contains("Top"));
        assert!(tags.contains("Bottom"));

        // Ensure hole elements have "Hole" tag
        let hole_count = mesh.boundary_elements.iter().filter(|b| b.tag == "Hole").count();
        assert_eq!(hole_count, 16);
    }
