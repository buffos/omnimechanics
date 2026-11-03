use std::collections::HashMap;

use super::types::*;
use crate::frame::types::CanonicalError;

/// Generates the default Greek National Annex (ELOT EN 1990/1998) profile
pub fn get_default_gr_profile() -> NationalAnnexProfile {
    let mut psi_factors = HashMap::new();

    // Table A1.1 / Greek NA values:
    // Category A & B (Domestic, residential, offices): 0.7, 0.5, 0.3
    psi_factors.insert("Imposed_A".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.50, psi_2: 0.30 });
    psi_factors.insert("Imposed_B".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.50, psi_2: 0.30 });
    // Category C & D (Congregation, shopping): 0.7, 0.7, 0.6
    psi_factors.insert("Imposed_C".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.70, psi_2: 0.60 });
    psi_factors.insert("Imposed_D".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.70, psi_2: 0.60 });
    // Category E (Storage): 1.0, 0.9, 0.8
    psi_factors.insert("Imposed_E".to_string(), PsiFactors { psi_0: 1.00, psi_1: 0.90, psi_2: 0.80 });
    // Category F (Traffic <= 30 kN): 0.7, 0.7, 0.6
    psi_factors.insert("Imposed_F".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.70, psi_2: 0.60 });
    // Category G (Traffic 30-160 kN): 0.7, 0.5, 0.3
    psi_factors.insert("Imposed_G".to_string(), PsiFactors { psi_0: 0.70, psi_1: 0.50, psi_2: 0.30 });
    // Category H (Roofs): 0.0, 0.0, 0.0
    psi_factors.insert("Imposed_H".to_string(), PsiFactors { psi_0: 0.00, psi_1: 0.00, psi_2: 0.00 });

    // Environmental
    psi_factors.insert("Snow".to_string(), PsiFactors { psi_0: 0.50, psi_1: 0.20, psi_2: 0.00 });
    psi_factors.insert("Wind".to_string(), PsiFactors { psi_0: 0.60, psi_1: 0.20, psi_2: 0.00 });
    psi_factors.insert("Temperature".to_string(), PsiFactors { psi_0: 0.60, psi_1: 0.50, psi_2: 0.00 });

    NationalAnnexProfile {
        code: "GR".to_string(),
        name: BilingualText::new(
            "Ελληνικό Εθνικό Προσάρτημα (ΕΛΟΤ EN 1990/1998)",
            "Greek National Annex (ELOT EN 1990/1998)",
        ),
        uls_equation_policy: UlsEquationPolicy::Single6_10,
        permanent_favorability_policy: PermanentFavorabilityPolicy::BothBranches,
        gamma_g_sup: 1.35,
        gamma_g_inf: 1.00,
        gamma_q_sup: 1.50,
        gamma_q_inf: 0.00,
        xi_reduction: 0.85,
        psi_factors,
    }
}

/// Returns the Action Category metadata catalog
pub fn get_action_category_catalog() -> Vec<ActionCategoryMetadata> {
    vec![
        ActionCategoryMetadata {
            key: "Imposed_A".to_string(),
            name: BilingualText::new("Κατηγορία Α: Χώροι κατοικιών", "Category A: Residential areas"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.5, psi_2: 0.3 },
        },
        ActionCategoryMetadata {
            key: "Imposed_B".to_string(),
            name: BilingualText::new("Κατηγορία Β: Χώροι γραφείων", "Category B: Office areas"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.5, psi_2: 0.3 },
        },
        ActionCategoryMetadata {
            key: "Imposed_C".to_string(),
            name: BilingualText::new("Κατηγορία C: Χώροι συνάθροισης κοινού", "Category C: Congregation areas"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.7, psi_2: 0.6 },
        },
        ActionCategoryMetadata {
            key: "Imposed_D".to_string(),
            name: BilingualText::new("Κατηγορία D: Χώροι εμπορικών καταστημάτων", "Category D: Shopping areas"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.7, psi_2: 0.6 },
        },
        ActionCategoryMetadata {
            key: "Imposed_E".to_string(),
            name: BilingualText::new("Κατηγορία Ε: Χώροι αποθήκευσης", "Category E: Storage areas"),
            default_psi: PsiFactors { psi_0: 1.0, psi_1: 0.9, psi_2: 0.8 },
        },
        ActionCategoryMetadata {
            key: "Imposed_F".to_string(),
            name: BilingualText::new("Κατηγορία F: Χώροι κυκλοφορίας οχημάτων <= 30 kN", "Category F: Traffic areas <= 30 kN"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.7, psi_2: 0.6 },
        },
        ActionCategoryMetadata {
            key: "Imposed_G".to_string(),
            name: BilingualText::new("Κατηγορία G: Χώροι κυκλοφορίας οχημάτων 30-160 kN", "Category G: Traffic areas 30-160 kN"),
            default_psi: PsiFactors { psi_0: 0.7, psi_1: 0.5, psi_2: 0.3 },
        },
        ActionCategoryMetadata {
            key: "Imposed_H".to_string(),
            name: BilingualText::new("Κατηγορία Η: Στέγες", "Category H: Roofs"),
            default_psi: PsiFactors { psi_0: 0.0, psi_1: 0.0, psi_2: 0.0 },
        },
        ActionCategoryMetadata {
            key: "Snow".to_string(),
            name: BilingualText::new("Χιόνι", "Snow"),
            default_psi: PsiFactors { psi_0: 0.5, psi_1: 0.2, psi_2: 0.0 },
        },
        ActionCategoryMetadata {
            key: "Wind".to_string(),
            name: BilingualText::new("Άνεμος", "Wind"),
            default_psi: PsiFactors { psi_0: 0.6, psi_1: 0.2, psi_2: 0.0 },
        },
        ActionCategoryMetadata {
            key: "Temperature".to_string(),
            name: BilingualText::new("Θερμοκρασία", "Temperature"),
            default_psi: PsiFactors { psi_0: 0.6, psi_1: 0.5, psi_2: 0.0 },
        },
    ]
}

/// Returns the Structural System q-factor guidance catalog
pub fn get_structural_system_guidance_catalog() -> Vec<StructuralSystemGuidanceEntry> {
    vec![
        StructuralSystemGuidanceEntry {
            id: "RcMomentFrameDcm".to_string(),
            name: BilingualText::new(
                "Ο/Σ Πλαισιακό Σύστημα (Μέση Πλαστιμότητα DCM)",
                "RC Moment Frame (Medium Ductility DCM)",
            ),
            default_q: 3.90,
        },
        StructuralSystemGuidanceEntry {
            id: "RcDualSystemDcm".to_string(),
            name: BilingualText::new(
                "Ο/Σ Μικτό Σύστημα Πλαισίων-Τοιχίων (DCM)",
                "RC Dual Frame-Wall System (DCM)",
            ),
            default_q: 3.60,
        },
        StructuralSystemGuidanceEntry {
            id: "RcDuctileWallDcm".to_string(),
            name: BilingualText::new(
                "Ο/Σ Σύστημα Πλαστίμων Τοιχίων (DCM)",
                "RC Ductile Wall System (DCM)",
            ),
            default_q: 3.00,
        },
        StructuralSystemGuidanceEntry {
            id: "LowDuctilityDcl".to_string(),
            name: BilingualText::new(
                "Σύστημα Χαμηλής Πλαστιμότητας (DCL)",
                "Low Ductility System (DCL)",
            ),
            default_q: 1.50,
        },
        StructuralSystemGuidanceEntry {
            id: "SteelMomentFrameDcm".to_string(),
            name: BilingualText::new(
                "Χαλύβδινο Πλαισιακό Σύστημα (DCM)",
                "Steel Moment Resisting Frame (DCM)",
            ),
            default_q: 4.00,
        },
        StructuralSystemGuidanceEntry {
            id: "SteelConcentricBracedDcm".to_string(),
            name: BilingualText::new(
                "Χαλύβδινο Σύστημα με Κεντρικούς Συνδέσμους Δυσκαμψίας (DCM)",
                "Steel Concentrically Braced Frame (DCM)",
            ),
            default_q: 4.00,
        },
    ]
}

/// Retrieves the National Annex Profile, Action Catalog, and System Guidance
pub fn get_default_na_profile(code_opt: Option<&str>) -> Result<GetNaProfileResponse, CanonicalError> {
    let code = code_opt.unwrap_or("GR").to_uppercase();
    if code != "GR" && code != "CUSTOM" {
        return Err(CanonicalError::new(
            "ACTIONS_UNKNOWN_NA_CODE",
            format!("Άγνωστος κωδικός Εθνικού Προσαρτήματος: '{}'.", code),
            format!("Unknown National Annex code: '{}'. Supported codes: 'GR', 'CUSTOM'.", code),
        ));
    }

    Ok(GetNaProfileResponse {
        profile: get_default_gr_profile(),
        action_catalog: get_action_category_catalog(),
        system_guidance_catalog: get_structural_system_guidance_catalog(),
    })
}

/// Validates a NationalAnnexProfile against Eurocode hard invariants (INV-ACT-01)
pub fn validate_na_profile(profile: &NationalAnnexProfile) -> Result<(), CanonicalError> {
    // 1. Gamma factors
    if profile.gamma_g_sup < profile.gamma_g_inf
        || profile.gamma_g_inf < 0.0
        || profile.gamma_q_sup <= 0.0
        || profile.gamma_q_inf < 0.0
    {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_GAMMA_FACTOR",
            "Μη έγκυρος επιμέρους συντελεστής ασφαλείας γ (πρέπει γG,sup >= γG,inf >= 0 και γQ,sup > 0).",
            "Invalid partial safety factor gamma (must satisfy gamma_G,sup >= gamma_G,inf >= 0 and gamma_Q,sup > 0).",
        ));
    }

    // 2. Xi reduction factor
    if profile.xi_reduction <= 0.0 || profile.xi_reduction > 1.0 {
        return Err(CanonicalError::new(
            "ACTIONS_INVALID_XI_FACTOR",
            "Ο συντελεστής μείωσης ξ πρέπει να ανήκει στο διάστημα (0, 1].",
            "Reduction factor xi must be in the interval (0, 1].",
        ));
    }

    // 3. Psi factor hierarchies (1.0 >= psi_0 >= psi_1 >= psi_2 >= 0.0)
    for (key, psi) in &profile.psi_factors {
        if psi.psi_0 < 0.0
            || psi.psi_0 > 1.0
            || psi.psi_1 < 0.0
            || psi.psi_1 > 1.0
            || psi.psi_2 < 0.0
            || psi.psi_2 > 1.0
            || psi.psi_1 > psi.psi_0
            || psi.psi_2 > psi.psi_1
        {
            return Err(CanonicalError::new(
                "ACTIONS_INVALID_PSI_FACTOR",
                format!(
                    "Οι συντελεστές ψ για την κατηγορία '{}' παραβιάζουν την ιεραρχία 1.0 >= ψ0 ({}) >= ψ1 ({}) >= ψ2 ({}) >= 0.0.",
                    key, psi.psi_0, psi.psi_1, psi.psi_2
                ),
                format!(
                    "Psi factors for category '{}' violate hierarchy 1.0 >= psi_0 ({}) >= psi_1 ({}) >= psi_2 ({}) >= 0.0.",
                    key, psi.psi_0, psi.psi_1, psi.psi_2
                ),
            ));
        }
    }

    Ok(())
}
