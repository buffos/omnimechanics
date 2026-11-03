use super::dynamics::{
    der_kiureghian_correlation, evaluate_en1998_design_spectrum, solve_lateral_force,
    solve_modal_analysis, solve_response_spectrum,
};
use super::types::*;

fn create_3storey_benchmark_frame() -> StructuralModel {
    let mut model = StructuralModel::default();

    // 3-storey single-bay frame:
    // Span = 6.0 m
    // Floor heights: Y = 0 (Base), 3.0 m (Storey 1), 6.0 m (Storey 2), 9.0 m (Storey 3)
    // Nodes:
    // N1 (0, 0), N2 (6, 0)
    // N3 (0, 3), N4 (6, 3)
    // N5 (0, 6), N6 (6, 6)
    // N7 (0, 9), N8 (6, 9)
    model.nodes = vec![
        Node { id: "N1".to_string(), x: 0.0, y: 0.0 },
        Node { id: "N2".to_string(), x: 6.0, y: 0.0 },
        Node { id: "N3".to_string(), x: 0.0, y: 3.0 },
        Node { id: "N4".to_string(), x: 6.0, y: 3.0 },
        Node { id: "N5".to_string(), x: 0.0, y: 6.0 },
        Node { id: "N6".to_string(), x: 6.0, y: 6.0 },
        Node { id: "N7".to_string(), x: 0.0, y: 9.0 },
        Node { id: "N8".to_string(), x: 6.0, y: 9.0 },
    ];

    model.supports = vec![
        Support {
            id: "S1".to_string(),
            node_id: "N1".to_string(),
            kind: SupportKind::Fixed,
            unilateral: false,
        },
        Support {
            id: "S2".to_string(),
            node_id: "N2".to_string(),
            kind: SupportKind::Fixed,
            unilateral: false,
        },
    ];

    // Reinforced Concrete C30/37: E = 33 GPa = 33e6 kN/m^2
    model.materials = vec![Material {
        id: "MAT_CONC".to_string(),
        name: "C30/37".to_string(),
        source: LibrarySource::Custom,
        class: MaterialClass::Concrete,
        e_gpa: 33.0,
        g_gpa: 14.0,
        density_gamma: 25.0,
        alpha_t: 1.0e-5,
    }];

    // Columns: 40x40 cm => A = 0.16 m^2, I = 0.16 * 0.4^2 / 12 = 0.002133 m^4
    // Beams: 30x50 cm => A = 0.15 m^2, I = 0.3 * 0.5^3 / 12 = 0.003125 m^4
    model.sections = vec![
        Section {
            id: "SEC_COL".to_string(),
            name: "Col 40x40".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.16,
            inertia_i: 0.002133,
            shear_area_av: None,
            depth_h: Some(0.4),
        },
        Section {
            id: "SEC_BEAM".to_string(),
            name: "Beam 30x50".to_string(),
            source: LibrarySource::Custom,
            shape: SectionShape::Generic,
            area_a: 0.15,
            inertia_i: 0.003125,
            shear_area_av: None,
            depth_h: Some(0.5),
        },
    ];

    model.elements = vec![
        // Columns Storey 1
        Element::FrameMember {
            id: "C1".to_string(),
            node_i: "N1".to_string(),
            node_j: "N3".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "C2".to_string(),
            node_i: "N2".to_string(),
            node_j: "N4".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Beam Floor 1
        Element::FrameMember {
            id: "B1".to_string(),
            node_i: "N3".to_string(),
            node_j: "N4".to_string(),
            section_id: "SEC_BEAM".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Columns Storey 2
        Element::FrameMember {
            id: "C3".to_string(),
            node_i: "N3".to_string(),
            node_j: "N5".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "C4".to_string(),
            node_i: "N4".to_string(),
            node_j: "N6".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Beam Floor 2
        Element::FrameMember {
            id: "B2".to_string(),
            node_i: "N5".to_string(),
            node_j: "N6".to_string(),
            section_id: "SEC_BEAM".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Columns Storey 3
        Element::FrameMember {
            id: "C5".to_string(),
            node_i: "N5".to_string(),
            node_j: "N7".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        Element::FrameMember {
            id: "C6".to_string(),
            node_i: "N6".to_string(),
            node_j: "N8".to_string(),
            section_id: "SEC_COL".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
        // Beam Roof
        Element::FrameMember {
            id: "B3".to_string(),
            node_i: "N7".to_string(),
            node_j: "N8".to_string(),
            section_id: "SEC_BEAM".to_string(),
            material_id: "MAT_CONC".to_string(),
            release_i: EndRelease::default(),
            release_j: EndRelease::default(),
            offset_i: RigidOffset::default(),
            offset_j: RigidOffset::default(),
        },
    ];

    // Floor masses (Storey 1 & 2: 25 tonnes each node => 50 t/floor; Roof: 15 tonnes each node => 30 t)
    model.lumped_masses = vec![
        LumpedMass {
            node_id: "N3".to_string(),
            mass_x_t: 25.0,
            mass_y_t: 25.0,
            inertia_theta_tm2: 1.0,
        },
        LumpedMass {
            node_id: "N4".to_string(),
            mass_x_t: 25.0,
            mass_y_t: 25.0,
            inertia_theta_tm2: 1.0,
        },
        LumpedMass {
            node_id: "N5".to_string(),
            mass_x_t: 25.0,
            mass_y_t: 25.0,
            inertia_theta_tm2: 1.0,
        },
        LumpedMass {
            node_id: "N6".to_string(),
            mass_x_t: 25.0,
            mass_y_t: 25.0,
            inertia_theta_tm2: 1.0,
        },
        LumpedMass {
            node_id: "N7".to_string(),
            mass_x_t: 15.0,
            mass_y_t: 15.0,
            inertia_theta_tm2: 1.0,
        },
        LumpedMass {
            node_id: "N8".to_string(),
            mass_x_t: 15.0,
            mass_y_t: 15.0,
            inertia_theta_tm2: 1.0,
        },
    ];

    model
}

#[test]
fn test_sc006_modal_eigenanalysis_and_effective_mass() {
    let model = create_3storey_benchmark_frame();

    let dynamic_data = solve_modal_analysis(&model, 0.90)
        .expect("Modal analysis should succeed");

    let modes = &dynamic_data.modes;
    assert!(!modes.is_empty(), "Should extract vibration modes");
    assert!(modes.len() >= 3, "Should extract at least 3 modes for 3-storey frame");

    // 1. Fundamental period T1 should be around 0.25 - 0.40 s for a 3-storey frame
    let t1 = modes[0].period_s;
    assert!(
        t1 > 0.15 && t1 < 0.60,
        "Fundamental period T1 {} s outside realistic range (0.15 - 0.60s)",
        t1
    );

    // 2. Periods strictly decreasing: T1 > T2 > T3
    for i in 1..modes.len() {
        assert!(
            modes[i - 1].period_s >= modes[i].period_s,
            "Mode {} period ({}) must be >= mode {} period ({})",
            i,
            modes[i - 1].period_s,
            i + 1,
            modes[i].period_s
        );
    }

    // 3. Mode 1 participation and effective mass should be dominant in horizontal X
    let m1 = &modes[0];
    assert!(
        m1.effective_mass_ratio_x > 0.65,
        "First mode horizontal effective mass ratio {} should be > 65%",
        m1.effective_mass_ratio_x
    );

    // 4. Cumulative mass ratio check
    let last_mode = modes.last().unwrap();
    assert!(
        last_mode.cumulative_mass_ratio_x >= 0.90,
        "Cumulative mass ratio {} must reach >= 90% (SC-006 requirement)",
        last_mode.cumulative_mass_ratio_x
    );
}

#[test]
fn test_sc006_lateral_force_method() {
    let model = create_3storey_benchmark_frame();
    let ag = 0.24 * 9.81; // 0.24g peak ground acceleration
    let ground_type = "B";
    let q = 1.5;

    let res = solve_lateral_force(&model, ag, ground_type, q)
        .expect("Lateral force solve should succeed");

    assert_eq!(res.analysis_kind, AnalysisKind::LateralForce);
    assert!(!res.superposition_valid);

    // Check base shear
    let dyn_data = res.dynamic_data.as_ref().unwrap();
    assert!(dyn_data.base_shear_x_kn > 50.0, "Base shear must be significant");

    // Top story displacement (N7, N8) should be larger than first story (N3, N4)
    let u_top = res.nodal_displacements.iter().find(|n| n.node_id == "N7").unwrap().ux_m;
    let u_mid = res.nodal_displacements.iter().find(|n| n.node_id == "N5").unwrap().ux_m;
    let u_bot = res.nodal_displacements.iter().find(|n| n.node_id == "N3").unwrap().ux_m;

    assert!(u_top > u_mid && u_mid > u_bot, "Displacement must increase monotonically with height");
    assert!(u_bot > 0.0, "Base push produces positive horizontal displacement");
}

#[test]
fn test_sc006_response_spectrum_srss_and_cqc() {
    let model = create_3storey_benchmark_frame();
    let ag = 0.24 * 9.81;
    let ground_type = "B";
    let q = 1.5;
    let damping = 0.05;

    // Solve with SRSS
    let res_srss = solve_response_spectrum(&model, ag, ground_type, q, damping, false)
        .expect("SRSS solve should succeed");

    // Solve with CQC
    let res_cqc = solve_response_spectrum(&model, ag, ground_type, q, damping, true)
        .expect("CQC solve should succeed");

    let bs_srss = res_srss.dynamic_data.as_ref().unwrap().base_shear_x_kn;
    let bs_cqc = res_cqc.dynamic_data.as_ref().unwrap().base_shear_x_kn;

    assert!(bs_srss > 50.0 && bs_cqc > 50.0);

    // For well-separated planar modes (T1 ~ 0.3s, T2 ~ 0.09s, T3 ~ 0.04s),
    // CQC and SRSS base shears must match within 5%
    let diff = (bs_cqc - bs_srss).abs() / bs_srss;
    assert!(
        diff < 0.05,
        "SRSS ({}) and CQC ({}) base shears diverge by {}% (> 5%) for planar frame",
        bs_srss,
        bs_cqc,
        diff * 100.0
    );

    // Verify Der Kiureghian correlation properties
    assert!((der_kiureghian_correlation(10.0, 10.0, 0.05) - 1.0).abs() < 1e-6);
    let rho_12 = der_kiureghian_correlation(10.0, 30.0, 0.05);
    let rho_21 = der_kiureghian_correlation(30.0, 10.0, 0.05);
    assert!((rho_12 - rho_21).abs() < 1e-6, "Correlation matrix must be symmetric");
    assert!(rho_12 < 0.05, "Well-separated modes have near-zero correlation");
}
