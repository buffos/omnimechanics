pub mod mesh;
pub mod solver_2d;
pub mod solver_3d;

pub use mesh::*;
pub use solver_2d::*;
pub use solver_3d::*;
pub use solver_2d::solve_oraiopoulos_bem_fem as solve_decoupled_bem_fem_hybrid;
