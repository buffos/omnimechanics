import fs from 'node:fs';

const content = fs.readFileSync('src-tauri/src/solver/oraiopoulos_bem_fem.rs', 'utf8');

const s2dStart = content.indexOf('/// Solves 2D elastostatics using Oraiopoulos');
const s3dStart = content.indexOf('/// Solves 3D elastostatics using Oraiopoulos');

const meshPart = content.slice(0, s2dStart);
const s2dPart = content.slice(s2dStart, s3dStart);
const s3dPart = content.slice(s3dStart);

fs.mkdirSync('src-tauri/src/solver/oraiopoulos', { recursive: true });

// 1. mesh.rs
let cleanMeshPart = meshPart
  .replace(/use super::anderson::/g, 'use crate::solver::anderson::')
  .replace(/use super::bem_solver::/g, 'use crate::solver::bem_solver::')
  .replace(/use super::fem_solver::/g, 'use crate::solver::fem_solver::')
  .replace(/use super::true_bem_fem::/g, 'use crate::solver::true_bem_fem::')
  .replace(/use super::types::/g, 'use crate::solver::types::');

fs.writeFileSync('src-tauri/src/solver/oraiopoulos/mesh.rs', cleanMeshPart);

// 2. solver_2d.rs
const s2dCode = `use super::mesh::*;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::anderson::AndersonAccelerator;
use crate::solver::bem_solver::{effective_bem_nu, integrate_poly_ln};
use crate::solver::fem_solver::{
    assemble_reference_stiffness_with_stress_state, compute_diagonal_inverse,
    compute_inhomogeneous_forces_with_stress_state, compute_nodal_von_mises,
    solve_pcg_sparse_with_preconditioner, sparse_mat_vec_mult,
};
use crate::solver::true_bem_fem::kelvin_2d_kernels;
use crate::solver::types::{ConvergenceLog, HybridAcceleration, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

` + s2dPart;
fs.writeFileSync('src-tauri/src/solver/oraiopoulos/solver_2d.rs', s2dCode);

// 3. solver_3d.rs
let cleanS3dPart = s3dPart.replace(
  'pub use solve_oraiopoulos_bem_fem as solve_decoupled_bem_fem_hybrid;',
  'pub use super::solver_2d::solve_oraiopoulos_bem_fem as solve_decoupled_bem_fem_hybrid;'
);

const s3dCode = `use super::mesh::*;
use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::ReferenceMaterial;
use crate::mesh::types::MeshData;
use crate::solver::anderson::AndersonAccelerator;
use crate::solver::fem_solver::{
    assemble_reference_stiffness_with_stress_state, compute_diagonal_inverse,
    compute_inhomogeneous_forces_with_stress_state, compute_nodal_von_mises,
    solve_pcg_sparse_with_preconditioner, sparse_mat_vec_mult,
};
use crate::solver::types::{ConvergenceLog, HybridAcceleration, SimulationResult, SolverMode, StressState2D};
use faer::prelude::SpSolver;
use faer::Mat;

` + cleanS3dPart;
fs.writeFileSync('src-tauri/src/solver/oraiopoulos/solver_3d.rs', s3dCode);

// 4. oraiopoulos/mod.rs
const modCode = `pub mod mesh;
pub mod solver_2d;
pub mod solver_3d;

pub use mesh::*;
pub use solver_2d::*;
pub use solver_3d::*;
`;
fs.writeFileSync('src-tauri/src/solver/oraiopoulos/mod.rs', modCode);

// 5. Replace oraiopoulos_bem_fem.rs with thin facade
const facadeCode = `pub use super::oraiopoulos::*;
`;
fs.writeFileSync('src-tauri/src/solver/oraiopoulos_bem_fem.rs', facadeCode);

console.log('Successfully split oraiopoulos_bem_fem!');
