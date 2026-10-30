---
type: project
title: BemApp - BEM-FEM Hybrid Platform for Inhomogeneous Elastodynamics & Structural Mechanics
description: High-performance computational platform coupling BEM as a homogeneous predictor with FEM as an inhomogeneous corrector for 2D and 3D elastodynamics, featuring 3 distinct application modes (Benchmark Workbench, Custom CAD Drafting & Import, and Structural Mechanics / Eurocode Step-by-Step Educational Studio).
tags: [bem, fem, elastodynamics, structural-mechanics, eurocode, teaching-mode, cad-drafting, tauri, angular, threejs, rust]
timestamp: 2026-10-30T14:55:00Z
state: bounded
children:
  - /capabilities/cap-geometry-meshing.md
  - /capabilities/cap-material-field.md
  - /capabilities/cap-solver-core.md
  - /capabilities/cap-interactive-canvas.md
  - /capabilities/cap-parametric-benchmarking.md
  - /capabilities/cap-project-persistence.md
  - /capabilities/cap-structural-mechanics-teaching.md
  - /capabilities/cap-eurocode-rc-design.md
---

# Intent

Provide an advanced, accessible, and high-performance desktop engineering and educational environment. The platform integrates continuum mechanics (BEM-FEM operator splitting and standalone BEM/FEM solvers for inhomogeneous media) with modern interactive modeling and pedagogical structural mechanics, organized into three primary operational workspaces:
1. **Benchmark & Verification Workbench Mode:** Rapid testing and validation of continuum mechanics solvers (Static, Harmonic, Transient) on standard parametric presets.
2. **Custom Modeling & CAD Import Studio Mode:** User-driven interactive drafting (2D/3D geometry creation, custom member drawing, boundary assignment) and CAD file import (DXF, SVG, STEP, STL).
3. **Structural Mechanics & Eurocode Teaching Studio Mode:** Step-by-step educational analysis of isostatic and hyperstatic frames/trusses, energy principles, and Eurocode 2 reinforced concrete design.

# Scope

### Included:
- **Tri-Mode Application Workspaces:**
  - **Mode 1 (Benchmark & Verification Workbench):** Pre-configured 2D/3D structural benchmarks (cantilevers, plates with holes, cracked media, 3D solid beams) with instant material field variation and automated verification plots.
  - **Mode 2 (CAD Drafting & Custom Modeling Studio):** Interactive 2D/3D geometric sketching, line/polygon wireframe creation, arbitrary support/load placement, and CAD import (DXF, STEP, STL).
  - **Mode 3 (Structural Mechanics & Eurocode Teaching Studio):**
    - Isostatic systems: Method of Sections (Ritter), Method of Joints (κόμβοι), Analytical $M(x), Q(x), N(x)$ diagrams.
    - Hyperstatic systems: Force Method, Clapeyron Three-Moment Equation, Slope-Deflection Method, Cross Moment Distribution, Kani Method.
    - Energy principles: Principle of Virtual Work (ΑΔΕ), Castigliano's Theorems.
    - Eurocode 2 RC Design: Cross-sectional ULS/SLS design of beams and columns, interaction diagrams, stirrups, and reinforcement detailing drawings.
- **Continuum Solver Infrastructure:**
  - 2D (plane stress/strain) and 3D solid elastodynamics.
  - Pure FEM and Pure BEM direct solvers across Static, Harmonic (Frequency Domain), and Transient (Talbot contour Laplace inversion and Newmark-$\beta$ direct time marching).
  - Open-Closed spatial inhomogeneity material engine (directional gradients, math expressions, discrete inclusions, non-linear MCFT concrete).
- **Interactive Visualization & UI:**
  - GPU-accelerated Three.js canvas with interactive face picking, deformed shape animations, stress colormaps, and modal dialog configurations.
  - Collapsible bottom engineering dock with SVG response history curves and live convergence dashboards.

### Excluded:
- Full-blown CSG solid boolean modeling in the first CAD release (focus is wireframe/surface boundary representation).
- Multi-physics coupling outside elastodynamics and structural mechanics (e.g. thermal-fluid interaction).

# Relationships

- Child: [Geometry & Meshing](/capabilities/cap-geometry-meshing.md)
- Child: [Material Field Specification](/capabilities/cap-material-field.md)
- Child: [Solver Core Engine](/capabilities/cap-solver-core.md)
- Child: [Interactive 3D Canvas](/capabilities/cap-interactive-canvas.md)
- Child: [Parametric Benchmarking](/capabilities/cap-parametric-benchmarking.md)
- Child: [Project Persistence](/capabilities/cap-project-persistence.md)
- Child: [Structural Mechanics Teaching Engine](/capabilities/cap-structural-mechanics-teaching.md)
- Child: [Eurocode Structural Design Engine](/capabilities/cap-eurocode-rc-design.md)
