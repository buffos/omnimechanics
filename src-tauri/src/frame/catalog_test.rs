use super::catalog::*;
use super::types::*;

#[test]
fn test_default_catalog_contents() {
    let cat = get_default_catalog();
    assert!(cat.materials.len() >= 12);
    assert!(cat.sections.len() >= 18);

    // Verify S355 steel
    let s355 = cat.materials.iter().find(|m| m.id == "MAT_S355").unwrap();
    assert_eq!(s355.class, MaterialClass::Steel);
    assert!((s355.e_gpa - 210.0).abs() < 1e-6);

    // Verify Concrete C30/37
    let c30 = cat.materials.iter().find(|m| m.id == "MAT_C30_37").unwrap();
    assert_eq!(c30.class, MaterialClass::Concrete);
    assert!((c30.e_gpa - 33.0).abs() < 1e-6);

    // Verify Cable steel Y1770
    let y1770 = cat.materials.iter().find(|m| m.id == "MAT_Y1770").unwrap();
    assert_eq!(y1770.class, MaterialClass::CableSteel);
    assert!((y1770.e_gpa - 160.0).abs() < 1e-6);

    // Verify IPE 300
    let ipe300 = cat.sections.iter().find(|s| s.id == "SEC_IPE_300").unwrap();
    assert_eq!(ipe300.shape, SectionShape::Ipe);
    assert!((ipe300.area_a - 0.00538).abs() < 1e-6);
    assert!((ipe300.inertia_i - 0.00008356).abs() < 1e-8);
    assert_eq!(ipe300.depth_h, Some(0.300));
}

#[test]
fn test_parametric_sections() {
    let rect = create_rectangular_section("SEC_R_30X60", "Rect 30x60", 0.30, 0.60);
    assert_eq!(rect.shape, SectionShape::Rectangular);
    assert!((rect.area_a - 0.18).abs() < 1e-6);
    let expected_i = 0.30 * 0.60f64.powi(3) / 12.0;
    assert!((rect.inertia_i - expected_i).abs() < 1e-8);
    assert_eq!(rect.depth_h, Some(0.60));

    let circ = create_circular_section("SEC_C_40", "Circ D40", 0.40);
    assert_eq!(circ.shape, SectionShape::Circular);
    let expected_area = std::f64::consts::PI * 0.20 * 0.20;
    assert!((circ.area_a - expected_area).abs() < 1e-6);
    assert_eq!(circ.depth_h, Some(0.40));
}
