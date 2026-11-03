use super::types::{LibrarySource, Material, MaterialClass, Section, SectionShape};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogResponse {
    pub materials: Vec<Material>,
    pub sections: Vec<Section>,
}

pub fn get_default_catalog() -> CatalogResponse {
    CatalogResponse {
        materials: get_standard_materials(),
        sections: get_standard_sections(),
    }
}

pub fn get_standard_materials() -> Vec<Material> {
    vec![
        // Steel Eurocode presets
        Material {
            id: "MAT_S235".to_string(),
            name: "Steel S235".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1993_S235".to_string(),
            },
            class: MaterialClass::Steel,
            e_gpa: 210.0,
            g_gpa: 80.77,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        },
        Material {
            id: "MAT_S275".to_string(),
            name: "Steel S275".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1993_S275".to_string(),
            },
            class: MaterialClass::Steel,
            e_gpa: 210.0,
            g_gpa: 80.77,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        },
        Material {
            id: "MAT_S355".to_string(),
            name: "Steel S355".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1993_S355".to_string(),
            },
            class: MaterialClass::Steel,
            e_gpa: 210.0,
            g_gpa: 80.77,
            density_gamma: 78.5,
            alpha_t: 1.2e-5,
        },
        // Concrete Eurocode presets
        Material {
            id: "MAT_C20_25".to_string(),
            name: "Concrete C20/25".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C20_25".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 30.0,
            g_gpa: 12.5,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        Material {
            id: "MAT_C25_30".to_string(),
            name: "Concrete C25/30".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C25_30".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 31.0,
            g_gpa: 12.92,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        Material {
            id: "MAT_C30_37".to_string(),
            name: "Concrete C30/37".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C30_37".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 33.0,
            g_gpa: 13.75,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        Material {
            id: "MAT_C35_45".to_string(),
            name: "Concrete C35/45".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C35_45".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 34.0,
            g_gpa: 14.17,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        Material {
            id: "MAT_C40_50".to_string(),
            name: "Concrete C40/50".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C40_50".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 35.0,
            g_gpa: 14.58,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        Material {
            id: "MAT_C50_60".to_string(),
            name: "Concrete C50/60".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1992_C50_60".to_string(),
            },
            class: MaterialClass::Concrete,
            e_gpa: 37.0,
            g_gpa: 15.42,
            density_gamma: 25.0,
            alpha_t: 1.0e-5,
        },
        // Timber Eurocode presets
        Material {
            id: "MAT_C24".to_string(),
            name: "Timber C24".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1995_C24".to_string(),
            },
            class: MaterialClass::Timber,
            e_gpa: 11.0,
            g_gpa: 0.69,
            density_gamma: 4.2,
            alpha_t: 0.5e-5,
        },
        Material {
            id: "MAT_GL24H".to_string(),
            name: "Glulam GL24h".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "EN1995_GL24H".to_string(),
            },
            class: MaterialClass::Timber,
            e_gpa: 11.5,
            g_gpa: 0.72,
            density_gamma: 4.5,
            alpha_t: 0.5e-5,
        },
        // High-strength structural cable steel
        Material {
            id: "MAT_Y1770".to_string(),
            name: "Cable Steel Y1770".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "ISO_Y1770".to_string(),
            },
            class: MaterialClass::CableSteel,
            e_gpa: 160.0,
            g_gpa: 61.5,
            density_gamma: 77.0,
            alpha_t: 1.2e-5,
        },
        Material {
            id: "MAT_Y1860".to_string(),
            name: "Cable Steel Y1860".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "ISO_Y1860".to_string(),
            },
            class: MaterialClass::CableSteel,
            e_gpa: 160.0,
            g_gpa: 61.5,
            density_gamma: 77.0,
            alpha_t: 1.2e-5,
        },
    ]
}

pub fn get_standard_sections() -> Vec<Section> {
    vec![
        // IPE series
        Section {
            id: "SEC_IPE_80".to_string(),
            name: "IPE 80".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_80".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 7.64e-4,
            inertia_i: 8.01e-7,
            shear_area_av: Some(3.58e-4),
            depth_h: Some(0.080),
        },
        Section {
            id: "SEC_IPE_100".to_string(),
            name: "IPE 100".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_100".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 1.03e-3,
            inertia_i: 1.71e-6,
            shear_area_av: Some(5.08e-4),
            depth_h: Some(0.100),
        },
        Section {
            id: "SEC_IPE_140".to_string(),
            name: "IPE 140".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_140".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 1.64e-3,
            inertia_i: 5.41e-6,
            shear_area_av: Some(7.64e-4),
            depth_h: Some(0.140),
        },
        Section {
            id: "SEC_IPE_200".to_string(),
            name: "IPE 200".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_200".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 2.85e-3,
            inertia_i: 1.94e-5,
            shear_area_av: Some(1.40e-3),
            depth_h: Some(0.200),
        },
        Section {
            id: "SEC_IPE_240".to_string(),
            name: "IPE 240".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_240".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 3.91e-3,
            inertia_i: 3.89e-5,
            shear_area_av: Some(1.91e-3),
            depth_h: Some(0.240),
        },
        Section {
            id: "SEC_IPE_300".to_string(),
            name: "IPE 300".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_300".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 5.38e-3,
            inertia_i: 8.356e-5,
            shear_area_av: Some(2.57e-3),
            depth_h: Some(0.300),
        },
        Section {
            id: "SEC_IPE_360".to_string(),
            name: "IPE 360".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_360".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 7.27e-3,
            inertia_i: 1.627e-4,
            shear_area_av: Some(3.51e-3),
            depth_h: Some(0.360),
        },
        Section {
            id: "SEC_IPE_400".to_string(),
            name: "IPE 400".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_400".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 8.45e-3,
            inertia_i: 2.313e-4,
            shear_area_av: Some(4.27e-3),
            depth_h: Some(0.400),
        },
        Section {
            id: "SEC_IPE_500".to_string(),
            name: "IPE 500".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_500".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 1.16e-2,
            inertia_i: 4.820e-4,
            shear_area_av: Some(5.99e-3),
            depth_h: Some(0.500),
        },
        Section {
            id: "SEC_IPE_600".to_string(),
            name: "IPE 600".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "IPE_600".to_string(),
            },
            shape: SectionShape::Ipe,
            area_a: 1.56e-2,
            inertia_i: 9.208e-4,
            shear_area_av: Some(8.38e-3),
            depth_h: Some(0.600),
        },
        // HEA series
        Section {
            id: "SEC_HEA_100".to_string(),
            name: "HEA 100".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEA_100".to_string(),
            },
            shape: SectionShape::Hea,
            area_a: 2.12e-3,
            inertia_i: 3.49e-6,
            shear_area_av: Some(7.56e-4),
            depth_h: Some(0.096),
        },
        Section {
            id: "SEC_HEA_140".to_string(),
            name: "HEA 140".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEA_140".to_string(),
            },
            shape: SectionShape::Hea,
            area_a: 3.14e-3,
            inertia_i: 1.033e-5,
            shear_area_av: Some(1.01e-3),
            depth_h: Some(0.133),
        },
        Section {
            id: "SEC_HEA_200".to_string(),
            name: "HEA 200".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEA_200".to_string(),
            },
            shape: SectionShape::Hea,
            area_a: 5.38e-3,
            inertia_i: 3.692e-5,
            shear_area_av: Some(1.81e-3),
            depth_h: Some(0.190),
        },
        Section {
            id: "SEC_HEA_300".to_string(),
            name: "HEA 300".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEA_300".to_string(),
            },
            shape: SectionShape::Hea,
            area_a: 1.13e-2,
            inertia_i: 1.826e-4,
            shear_area_av: Some(3.73e-3),
            depth_h: Some(0.290),
        },
        // HEB series
        Section {
            id: "SEC_HEB_100".to_string(),
            name: "HEB 100".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEB_100".to_string(),
            },
            shape: SectionShape::Heb,
            area_a: 2.60e-3,
            inertia_i: 4.50e-6,
            shear_area_av: Some(9.04e-4),
            depth_h: Some(0.100),
        },
        Section {
            id: "SEC_HEB_200".to_string(),
            name: "HEB 200".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEB_200".to_string(),
            },
            shape: SectionShape::Heb,
            area_a: 7.81e-3,
            inertia_i: 5.696e-5,
            shear_area_av: Some(2.48e-3),
            depth_h: Some(0.200),
        },
        Section {
            id: "SEC_HEB_300".to_string(),
            name: "HEB 300".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "HEB_300".to_string(),
            },
            shape: SectionShape::Heb,
            area_a: 1.49e-2,
            inertia_i: 2.517e-4,
            shear_area_av: Some(4.74e-3),
            depth_h: Some(0.300),
        },
        // Hollow RHS
        Section {
            id: "SEC_RHS_100X50X4".to_string(),
            name: "RHS 100x50x4".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "RHS_100X50X4".to_string(),
            },
            shape: SectionShape::HollowRect,
            area_a: 1.10e-3,
            inertia_i: 1.49e-6,
            shear_area_av: Some(7.33e-4),
            depth_h: Some(0.100),
        },
        Section {
            id: "SEC_RHS_200X100X8".to_string(),
            name: "RHS 200x100x8".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "RHS_200X100X8".to_string(),
            },
            shape: SectionShape::HollowRect,
            area_a: 4.47e-3,
            inertia_i: 2.37e-5,
            shear_area_av: Some(2.98e-3),
            depth_h: Some(0.200),
        },
        // Hollow CHS
        Section {
            id: "SEC_CHS_88_9X4".to_string(),
            name: "CHS 88.9x4".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "CHS_88_9X4".to_string(),
            },
            shape: SectionShape::HollowCirc,
            area_a: 1.07e-3,
            inertia_i: 9.77e-7,
            shear_area_av: Some(6.82e-4),
            depth_h: Some(0.0889),
        },
        Section {
            id: "SEC_CHS_168_3X6_3".to_string(),
            name: "CHS 168.3x6.3".to_string(),
            source: LibrarySource::Preset {
                catalog_id: "CHS_168_3X6_3".to_string(),
            },
            shape: SectionShape::HollowCirc,
            area_a: 3.21e-3,
            inertia_i: 1.07e-5,
            shear_area_av: Some(2.04e-3),
            depth_h: Some(0.1683),
        },
    ]
}

pub fn create_rectangular_section(id: &str, name: &str, b_m: f64, h_m: f64) -> Section {
    let area = b_m * h_m;
    let inertia = b_m * h_m.powi(3) / 12.0;
    let shear_area = (5.0 / 6.0) * area;
    Section {
        id: id.to_string(),
        name: name.to_string(),
        source: LibrarySource::Custom,
        shape: SectionShape::Rectangular,
        area_a: area,
        inertia_i: inertia,
        shear_area_av: Some(shear_area),
        depth_h: Some(h_m),
    }
}

pub fn create_circular_section(id: &str, name: &str, d_m: f64) -> Section {
    let radius = d_m / 2.0;
    let area = std::f64::consts::PI * radius * radius;
    let inertia = std::f64::consts::PI * d_m.powi(4) / 64.0;
    let shear_area = 0.9 * area;
    Section {
        id: id.to_string(),
        name: name.to_string(),
        source: LibrarySource::Custom,
        shape: SectionShape::Circular,
        area_a: area,
        inertia_i: inertia,
        shear_area_av: Some(shear_area),
        depth_h: Some(d_m),
    }
}
