---
type: project
title: BemApp - BEM-FEM Hybrid Platform for Inhomogeneous Elastodynamics
description: High-performance computational platform coupling BEM as a homogeneous predictor with FEM as an inhomogeneous corrector for 2D and 3D elastodynamics with GPU acceleration and interactive 3D visualization.
tags: [bem, fem, elastodynamics, tauri, angular, threejs, rust, gpu]
timestamp: 2026-10-28T05:18:00Z
state: bounded
children:
  - /capabilities/cap-geometry-meshing.md
  - /capabilities/cap-material-field.md
  - /capabilities/cap-solver-core.md
  - /capabilities/cap-parametric-benchmarking.md
  - /capabilities/cap-interactive-canvas.md
  - /capabilities/cap-project-persistence.md
---

# Intent

Provide an advanced, accessible, and high-performance desktop application for modeling, solving, and benchmarking linear and dynamic elastodynamic problems in inhomogeneous and functionally graded media. The application operationalizes the $(E + K)x = F$ paradigm, utilizing BEM to solve the homogeneous reference medium and FEM exclusively to evaluate volume perturbations without matrix inversion.

# Scope

### Included:
- Support for both 2D (plane stress/strain) and full 3D continuum mechanics.
- BEM homogeneous predictor coupled with FEM inhomogeneous corrector across Static, Time-domain, and Frequency/Laplace (Talbot contour + GMRES) domains.
- Pure FEM baseline direct solver for ground-truth verification and automated benchmarking.
- Conformal auto-tessellation (surface triangles for BEM + interior tetrahedra/triangles for FEM).
- Geometry creation via parametric benchmark presets and external CAD import (DXF, STEP, STL).
- Extensible material inhomogeneity engine adhering to the Open-Closed Principle (gradients, math expressions, inclusions).
- Interactive 3D visualization canvas powered by Three.js with face picking for boundary conditions, deformed shapes, stress contours, and live convergence streaming.
- Native standalone desktop distribution via Tauri v2 with Rust computational engine and Angular 20+ frontend.

### Excluded:
- Non-linear large deformation finite strain plasticity (focus is linear and functionally graded elastodynamics).
- Multi-physics coupling outside elastodynamics (e.g., thermal-fluid interaction).

# Relationships

- Child: [Geometry & Meshing](/capabilities/cap-geometry-meshing.md)
- Child: [Material Field Specification](/capabilities/cap-material-field.md)
- Child: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Child: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
- Child: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
- Child: [Project Persistence](/capabilities/cap-project-persistence.md)
