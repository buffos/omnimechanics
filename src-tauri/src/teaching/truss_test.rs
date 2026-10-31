use super::*;

#[test]
fn test_simple_triangle_truss() {
    // Triangular truss: nodes (0,0), (4,0), (2,2)
    let nodes = vec![
        Node2D { id: 0, x: 0.0, y: 0.0, label: "A".to_string() },
        Node2D { id: 1, x: 4.0, y: 0.0, label: "B".to_string() },
        Node2D { id: 2, x: 2.0, y: 2.0, label: "C".to_string() },
    ];
    let members = vec![
        Member2D { id: 0, start_node: 0, end_node: 1, length: 4.0, ea: 1e7, ei: 0.0, label: "AB".to_string() },
        Member2D { id: 1, start_node: 0, end_node: 2, length: (8.0f64).sqrt(), ea: 1e7, ei: 0.0, label: "AC".to_string() },
        Member2D { id: 2, start_node: 1, end_node: 2, length: (8.0f64).sqrt(), ea: 1e7, ei: 0.0, label: "BC".to_string() },
    ];
    let supports = vec![
        Support2D { node_id: 0, condition: SupportCondition::Pinned },
        Support2D { node_id: 1, condition: SupportCondition::RollerX },
    ];
    let loads = vec![
        PointLoad2D { node_id: 2, fx: 0.0, fy: -10.0, mz: 0.0 }, // 10 kN downward at apex
    ];

    let truss = TrussSystem { nodes, members, supports, loads };
    let solution = solve_teaching_truss(&truss, "el").unwrap();

    assert!(solution.is_isostatic);
    assert_eq!(solution.reactions.len(), 2);
    // Vertical reactions: R_Ay = 5, R_By = 5
    assert!((solution.reactions[0].ry - 5.0).abs() < 1e-4);
    assert!((solution.reactions[1].ry - 5.0).abs() < 1e-4);

    // Diagonals AC and BC are in compression: N = -5 / sin(45) = -5 * sqrt(2) = -7.07 kN
    // Bottom chord AB is in tension: N = 5 / tan(45) = +5.0 kN
    let n_ab = solution.diagrams[0].samples[0].n;
    assert!((n_ab - 5.0).abs() < 1e-2);
}
