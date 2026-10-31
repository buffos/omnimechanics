use super::*;

#[test]
fn test_preset_simply_supported() {
    let config = TeachingInputConfig {
        preset_name: "simply_supported_distributed".to_string(),
        method: "diagrams".to_string(),
        language: Some("en".to_string()),
        length: Some(6.0),
        height: None,
        load_q: Some(12.0),
        load_p: None,
        load_p_pos: None,
        load_m: None,
        custom_nodes: None,
        custom_members: None,
        custom_supports: None,
        custom_point_loads: None,
        custom_dist_loads: None,
    };

    let solution = solve_preset(&config).unwrap();
    assert_eq!(solution.system_type, "beam");
    assert!(solution.steps.len() >= 3);
}

#[test]
fn test_preset_pratt_truss() {
    let config = TeachingInputConfig {
        preset_name: "pratt_truss".to_string(),
        method: "joints".to_string(),
        language: Some("el".to_string()),
        length: Some(12.0),
        height: Some(3.0),
        load_q: None,
        load_p: Some(20.0),
        load_p_pos: None,
        load_m: None,
        custom_nodes: None,
        custom_members: None,
        custom_supports: None,
        custom_point_loads: None,
        custom_dist_loads: None,
    };

    let solution = solve_preset(&config).unwrap();
    assert_eq!(solution.system_type, "truss");
    assert!(!solution.diagrams.is_empty());
}

#[test]
fn test_preset_cross() {
    let config = TeachingInputConfig {
        preset_name: "cross_two_span".to_string(),
        method: "cross".to_string(),
        language: None,
        length: Some(6.0),
        height: Some(4.0),
        load_q: Some(15.0),
        load_p: Some(30.0),
        load_p_pos: None,
        load_m: None,
        custom_nodes: None,
        custom_members: None,
        custom_supports: None,
        custom_point_loads: None,
        custom_dist_loads: None,
    };

    let solution = solve_preset(&config).unwrap();
    assert_eq!(solution.system_type, "continuous_beam");
    assert!(solution.cross_table.is_some());
}
