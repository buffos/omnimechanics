---
type: capability
title: BEM Predictor -- FEM Corrector Hybrid Solver Core
description: Multi-domain computational solvers (Static, Time-domain convolution, Transformed Laplace-Talbot with GMRES) implementing the (E + K)x = F decomposition alongside a pure FEM verification solver.
tags: [bem, fem, solver, elastodynamics, gmres, talbot, laplace, time-domain, static]
timestamp: 2026-10-28T05:18:00Z
state: bounded
project: /project.md
parent: /project.md
---

# Intent

Execute high-accuracy elastodynamic simulations by combining the boundary efficiency of BEM for the homogeneous reference medium with the volume evaluation efficiency of FEM for material perturbations. Implement the modern frequency/Laplace domain formulation (Krylov GMRES preconditioning + Talbot contour) to eliminate historical divergence issues.

# Scope

### Included:
- **Core Partition Architecture:**
  - $\mathbf{E}$: Homogeneous elastodynamic operator solved via BEM (dense LU / forward-backward substitution).
  - $\mathbf{K}$: Volume perturbation operator evaluated via FEM ($\mathbf{F} = -\mathbf{K}_{\text{FEM}}\mathbf{u} - \mathbf{M}_{\text{FEM}}\ddot{\mathbf{u}}$) without matrix inversion.
  - $\mathbf{S}_{\text{shape}}$: Boundary consistency mass matrix converting nodal forces to surface tractions ($\mathbf{t}_\Gamma = \mathbf{S}_{\text{shape}}^{-1}\mathbf{F}_\Gamma$).
- **Solver Regimes:**
  - **1. Standalone BEM Solver:** Pure boundary element solution for homogeneous elastic and elastodynamic domains.
  - **2. Standalone FEM Solver:** Pure finite element solution for linear, inhomogeneous, and non-linear domains (with direct sparse LU and Arc-Length / Newton-Raphson solvers).
  - **3. BEM-FEM Hybrid Solver (BEM Predictor -- FEM Corrector):** BEM solves homogeneous reference field $\mathbf{E}_0$, FEM evaluates internal residual forces $\mathbf{F}_{\text{res}} = \mathbf{F}_{\text{ext}} - \int \mathbf{B}^T \boldsymbol{\sigma} d\Omega$.
  - **4. FEM-BEM Hybrid Solver (FEM Predictor -- BEM Corrector):** FEM solves global predictor $\mathbf{K} \mathbf{u} = \mathbf{F}$, BEM computes boundary reaction/radiation corrections.
  - **Transformed Complex Laplace Mode:** Modified Talbot contour integration with BEM-preconditioned GMRES for dynamic frequency/time response without resonance poles.
- **Hardware Acceleration:**
  - SIMD multi-threading via `rayon` for Gauss quadrature assembly.
  - Optional GPU compute kernels (via `wgpu`) for BEM dense matrix generation.

### Excluded:
- Non-linear contact mechanics and large strain kinematic transformations.

# Relationships

- Parent: [BemApp Platform](/project.md)
- Depends on: [Geometry & Meshing](/capabilities/cap-geometry-meshing.md)
- Depends on: [Material Field Specification](/capabilities/cap-material-field.md)
- Feeds into: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
- Feeds into: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
