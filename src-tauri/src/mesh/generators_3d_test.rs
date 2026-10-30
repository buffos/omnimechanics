use super::*;

#[test]
    fn test_beam_3d_hex() {
        let mesh = generate_beam_3d(10.0, 2.0, 3.0, 4, 2, 2, false);
        assert_eq!(mesh.stats.dimension, 3);
        assert_eq!(mesh.nodes.len(), (4 + 1) * (2 + 1) * (2 + 1));
        assert_eq!(mesh.volume_elements.len(), 4 * 2 * 2);

        // Check boundary tags
        let tags: std::collections::HashSet<_> = mesh.boundary_elements.iter().map(|b| b.tag.as_str()).collect();
        assert!(tags.contains("Left"));
        assert!(tags.contains("Right"));
        assert!(tags.contains("Bottom"));
        assert!(tags.contains("Top"));
        assert!(tags.contains("Back"));
        assert!(tags.contains("Front"));

        for elem in &mesh.volume_elements {
            assert_eq!(elem.elem_type, ElementType::Hex8);
            assert_eq!(elem.nodes.len(), 8);
        }
    }

    #[test]
    fn test_beam_3d_tet() {
        let mesh = generate_beam_3d(6.0, 2.0, 2.0, 3, 2, 2, true);
        assert_eq!(mesh.volume_elements.len(), 3 * 2 * 2 * 6); // 6 tets per cube
        for elem in &mesh.volume_elements {
            assert_eq!(elem.elem_type, ElementType::Tet4);
            assert_eq!(elem.nodes.len(), 4);
        }
    }
