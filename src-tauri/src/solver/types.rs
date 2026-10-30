use crate::material::types::{MaterialStrategyConfig, ReferenceMaterial};
use crate::mesh::types::MeshData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BemElementType {
    Constant,
    Linear,
    LinearDoubleNodes,
    Quadratic,
    QuadraticDiscontinuous,
    Tri3,
    Quad4,
}

impl Default for BemElementType {
    fn default() -> Self {
        BemElementType::Constant
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StressState2D {
    PlaneStress,
    PlaneStrain,
}

impl Default for StressState2D {
    fn default() -> Self {
        StressState2D::PlaneStress
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HybridAcceleration {
    None,
    Relaxation(f64),
    Anderson { depth: usize, beta: f64 },
}

impl Default for HybridAcceleration {
    fn default() -> Self {
        HybridAcceleration::Anderson { depth: 3, beta: 0.65 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverMode {
    PureBEM,
    PureFEM,
    BEMFEM, // BEM Predictor -- FEM Corrector
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarmonicSolverMode {
    PureFEM,
    PureBEM,
    Robin,
}

impl Default for HarmonicSolverMode {
    fn default() -> Self {
        HarmonicSolverMode::PureFEM
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequest {
    pub solver_mode: SolverMode,
    pub mesh: MeshData,
    pub reference_material: ReferenceMaterial,
    pub strategy: MaterialStrategyConfig,
    /// Total external applied load magnitude (N in 2D/3D, e.g. 10,000 N downward)
    pub load_magnitude: f64,
    /// Maximum iterations for non-linear / hybrid solver
    pub max_iterations: usize,
    /// Convergence tolerance for residual norm
    pub tolerance: f64,
    /// Boundary element formulation for BEM: Constant, Linear, Quadratic, etc.
    #[serde(default)]
    pub bem_element_type: Option<BemElementType>,
    /// 2D stress state assumption: PlaneStress (thin plates/beams) or PlaneStrain (thick slabs/tunnels)
    #[serde(default)]
    pub stress_state: Option<StressState2D>,
    /// Optional acceleration scheme for iterative hybrid BEM-FEM solver (Relaxation or Anderson)
    #[serde(default)]
    pub hybrid_acceleration: Option<HybridAcceleration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceLog {
    pub iteration: usize,
    pub residual_norm: f64,
    pub relative_error: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub solver_mode: SolverMode,
    pub converged: bool,
    pub iterations: usize,
    /// Nodal displacement vectors [ux, uy, uz] (meters)
    pub displacements: Vec<[f64; 3]>,
    /// Deformed mesh coordinates [x + ux, y + uy, z + uz]
    pub deformed_nodes: Vec<[f64; 3]>,
    /// Nodal or element von Mises stress (MPa)
    pub von_mises_stresses: Vec<f64>,
    /// Maximum displacement magnitude (m)
    pub max_displacement: f64,
    /// Maximum von Mises stress (MPa)
    pub max_von_mises: f64,
    /// Iteration convergence history
    pub convergence_history: Vec<ConvergenceLog>,
    /// Execution time in milliseconds
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicSimulationRequest {
    pub mesh: MeshData,
    pub reference_material: ReferenceMaterial,
    pub strategy: MaterialStrategyConfig,
    pub frequency_hz: f64,
    pub load_magnitude: f64,
    pub max_gmres_iter: usize,
    pub tolerance: f64,
    #[serde(default)]
    pub use_robin_coupling: Option<bool>,
    #[serde(default)]
    pub solver_mode: Option<HarmonicSolverMode>,
    #[serde(default)]
    pub bem_element_type: Option<BemElementType>,
    #[serde(default)]
    pub stress_state: Option<StressState2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicSimulationResult {
    pub converged: bool,
    pub iterations: usize,
    pub relative_residual: f64,
    pub frequency_hz: f64,
    /// Complex displacement amplitude |u| for each node
    pub amplitudes: Vec<[f64; 3]>,
    /// Phase angles (radians) for each node
    pub phase_lags: Vec<[f64; 3]>,
    /// Instantaneous deformed mesh coordinates at peak amplitude
    pub deformed_nodes: Vec<[f64; 3]>,
    /// Nodal von Mises stress amplitude (MPa)
    pub von_mises_stresses: Vec<f64>,
    pub max_amplitude: f64,
    pub max_von_mises: f64,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationFrame {
    pub time: f64,
    pub displacements: Vec<[f64; 3]>,
    pub deformed_nodes: Vec<[f64; 3]>,
    pub von_mises_stresses: Vec<f64>,
    pub max_displacement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientSimulationRequest {
    pub mesh: MeshData,
    pub reference_material: ReferenceMaterial,
    pub strategy: MaterialStrategyConfig,
    pub total_time: f64,
    pub time_steps: usize,
    pub num_talbot_points: usize,
    pub load_magnitude: f64,
    pub pulse_duration: f64,
    #[serde(default)]
    pub excitation_type: Option<String>,
    #[serde(default)]
    pub excitation_frequency: Option<f64>,
    #[serde(default)]
    pub integration_method: Option<String>,
    #[serde(default)]
    pub hht_alpha: Option<f64>,
    #[serde(default)]
    pub rayleigh_alpha: Option<f64>,
    #[serde(default)]
    pub rayleigh_beta: Option<f64>,
    #[serde(default)]
    pub solver_mode: Option<String>,
    #[serde(default)]
    pub bem_element_type: Option<BemElementType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientSimulationResult {
    pub time_points: Vec<f64>,
    pub frames: Vec<SimulationFrame>,
    pub max_overall_displacement: f64,
    pub execution_time_ms: f64,
}
