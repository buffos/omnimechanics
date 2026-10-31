use super::*;

#[test]
fn test_cross_continuous_beam() {
    let config = ContinuousBeamConfig {
        l1: 6.0,
        l2: 4.0,
        q1: 12.0, // 12 kN/m on span 1
        p2: 20.0, // 20 kN on span 2
        left_fixed: false,
        right_fixed: false,
    };

    let solution = solve_continuous_beam_cross(&config, "el").unwrap();
    assert!(!solution.is_isostatic);
    assert!(solution.cross_table.is_some());

    let table = solution.cross_table.unwrap();
    assert!(table.distribution_factors.contains_key("BA"));
    assert!(table.distribution_factors.contains_key("BC"));

    // Check sum of distribution factors at B
    let df_ba = table.distribution_factors["BA"];
    let df_bc = table.distribution_factors["BC"];
    assert!((df_ba + df_bc - 1.0).abs() < 1e-4);

    // Check equilibrium of final moments at joint B: M_BA + M_BC approx 0
    let m_ba = table.final_moments["BA"];
    let m_bc = table.final_moments["BC"];
    assert!((m_ba + m_bc).abs() < 0.2);

    // Check diagrams generation
    assert_eq!(solution.diagrams.len(), 2);
    assert_eq!(solution.reactions.len(), 3);

    // Vertical equilibrium: R_Ay + R_By + R_Cy == q1 * L1 + P2
    let total_reaction_y: f64 = solution.reactions.iter().map(|r| r.ry).sum();
    let total_load = config.q1 * config.l1 + config.p2;
    assert!((total_reaction_y - total_load).abs() < 1e-4);
}
