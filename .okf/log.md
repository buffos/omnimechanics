# OKF Knowledge Bundle Log

## 2026-10-28
- Initialized greenfield planning bundle for BemApp.
- Created root project concept `project.md` with bounded initial state.
- Modeled 6 top-level capabilities:
  - `cap-geometry-meshing`
  - `cap-material-field`
  - `cap-solver-core`
  - `cap-parametric-benchmarking`
  - `cap-interactive-canvas`
  - `cap-project-persistence`
- Incorporates user specifications: 2D & 3D dimension modes, CAD import (DXF/STEP/STL), interactive 3D face picking for boundary conditions, Open-Closed extensibility for inhomogeneity fields, and `.bem` project file persistence.
- Closed issue 001: Scaffolding of Tauri v2, Angular 22, Three.js, and Rust core with pnpm and Makefile. Human visual review approved.
- Updated cap-solver-core and cap-material-field: added Standalone BEM, Standalone FEM, BEM-FEM (BEM predictor), and FEM-BEM (FEM predictor) solvers, along with Vecchio-Collins (MCFT) concrete and J2 plasticity models.
