use super::*;

#[test]
fn test_simply_supported_beam_distributed_load() {
    let length = 6.0;
    let supports = vec![
        (0.0, SupportCondition::Pinned),
        (6.0, SupportCondition::RollerX),
    ];
    let loads = BeamLoadSpec {
        point_loads: vec![],
        dist_loads: vec![(0.0, 6.0, 12.0)], // q = 12 kN/m
        point_moments: vec![],
    };

    let solution = solve_isostatic_beam_system(length, &supports, &loads, "el").unwrap();
    assert_eq!(solution.reactions.len(), 2);
    // R_Ay = q*L/2 = 12*6/2 = 36 kN
    assert!((solution.reactions[0].ry - 36.0).abs() < 1e-4);
    assert!((solution.reactions[1].ry - 36.0).abs() < 1e-4);

    // M_max = q*L^2/8 = 12*36/8 = 54 kNm at x = 3.0
    let diag = &solution.diagrams[0];
    assert!(!diag.q_zero_locations.is_empty());
    assert!((diag.q_zero_locations[0] - 3.0).abs() < 0.1);
    assert!(!diag.m_extreme_values.is_empty());
    assert!((diag.m_extreme_values[0] - 54.0).abs() < 0.2);
}

#[test]
fn test_cantilever_beam_point_load() {
    let length = 4.0;
    let supports = vec![(0.0, SupportCondition::Clamped)];
    let loads = BeamLoadSpec {
        point_loads: vec![(4.0, 15.0)], // P = 15 kN at tip
        dist_loads: vec![],
        point_moments: vec![],
    };

    let solution = solve_isostatic_beam_system(length, &supports, &loads, "el").unwrap();
    assert_eq!(solution.reactions.len(), 1);
    // R_Ay = 15 kN, M_A = 15 * 4 = 60 kNm
    assert!((solution.reactions[0].ry - 15.0).abs() < 1e-4);
    assert!((solution.reactions[0].rm - 60.0).abs() < 1e-4);
}
