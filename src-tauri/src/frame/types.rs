use serde::{Deserialize, Serialize};

fn default_gravity() -> f64 {
    9.81
}

fn default_zero() -> f64 {
    0.0
}

fn default_one() -> f64 {
    1.0
}

/// Authoritative 2D Structural Model Aggregate Root
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuralModel {
    pub id: String,
    pub name: String,
    pub revision: u64,
    #[serde(default = "default_gravity")]
    pub gravity_g: f64,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub supports: Vec<Support>,
    #[serde(default)]
    pub load_cases: Vec<LoadCase>,
    #[serde(default)]
    pub lumped_masses: Vec<LumpedMass>,
}

impl Default for StructuralModel {
    fn default() -> Self {
        Self {
            id: "model-default".to_string(),
            name: "Default Model".to_string(),
            revision: 1,
            gravity_g: 9.81,
            nodes: Vec::new(),
            materials: Vec::new(),
            sections: Vec::new(),
            elements: Vec::new(),
            supports: Vec::new(),
            load_cases: Vec::new(),
            lumped_masses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum LibrarySource {
    Preset { catalog_id: String },
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaterialClass {
    Concrete,
    Steel,
    Timber,
    CableSteel,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub source: LibrarySource,
    pub class: MaterialClass,
    /// Young's modulus in GPa
    pub e_gpa: f64,
    /// Shear modulus in GPa
    pub g_gpa: f64,
    /// Unit weight / density in kN/m^3
    pub density_gamma: f64,
    /// Thermal expansion coefficient (1 / degC)
    #[serde(default)]
    pub alpha_t: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SectionShape {
    Ipe,
    Hea,
    Heb,
    HollowRect,
    HollowCirc,
    Rectangular,
    Circular,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub source: LibrarySource,
    pub shape: SectionShape,
    /// Cross-section area in m^2
    pub area_a: f64,
    /// Second moment of area (bending inertia) in m^4
    pub inertia_i: f64,
    /// Shear area in m^2 (optional)
    #[serde(default)]
    pub shear_area_av: Option<f64>,
    /// Outer depth / height in m (mandatory for thermal gradient loads)
    #[serde(default)]
    pub depth_h: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EndRelease {
    #[serde(default)]
    pub axial: bool,
    #[serde(default)]
    pub shear: bool,
    #[serde(default)]
    pub moment: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OffsetFrame {
    #[default]
    Local,
    Global,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct RigidOffset {
    #[serde(default)]
    pub dx: f64,
    #[serde(default)]
    pub dy: f64,
    #[serde(default)]
    pub frame: OffsetFrame,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AxialBehavior {
    #[default]
    Both,
    TensionOnly,
    CompressionOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode")]
pub enum CablePrestress {
    UnstressedLength { l0: f64 },
    SagRatio { sag_to_span: f64 },
    Pretension { t0: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum Element {
    FrameMember {
        id: String,
        node_i: String,
        node_j: String,
        section_id: String,
        material_id: String,
        #[serde(default)]
        release_i: EndRelease,
        #[serde(default)]
        release_j: EndRelease,
        #[serde(default)]
        offset_i: RigidOffset,
        #[serde(default)]
        offset_j: RigidOffset,
    },
    TrussBar {
        id: String,
        node_i: String,
        node_j: String,
        section_id: String,
        material_id: String,
        #[serde(default)]
        axial_behavior: AxialBehavior,
    },
    Spring {
        id: String,
        node_i: String,
        #[serde(default)]
        node_j: Option<String>,
        #[serde(default)]
        kx: f64,
        #[serde(default)]
        ky: f64,
        #[serde(default)]
        ktheta: f64,
        #[serde(default)]
        angle_deg: f64,
    },
    Cable {
        id: String,
        node_i: String,
        node_j: String,
        section_id: String,
        material_id: String,
        prestress: CablePrestress,
    },
}

impl Element {
    pub fn id(&self) -> &str {
        match self {
            Element::FrameMember { id, .. }
            | Element::TrussBar { id, .. }
            | Element::Spring { id, .. }
            | Element::Cable { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SupportKind {
    Fixed,
    Pinned,
    Roller {
        #[serde(default)]
        angle_deg: f64,
    },
    Guided {
        #[serde(default)]
        angle_deg: f64,
    },
    Elastic {
        #[serde(default)]
        kx: f64,
        #[serde(default)]
        ky: f64,
        #[serde(default)]
        ktheta: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Support {
    pub id: String,
    pub node_id: String,
    pub kind: SupportKind,
    #[serde(default)]
    pub unilateral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ActionType {
    Permanent,
    Variable {
        #[serde(default)]
        sub_category: Option<String>,
    },
    Accidental,
    Seismic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LoadFrame {
    #[default]
    Local,
    Global,
    Projected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum Load {
    NodalLoad {
        node_id: String,
        #[serde(default)]
        fx: f64,
        #[serde(default)]
        fy: f64,
        #[serde(default)]
        mz: f64,
    },
    MemberPointLoad {
        element_id: String,
        #[serde(default)]
        fx: f64,
        #[serde(default)]
        fy: f64,
        #[serde(default)]
        mz: f64,
        #[serde(default)]
        x_ratio: f64,
        #[serde(default)]
        frame: LoadFrame,
    },
    MemberDistributedLoad {
        element_id: String,
        #[serde(default)]
        qx1: f64,
        #[serde(default)]
        qy1: f64,
        #[serde(default)]
        qx2: f64,
        #[serde(default)]
        qy2: f64,
        #[serde(default = "default_zero")]
        a_ratio: f64,
        #[serde(default = "default_one")]
        b_ratio: f64,
        #[serde(default)]
        frame: LoadFrame,
    },
    SupportSettlement {
        support_id: String,
        #[serde(default)]
        dx: f64,
        #[serde(default)]
        dy: f64,
        #[serde(default)]
        dtheta: f64,
    },
    ThermalLoad {
        element_id: String,
        #[serde(default)]
        dt_uniform: f64,
        #[serde(default)]
        dt_gradient: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadCase {
    pub id: String,
    pub name: String,
    pub action_type: ActionType,
    #[serde(default)]
    pub self_weight: bool,
    #[serde(default)]
    pub loads: Vec<Load>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LumpedMass {
    pub node_id: String,
    #[serde(default)]
    pub mass_x_t: f64,
    #[serde(default)]
    pub mass_y_t: f64,
    #[serde(default)]
    pub inertia_theta_tm2: f64,
}

// ---------------------------------------------------------------------------
// Combinations & Analysis Inputs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadCaseFactor {
    pub load_case_id: String,
    pub factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Combination {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub factors: Vec<LoadCaseFactor>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisKind {
    LinearStatic,
    SecondOrderPDelta,
    LinearBuckling,
    Modal,
    ResponseSpectrum,
    LateralForce,
}

// ---------------------------------------------------------------------------
// Diagnostics & Kinematic Instability
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MechanismClassification {
    GlobalSwayOrHingeChain,
    LocalHingeMechanism,
    UnrestrainedDof,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeDisplacement {
    pub node_id: String,
    #[serde(default)]
    pub dux: f64,
    #[serde(default)]
    pub duy: f64,
    #[serde(default)]
    pub dtheta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MechanismMode {
    pub mode_index: usize,
    pub classification: MechanismClassification,
    pub node_displacements: Vec<NodeDisplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticMessage {
    pub code: String,
    #[serde(default)]
    pub element_id: Option<String>,
    #[serde(default)]
    pub node_id: Option<String>,
    pub message_el: String,
    pub message_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diagnostics {
    pub revision: u64,
    pub static_indeterminacy_degree: i32,
    pub kinematic_instability_degree: u32,
    pub free_dofs: usize,
    pub is_stable: bool,
    #[serde(default)]
    pub mechanism_modes: Vec<MechanismMode>,
    #[serde(default)]
    pub warnings: Vec<DiagnosticMessage>,
}

// ---------------------------------------------------------------------------
// Canonical Bilingual Error Envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanonicalError {
    pub code: String,
    pub message_el: String,
    pub message_en: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl CanonicalError {
    pub fn new(code: impl Into<String>, msg_el: impl Into<String>, msg_en: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message_el: msg_el.into(),
            message_en: msg_en.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

// ---------------------------------------------------------------------------
// Result Sets & Envelopes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodalDisplacementResult {
    pub node_id: String,
    pub ux_m: f64,
    pub uy_m: f64,
    pub theta_rad: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportReactionResult {
    pub support_id: String,
    pub node_id: String,
    pub rx_kn: f64,
    pub ry_kn: f64,
    pub mz_knm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemberEndForces {
    pub ni_kn: f64,
    pub qi_kn: f64,
    pub mi_knm: f64,
    pub nj_kn: f64,
    pub qj_kn: f64,
    pub mj_knm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StationResult {
    pub x_m: f64,
    pub n_kn: f64,
    pub q_kn: f64,
    pub m_knm: f64,
    pub ux_local_m: f64,
    pub uy_local_m: f64,
    pub theta_rad: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtremaPoint {
    pub x_m: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemberExtrema {
    pub max_m: ExtremaPoint,
    pub min_m: ExtremaPoint,
    pub max_q: ExtremaPoint,
    pub min_q: ExtremaPoint,
    pub max_n: ExtremaPoint,
    pub min_n: ExtremaPoint,
    pub max_abs_deflection_m: ExtremaPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemberResult {
    pub element_id: String,
    pub length_eff_m: f64,
    pub end_forces_local: MemberEndForces,
    pub stations: Vec<StationResult>,
    pub extrema: MemberExtrema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CableResult {
    pub element_id: String,
    pub unstressed_length_l0_m: f64,
    pub chord_length_m: f64,
    pub horizontal_tension_h_kn: f64,
    pub vertical_reaction_vi_kn: f64,
    pub vertical_reaction_vj_kn: f64,
    pub max_tension_kn: f64,
    pub mid_sag_m: f64,
    pub is_slack: bool,
    pub curve_points_global: Vec<Point2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BucklingMode {
    pub mode_index: usize,
    pub alpha_cr: f64,
    pub node_displacements: Vec<NodeDisplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VibrationMode {
    pub mode_index: usize,
    pub omega_rad_s: f64,
    pub frequency_hz: f64,
    pub period_s: f64,
    pub gamma_x: f64,
    pub gamma_y: f64,
    pub modal_mass_x_t: f64,
    pub modal_mass_y_t: f64,
    pub effective_mass_ratio_x: f64,
    pub effective_mass_ratio_y: f64,
    pub cumulative_mass_ratio_x: f64,
    pub cumulative_mass_ratio_y: f64,
    pub node_displacements: Vec<NodeDisplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicAnalysisData {
    #[serde(default)]
    pub modes: Vec<VibrationMode>,
    #[serde(default)]
    pub base_shear_x_kn: f64,
    #[serde(default)]
    pub base_shear_y_kn: f64,
    #[serde(default)]
    pub combination_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultSet {
    pub revision: u64,
    pub target_id: String,
    pub target_kind: String,
    pub analysis_kind: AnalysisKind,
    pub superposition_valid: bool,
    #[serde(default)]
    pub nodal_displacements: Vec<NodalDisplacementResult>,
    #[serde(default)]
    pub support_reactions: Vec<SupportReactionResult>,
    #[serde(default)]
    pub member_results: Vec<MemberResult>,
    #[serde(default)]
    pub cable_results: Vec<CableResult>,
    #[serde(default)]
    pub buckling_modes: Vec<BucklingMode>,
    #[serde(default)]
    pub dynamic_data: Option<DynamicAnalysisData>,
}

// ---------------------------------------------------------------------------
// Envelopes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvelopeStation {
    pub x_m: f64,
    pub max_m: f64,
    pub governing_max_m: String,
    pub min_m: f64,
    pub governing_min_m: String,
    pub max_q: f64,
    pub governing_max_q: String,
    pub min_q: f64,
    pub governing_min_q: String,
    pub max_n: f64,
    pub governing_max_n: String,
    pub min_n: f64,
    pub governing_min_n: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemberEnvelope {
    pub element_id: String,
    #[serde(default)]
    pub length_eff_m: f64,
    pub stations: Vec<EnvelopeStation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope {
    pub revision: u64,
    pub combinations_count: usize,
    pub member_envelopes: Vec<MemberEnvelope>,
}

// ---------------------------------------------------------------------------
// DSM Walkthrough Data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementWalkthrough {
    pub element_id: String,
    pub dof_indices_global: Vec<usize>,
    pub k_local: Vec<Vec<f64>>,
    pub t_matrix: Vec<Vec<f64>>,
    pub k_global: Vec<Vec<f64>>,
    pub f0_local: Vec<f64>,
    pub f0_global: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsmWalkthroughData {
    pub revision: u64,
    pub target_id: String,
    pub num_nodes: usize,
    pub num_dofs: usize,
    pub free_dofs: Vec<usize>,
    pub restrained_dofs: Vec<usize>,
    pub elements: Vec<ElementWalkthrough>,
    pub k_ff: Vec<Vec<f64>>,
    pub f_f: Vec<f64>,
    pub u_f: Vec<f64>,
    pub full_global_matrix_omitted: bool,
    #[serde(default)]
    pub k_global_dense: Option<Vec<Vec<f64>>>,
    pub step_descriptions_el: Vec<String>,
    pub step_descriptions_en: Vec<String>,
}

// ---------------------------------------------------------------------------
// Background Analysis Jobs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Converged,
    Failed,
    Cancelled,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisProgressEvent {
    pub job_id: String,
    pub model_revision: u64,
    pub kind: AnalysisKind,
    pub status: JobStatus,
    pub progress_percent: f32,
    #[serde(default)]
    pub current_target_id: Option<String>,
    #[serde(default)]
    pub current_iteration: u32,
    #[serde(default)]
    pub current_residual: Option<f64>,
    pub message_el: String,
    pub message_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobResultPayload {
    pub job_id: String,
    pub model_revision: u64,
    pub kind: AnalysisKind,
    pub status: JobStatus,
    #[serde(default)]
    pub result_sets: Vec<ResultSet>,
    #[serde(default)]
    pub envelope: Option<Envelope>,
    #[serde(default)]
    pub error: Option<CanonicalError>,
}

// ---------------------------------------------------------------------------
// IPC Request Payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpsertModelRequest {
    pub model: StructuralModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolveLinearRequest {
    pub model_revision: u64,
    pub model: Option<StructuralModel>,
    #[serde(default)]
    pub combinations: Vec<Combination>,
    #[serde(default)]
    pub compute_envelope: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolveLinearResponse {
    pub revision: u64,
    pub diagnostics: Diagnostics,
    pub result_sets: Vec<ResultSet>,
    #[serde(default)]
    pub envelope: Option<Envelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InspectMemberPointRequest {
    pub model_revision: u64,
    pub model: Option<StructuralModel>,
    pub element_id: String,
    pub x_m: f64,
    pub target_id: String,
    pub target_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InspectMemberPointResponse {
    pub revision: u64,
    pub element_id: String,
    pub x_m: f64,
    pub n_kn: f64,
    pub q_kn: f64,
    pub m_knm: f64,
    pub ux_local_m: f64,
    pub uy_local_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalkthroughRequest {
    pub model_revision: u64,
    pub model: Option<StructuralModel>,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartJobRequest {
    pub model_revision: u64,
    pub model: Option<StructuralModel>,
    pub kind: AnalysisKind,
    #[serde(default)]
    pub target_ids: Vec<String>,
    #[serde(default)]
    pub combinations: Vec<Combination>,
    #[serde(default)]
    pub compute_envelope: bool,
    #[serde(default)]
    pub damping_ratio: Option<f64>,
    #[serde(default)]
    pub combination_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartJobResponse {
    pub job_id: String,
    pub model_revision: u64,
    pub status: JobStatus,
}
