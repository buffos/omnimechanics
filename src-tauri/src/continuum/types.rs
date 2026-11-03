use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopOrientation {
    CounterClockwise,
    Clockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopClosureState {
    OpenDraft,
    ClosedValid,
    ClosedInvalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex2D {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySegment {
    pub id: String,
    pub start_vertex_id: String,
    pub end_vertex_id: String,
    pub tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_element_size: Option<f64>,
    pub outward_normal: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryLoop {
    pub id: String,
    pub orientation: LoopOrientation,
    pub closure_state: LoopClosureState,
    pub vertices: Vec<Vertex2D>,
    pub segments: Vec<BoundarySegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CutoutShape {
    Circle {
        center_x: f64,
        center_y: f64,
        radius: f64,
        angular_divisions: usize,
    },
    Rectangle {
        center_x: f64,
        center_y: f64,
        width: f64,
        height: f64,
        rotation_deg: f64,
    },
    Polygon {
        vertices: Vec<Vertex2D>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cutout {
    pub id: String,
    pub shape: CutoutShape,
    #[serde(default = "default_hole_tag")]
    pub tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_element_size: Option<f64>,
}

fn default_hole_tag() -> String {
    "Hole".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionRegion {
    pub id: String,
    pub shape: CutoutShape,
    pub material_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub youngs_modulus_override_gpa: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poisson_ratio_override: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_element_size: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TractionFrame {
    Global,
    LocalNormalTangential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BoundaryConditionSpec {
    Free,
    DirichletDisplacement {
        fix_x: bool,
        fix_y: bool,
        ux_m: f64,
        uy_m: f64,
    },
    NeumannTraction {
        tx_kn_m2: f64,
        ty_kn_m2: f64,
        coordinate_frame: TractionFrame,
    },
    RobinImpedance {
        kn_kn_m3: f64,
        ks_kn_m3: f64,
        cn_kn_s_m3: f64,
        cs_kn_s_m3: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeElementKind {
    Tri3,
    Quad4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryElementKind {
    Line2,
    Line3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshingProfile {
    #[serde(default = "default_target_size")]
    pub target_element_size: f64,
    #[serde(default = "default_hole_factor")]
    pub hole_refinement_factor: f64,
    #[serde(default = "default_min_angle")]
    pub min_angle_deg: f64,
    #[serde(default = "default_vol_elem")]
    pub volume_element_type: VolumeElementKind,
    #[serde(default = "default_bnd_elem")]
    pub boundary_element_type: BoundaryElementKind,
}

fn default_target_size() -> f64 { 0.5 }
fn default_hole_factor() -> f64 { 0.35 }
fn default_min_angle() -> f64 { 25.0 }
fn default_vol_elem() -> VolumeElementKind { VolumeElementKind::Tri3 }
fn default_bnd_elem() -> BoundaryElementKind { BoundaryElementKind::Line2 }

impl Default for MeshingProfile {
    fn default() -> Self {
        Self {
            target_element_size: default_target_size(),
            hole_refinement_factor: default_hole_factor(),
            min_angle_deg: default_min_angle(),
            volume_element_type: default_vol_elem(),
            boundary_element_type: default_bnd_elem(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuumDomain {
    pub id: String,
    #[serde(default = "default_revision")]
    pub revision: u64,
    pub outer_boundary: BoundaryLoop,
    #[serde(default)]
    pub cutouts: Vec<Cutout>,
    #[serde(default)]
    pub inclusions: Vec<InclusionRegion>,
    #[serde(default)]
    pub tag_conditions: BTreeMap<String, BoundaryConditionSpec>,
    #[serde(default)]
    pub meshing_profile: MeshingProfile,
}

fn default_revision() -> u64 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateDomainRequest {
    pub domain: ContinuumDomain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewSegmentRequest {
    pub existing_vertices: Vec<Vertex2D>,
    pub candidate_point: Vertex2D,
    #[serde(default)]
    pub closing_loop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseLoopRequest {
    pub loop_id: String,
    pub vertices: Vec<Vertex2D>,
    #[serde(default)]
    pub is_hole: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSegmentPreview {
    pub is_valid: bool,
    pub intersecting_segment_indices: Vec<usize>,
    pub snap_closure_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainValidationDiagnostic {
    pub code: String,
    pub message_el: String,
    pub message_en: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainValidationResponse {
    pub is_valid_for_meshing: bool,
    pub diagnostics: Vec<DomainValidationDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshQualityReport {
    pub num_nodes: usize,
    pub num_boundary_elements: usize,
    pub num_volume_elements: usize,
    pub min_angle_deg: f64,
    pub max_angle_deg: f64,
    pub min_aspect_ratio: f64,
    pub max_aspect_ratio: f64,
    pub is_conformal: bool,
    pub outward_normals_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformalMeshHandoff {
    pub domain_id: String,
    pub domain_revision: u64,
    pub mesh_data: crate::mesh::types::MeshData,
    pub quality_report: MeshQualityReport,
    pub tag_element_counts: std::collections::HashMap<String, usize>,
}
