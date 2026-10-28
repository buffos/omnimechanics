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
  - **Static Regime:** Successive Picard iterations with incremental back-substitution until $\|\mathbf{F}_{\text{res}}\| < \epsilon$.
  - **Time-Domain Transient Regime:** Time-stepping with discrete boundary convolution history and within-step BEM-FEM corrector iterations.
  - **Transformed Complex Laplace Regime:** Modified Talbot contour integration in the complex plane with BEM-preconditioned GMRES, guaranteeing unconditional convergence and avoiding fictitious eigenfrequencies.
- **Reference Ground-Truth Solver:**
  - Pure FEM direct sparse solver ($\mathbf{K}_{\text{total}}\mathbf{u} = \mathbf{F}$) for comparative verification, error norm calculation, and benchmarking.
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
