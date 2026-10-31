//! Domain models and Data Transfer Objects (DTOs) for Educational Structural Mechanics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::fraction::ExactFraction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportCondition {
    Pinned,   // Restrains Dx, Dy (Rot free)
    RollerX,  // Restrains Dy (Dx free, Rot free)
    RollerY,  // Restrains Dx (Dy free, Rot free)
    Clamped,  // Restrains Dx, Dy, Rot
    Free,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node2D {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Support2D {
    pub node_id: usize,
    pub condition: SupportCondition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member2D {
    pub id: usize,
    pub start_node: usize,
    pub end_node: usize,
    pub length: f64,
    pub ea: f64,
    pub ei: f64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointLoad2D {
    pub node_id: usize,
    pub fx: f64,
    pub fy: f64,
    pub mz: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistributedLoad2D {
    pub member_id: usize,
    pub q_start: f64, // Positive downward / perpendicular
    pub q_end: f64,
    pub direction: String, // "perpendicular", "global_y"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberPointLoad2D {
    pub member_id: usize,
    pub distance_from_start: f64,
    pub p: f64, // Positive downward
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PedagogicalPhase {
    DegreeOfIndeterminacy,
    SupportReactions,
    MethodOfJoints,
    MethodOfSections,
    InternalForcesPiecewise,
    CriticalExtrema,
    CrossIteration,
    KaniIteration,
    Summary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PedagogicalStep {
    pub step_index: usize,
    pub phase: PedagogicalPhase,
    pub title: String,
    pub rationale: String,
    pub active_entity_id: Option<String>,
    pub active_diagram: Option<String>,
    pub symbolic_equation: String,
    pub numerical_substitution: String,
    pub result_latex: String,
    pub solved_quantities: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactionResult {
    pub node_id: usize,
    pub rx: f64,
    pub ry: f64,
    pub rm: f64,
    pub rx_fraction: String,
    pub ry_fraction: String,
    pub rm_fraction: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalForceSample {
    pub x: f64,
    pub n: f64,
    pub q: f64,
    pub m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberDiagramData {
    pub member_id: usize,
    pub samples: Vec<InternalForceSample>,
    pub q_zero_locations: Vec<f64>,
    pub m_extreme_values: Vec<f64>,
    pub m_inflection_points: Vec<f64>,
    pub n_expr: String,
    pub q_expr: String,
    pub m_expr: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossCycleRow {
    pub cycle_index: usize,
    pub operation: String, // "Distribution", "CarryOver", "Sum"
    pub member_end_moments: HashMap<String, f64>, // e.g. "AB": -12.5, "BA": 24.3
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossTable {
    pub distribution_factors: HashMap<String, f64>,
    pub fixed_end_moments: HashMap<String, f64>,
    pub cycles: Vec<CrossCycleRow>,
    pub final_moments: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeachingInputConfig {
    pub preset_name: String,
    pub method: String, // "reactions", "diagrams", "joints", "ritter", "cross", "kani"
    pub language: Option<String>, // "el" | "en"
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub load_q: Option<f64>,
    pub load_p: Option<f64>,
    pub load_p_pos: Option<f64>,
    pub load_m: Option<f64>,
    pub custom_nodes: Option<Vec<Node2D>>,
    pub custom_members: Option<Vec<Member2D>>,
    pub custom_supports: Option<Vec<Support2D>>,
    pub custom_point_loads: Option<Vec<PointLoad2D>>,
    pub custom_dist_loads: Option<Vec<DistributedLoad2D>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeachingSolutionDto {
    pub system_type: String, // "beam", "truss", "frame", "continuous_beam"
    pub is_isostatic: bool,
    pub degree_of_indeterminacy: i32,
    pub nodes: Vec<Node2D>,
    pub members: Vec<Member2D>,
    pub supports: Vec<Support2D>,
    pub reactions: Vec<ReactionResult>,
    pub applied_point_loads: Vec<MemberPointLoad2D>,
    pub applied_dist_loads: Vec<DistributedLoad2D>,
    pub diagrams: Vec<MemberDiagramData>,
    pub steps: Vec<PedagogicalStep>,
    pub cross_table: Option<CrossTable>,
}

#[cfg(test)]
#[path = "types_test.rs"]
mod tests;
