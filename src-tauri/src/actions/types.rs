use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BilingualText {
    pub el: String,
    pub en: String,
}

impl BilingualText {
    pub fn new(el: impl Into<String>, en: impl Into<String>) -> Self {
        Self {
            el: el.into(),
            en: en.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum ActionType {
    Permanent,
    Imposed {
        category: String,
    },
    Snow,
    Wind,
    Temperature,
    Seismic,
    Accidental,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadCaseDescriptor {
    pub id: String,
    pub code: String,
    pub name: BilingualText,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PsiFactors {
    pub psi_0: f64,
    pub psi_1: f64,
    pub psi_2: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum UlsEquationPolicy {
    #[default]
    Single6_10,
    Twin6_10a_6_10b,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PermanentFavorabilityPolicy {
    #[default]
    BothBranches,
    UnfavorableOnly,
    PerCasePermutation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NationalAnnexProfile {
    pub code: String,
    pub name: BilingualText,
    pub uls_equation_policy: UlsEquationPolicy,
    pub permanent_favorability_policy: PermanentFavorabilityPolicy,
    pub gamma_g_sup: f64,
    pub gamma_g_inf: f64,
    pub gamma_q_sup: f64,
    pub gamma_q_inf: f64,
    pub xi_reduction: f64,
    pub psi_factors: HashMap<String, PsiFactors>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LimitState {
    ULS_STR,
    ULS_GEO,
    SLS_CHAR,
    SLS_FREQ,
    SLS_QP,
    SEISMIC,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombinationOrigin {
    Generated,
    UserEdited,
    Manual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombinationEquation {
    Eq6_10,
    Eq6_10a,
    Eq6_10b,
    Eq6_12b,
    Eq6_14b,
    Eq6_15b,
    Eq6_16b,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CombinationRole {
    PermanentUnfavorable,
    PermanentFavorable,
    LeadingVariable,
    AccompanyingVariable,
    SeismicPrimary,
    SeismicAccompanying,
    ManualFactor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombinationTerm {
    pub load_case_id: String,
    pub factor: f64,
    pub role: CombinationRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Combination {
    pub id: String,
    pub name: BilingualText,
    pub limit_state: LimitState,
    pub origin: CombinationOrigin,
    pub equation: Option<CombinationEquation>,
    pub leading_load_case_id: Option<String>,
    pub terms: Vec<CombinationTerm>,
    pub symbolic_formula: BilingualText,
    #[serde(default)]
    pub generation_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvelopeSpecification {
    pub id: String,
    pub name: BilingualText,
    pub limit_state: LimitState,
    pub combination_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombinationSet {
    pub revision: u64,
    pub combinations: Vec<Combination>,
    pub envelopes: Vec<EnvelopeSpecification>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuralSystemGuidanceEntry {
    pub id: String,
    pub name: BilingualText,
    pub default_q: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionCategoryMetadata {
    pub key: String,
    pub name: BilingualText,
    pub default_psi: PsiFactors,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombinationDraft {
    pub id: Option<String>,
    pub name: BilingualText,
    pub limit_state: LimitState,
    pub terms: Vec<CombinationTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetNaProfileResponse {
    pub profile: NationalAnnexProfile,
    pub action_catalog: Vec<ActionCategoryMetadata>,
    pub system_guidance_catalog: Vec<StructuralSystemGuidanceEntry>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HazardZone {
    ZoneI,
    ZoneII,
    ZoneIII,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GroundType {
    A,
    B,
    C,
    D,
    E,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportanceClass {
    ClassI,
    ClassII,
    ClassIII,
    ClassIV,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModalCombinationRule {
    SRSS,
    CQC,
}

fn default_qv() -> f64 {
    1.50
}

fn default_psi_e_phi() -> f64 {
    1.00
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeismicSettings {
    pub zone: HazardZone,
    pub ag_reference_g: f64,
    pub ground_type: GroundType,
    pub soil_factor_s: f64,
    pub tb_s: f64,
    pub tc_s: f64,
    pub td_s: f64,
    pub importance_class: ImportanceClass,
    pub gamma_i: f64,
    #[serde(default)]
    pub system_guidance: Option<String>,
    pub behavior_factor_q: f64,
    pub damping_ratio_percent: f64,
    pub beta_lower_bound: f64,
    pub modal_combination_rule: ModalCombinationRule,
    #[serde(default)]
    pub include_vertical_component: bool,
    #[serde(default = "default_qv")]
    pub vertical_behavior_factor_qv: f64,
    #[serde(default = "default_psi_e_phi")]
    pub psi_e_phi: f64,
}

impl Default for SeismicSettings {
    fn default() -> Self {
        Self {
            zone: HazardZone::ZoneII,
            ag_reference_g: 0.24,
            ground_type: GroundType::B,
            soil_factor_s: 1.20,
            tb_s: 0.15,
            tc_s: 0.50,
            td_s: 2.00,
            importance_class: ImportanceClass::ClassII,
            gamma_i: 1.00,
            system_guidance: Some("RcMomentFrameDcm".to_string()),
            behavior_factor_q: 3.90,
            damping_ratio_percent: 5.0,
            beta_lower_bound: 0.20,
            modal_combination_rule: ModalCombinationRule::CQC,
            include_vertical_component: false,
            vertical_behavior_factor_qv: 1.50,
            psi_e_phi: 1.00,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpectrumPoint {
    pub period_s: f64,
    pub se_m_s2: f64,
    pub sd_m_s2: f64,
    pub se_g: f64,
    pub sd_g: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignSpectrum {
    pub ag_m_s2: f64,
    pub ag_g: f64,
    #[serde(default)]
    pub avg_m_s2: Option<f64>,
    pub eta_damping: f64,
    pub soil_factor_s: f64,
    pub tb_s: f64,
    pub tc_s: f64,
    pub td_s: f64,
    pub beta_lower_bound: f64,
    pub horizontal_curve: Vec<SpectrumPoint>,
    #[serde(default)]
    pub vertical_curve: Option<Vec<SpectrumPoint>>,
    #[serde(default)]
    pub queried_ordinates: Vec<SpectrumPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeDesignSpectrumRequest {
    pub settings: SeismicSettings,
    #[serde(default)]
    pub max_period_s: Option<f64>,
    #[serde(default)]
    pub num_points: Option<usize>,
    #[serde(default)]
    pub query_periods_s: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeDesignSpectrumResponse {
    pub spectrum: DesignSpectrum,
    pub queried_ordinates: Vec<SpectrumPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeismicMassTerm {
    pub load_case_id: String,
    pub factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeismicMassCombinationRule {
    pub terms: Vec<SeismicMassTerm>,
    pub symbolic_formula: BilingualText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeSeismicMassRuleRequest {
    pub load_cases: Vec<LoadCaseDescriptor>,
    pub profile: NationalAnnexProfile,
    pub settings: SeismicSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MassLevelInput {
    pub level_name: String,
    pub elevation_z_m: f64,
    pub mass_tonnes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LateralStoryForce {
    pub level_name: String,
    pub elevation_z_m: f64,
    pub mass_tonnes: f64,
    pub force_kn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LateralForceDistributionResult {
    pub fundamental_period_t1_s: f64,
    pub lambda_correction: f64,
    pub sd_t1_m_s2: f64,
    pub total_mass_tonnes: f64,
    pub base_shear_kn: f64,
    pub story_forces: Vec<LateralStoryForce>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeLateralForcesRequest {
    pub settings: SeismicSettings,
    pub fundamental_period_t1_s: f64,
    pub n_storeys: usize,
    pub levels: Vec<MassLevelInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComputeCqcMatrixRequest {
    pub rule: String,
    pub modal_periods_s: Vec<f64>,
    pub damping_ratio_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModalCorrelationMatrix {
    pub rule: String,
    pub modal_periods_s: Vec<f64>,
    pub matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerateCombinationsRequest {
    pub load_cases: Vec<LoadCaseDescriptor>,
    pub profile: NationalAnnexProfile,
    #[serde(default)]
    pub seismic_settings: Option<SeismicSettings>,
    #[serde(default)]
    pub existing_combinations: Vec<Combination>,
    #[serde(default)]
    pub include_uls_geo: bool,
    #[serde(default)]
    pub reset_user_edited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveCustomCombinationRequest {
    pub combination: CombinationDraft,
    pub load_cases: Vec<LoadCaseDescriptor>,
    #[serde(default)]
    pub existing_combinations: Vec<Combination>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteOrResetCombinationRequest {
    pub combination_id: String,
    pub action: String, // "Delete" | "ResetToGenerated"
    pub load_cases: Vec<LoadCaseDescriptor>,
    pub profile: NationalAnnexProfile,
    #[serde(default)]
    pub seismic_settings: Option<SeismicSettings>,
    #[serde(default)]
    pub existing_combinations: Vec<Combination>,
}
