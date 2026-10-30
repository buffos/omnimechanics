---
type: capability
title: BEM Predictor -- FEM Corrector Hybrid Solver Core
description: Multi-domain computational solvers (Static, Time-domain convolution, Transformed Laplace-Talbot with GMRES) implementing the (E + K)x = F decomposition alongside a pure FEM verification solver.
tags: [bem, fem, solver, elastodynamics, gmres, talbot, laplace, time-domain, static]
timestamp: 2026-10-28T21:30:00Z
state: implemented
project: /project.md
parent: /project.md
issues:
  - docs/agents/issues/done/20261028-004-solver-core-bem-fem.md
  - docs/agents/issues/done/20261028-005-dynamic-laplace-talbot-solver.md
  - docs/agents/issues/done/20261029-006-dynamic-robin-schwarz-coupling.md
  - docs/agents/issues/done/20261029-007-classical-time-marching-newmark.md
---

# Intent

Execute high-accuracy elastodynamic simulations by combining the boundary efficiency of BEM for the homogeneous reference medium with the volume evaluation efficiency of FEM for material perturbations. Implement the modern frequency/Laplace domain formulation (Krylov GMRES preconditioning + Talbot contour) to eliminate historical divergence issues.

# Scope

### Included:
- **Core Partition Architecture (Operator Splitting on Inhomogeneous Continuum):**
  - $\mathbf{E}_0$: Homogeneous linear elastostatic/dynamic reference operator solved strictly on the boundary $\Gamma = \partial\Omega$ via BEM (dense LU / forward-backward substitution).
  - $\mathbf{K}_{\text{pert}}$: Volume perturbation operator evaluated strictly at element Gauss points via FEM ($\mathbf{F}_{\text{int}} = \int \mathbf{B}^T \boldsymbol{\sigma} d\Omega$) without global volume matrix inversion.
  - $\mathbf{S}_{\text{shape}}$: Variational boundary consistency mass matrix converting nodal forces to surface tractions ($\mathbf{t}_\Gamma = \mathbf{S}_{\text{shape}}^{-1}\mathbf{F}_\Gamma$).
- **Solver Regimes:**
  - **1. Standalone BEM Solver:** Pure boundary element solution for homogeneous elastic and elastodynamic domains.
  - **2. Standalone FEM Solver:** Pure finite element solution for linear, inhomogeneous, and non-linear domains (with direct sparse LU and Arc-Length / Newton-Raphson solvers).
  - **3. BEM-FEM Hybrid Solver (BEM Predictor -- FEM Corrector):** BEM solves homogeneous reference field $\mathbf{E}_0$, FEM evaluates internal residual forces $\mathbf{F}_{\text{res}} = \mathbf{F}_{\text{ext}} - \int \mathbf{B}^T \boldsymbol{\sigma} d\Omega$.
  - **4. FEM-BEM Hybrid Solver (FEM Predictor -- BEM Corrector):** FEM solves global predictor $\mathbf{K} \mathbf{u} = \mathbf{F}$, BEM computes boundary reaction corrections.
  - **5. Harmonic Frequency Mode (GMRES):** Complex Krylov iterative GMRES solver for indefinite dynamic operators $(\mathbf{K} - \omega^2\mathbf{M} + i\omega\mathbf{C})$ with exact Dirichlet degrees-of-freedom filtering.
  - **6. Transformed Complex Laplace-Talbot Mode:** Modified Talbot contour integration with parallel rayon execution for non-recursive, zero error-accumulation transient wave evaluation.
  - **7. Classical Direct Time-Marching Mode:** Step-by-step implicit temporal integration (Newmark-$\beta$ average acceleration $\gamma=1/2, \beta=1/4$, and HHT-$\alpha$ numerical dissipation) for direct temporal benchmark verification.
  - **8. Unbounded Medium / Soil-Structure Interaction (Optional Robin Extension):** Impedance-matched dynamic Robin boundary conditions $\mathbf{t}_\Gamma + \mathbf{Z}_{\text{ext}}(s)\mathbf{u}_\Gamma = \mathbf{g}_\Gamma$ applied strictly to designated radiating boundary tags with specified external continuum properties ($E_{\text{ext}}, \nu_{\text{ext}}, \rho_{\text{ext}}$).

### Excluded:
- Non-linear contact mechanics and large strain kinematic transformations.
- Application of absorbing radiation impedance to traction-free natural boundaries of finite bodies in air/vacuum.

# Relationships

- Parent: [BemApp Platform](/project.md)
- Depends on: [Geometry & Meshing](/capabilities/cap-geometry-meshing.md)
- Depends on: [Material Field Specification](/capabilities/cap-material-field.md)
- Feeds into: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
- Feeds into: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
