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
- Closed issue 002: Geometry Definition & Conformal Auto-Tessellation Engine (`docs/agents/issues/done/20261028-002-geometry-conformal-meshing.md`). Conformal meshing for 2D/3D benchmarks, boundary tags (Left, Right, Top, Bottom, Hole, Front, Back), Angular MeshService, and Three.js 3D viewport rendering verified and visual-review approved. Transitioned `cap-geometry-meshing` to `implemented`.
- Closed issue 003: Material Modeling & Open-Closed Spatial Inhomogeneity Engine (`docs/agents/issues/done/20261028-003-material-inhomogeneity-field.md`). Implemented Open-Closed MaterialField trait with 5 strategies (Directional Gradient FGM, Analytical Math Expressions, Discrete Inclusions/Defects, Vecchio-Collins MCFT non-linear concrete, and J2 von Mises elastoplasticity). Scientific Turbo colormap rendering, HUD Colorbar, and live slider reactivity in Three.js verified and visual-review approved. Transitioned `cap-material-field` to `implemented`.
- Closed issue 004: Multi-Domain Computational Solver Core (`docs/agents/issues/done/20261028-004-solver-core-bem-fem.md`). Implemented variational boundary shape matrix S_shape with PCG inversion, standalone pure FEM direct solver with Gauss quadrature internal force integration, standalone pure BEM Kelvin solver, authentic BEM-FEM & FEM-BEM predictor-corrector relaxation loop with relative error convergence monitoring, Tauri run_simulation IPC, Angular SolverService, and Three.js deformed mesh visualization with live Convergence HUD. Transitioned `cap-solver-core` to `implemented`. User visual review approved.
- Opened issue 005: Dynamic Elastodynamics via Transformed Laplace-Talbot & BEM-Preconditioned GMRES (`docs/agents/issues/pending/005-dynamic-laplace-talbot-solver.md`). Extends cap-solver-core with consistent mass matrix assembly, complex modified Bessel dynamic BEM kernels, matrix-free BEM-preconditioned GMRES solver, parallel Talbot contour integration via rayon, and Harmonic Steady-State & Transient wave playback. Re-bounded cap-solver-core for active dynamic development.
- Closed issue 005: Dynamic Elastodynamics via Transformed Laplace-Talbot & BEM-Preconditioned GMRES (`docs/agents/issues/done/20261028-005-dynamic-laplace-talbot-solver.md`). Consistent mass matrix, complex modified Bessel dynamic kernels (K0, K1, K2), complex matrix-free Krylov GMRES with diagonal preconditioning, and parallel Talbot contour time-domain inversion implemented in Rust (21/21 tests passing). Interactive Harmonic Steady-State and Transient wave playback bar integrated into Angular and Three.js viewport. Transitioned `cap-solver-core` to `implemented`. User visual review approved.

## 2026-10-29
- Closed issue 006: Impedance-Matched Dynamic Robin Transmission Conditions & Robin-Schwarz Coupling (`docs/agents/issues/done/20261029-006-dynamic-robin-schwarz-coupling.md`). Derived and implemented anisotropic impedance tensor Z(s), boundary radiation damping matrix C_Robin(s), mixed Robin-Dirichlet BEM predictor, and Robin-Schwarz domain decomposition iteration contracting in <= 6 iterations. Exposed "Dynamic Robin Transmission (Z(s) Absorbing BEM-FEM)" toggle in Angular frontend. Verified across 24 Rust unit tests and Angular production build. Monograph expanded to 34 pages compiling cleanly.
- Established Fog-of-War Navigation Roadmap (`docs/roadmap.md`) per project planning profile.
- Clarified `cap-solver-core`: explicit distinction between core Operator Splitting for inhomogeneous continua and optional Unbounded Medium Robin radiation damping for soil-structure / semi-infinite domains.
- Updated `cap-interactive-canvas`: ported standalone `DialogService` / `DialogHost` and `NotificationService` / `NotificationHost` from `code.teacher` to declutter the sidebar UI and provide modular modal dialogs for advanced configuration. Removed blind global Robin damping from cantilever beam sidebar.
- Implemented Issue 007: Classical Direct Time-Marching via Newmark-$\beta$ and Hilber--Hughes--Taylor $\alpha$ (`docs/agents/issues/done/20261029-007-classical-time-marching-newmark.md`). Implemented pre-factorized effective dynamic stiffness $\hat{\mathbf{K}} = (1 + a_1 \beta_R)\mathbf{K} + (a_0 + a_1 \alpha_R)\mathbf{M}$ with initial acceleration recovery and kinematic Taylor predictor in `time_marching.rs`. Direct benchmark test against Talbot Laplace contour confirms dynamic deflection consistency ($0.124\text{ mm}$ vs $0.125\text{ mm}$, $<0.8\%$ difference). Exposed integration method segmented toggle in Angular frontend. Authored Section 10 in monograph (38 pages, compiled with LuaLaTeX exit code 0). 26/26 Rust unit tests passing cleanly. Closed issue 007.
- Implemented Issue 008: Collapsible Bottom Dock Response Chart & Tabbed Sidebar UI (`docs/agents/issues/pending/20261029-008-bottom-dock-response-chart-ui-tabs.md`). Reorganized left inspector into 3 workflow tabs (Mesh, Material, Solver) with pinned bottom action button. Built collapsible Bottom Engineering Dock featuring reactive SVG response history chart (Talbot Cyan curve, Newmark-β Amber curve with time step markers, Euler-Bernoulli dashed reference line, real-time animation cursor scrubbing, CSV export, and SVG vector export for paper figures). Verified across 28 Rust tests and Angular build. Registered as `awaiting-human-review`.

## 2026-10-30
- Removed Oraiopoulos BEM-FEM (`BEMFEM`) from the static solver pipeline select in Angular frontend (`app.component.html`), keeping the backend implementation intact. Set default static solver mode to `PureFEM`.
- Executed Fog-of-War Planning update per user specifications:
  - Established Tri-Mode Workspace Architecture in `project.md`:
    1. **Benchmark & Verification Workbench Mode** (pre-canned parametric structures, solver comparison).
    2. **Custom CAD Drafting & Import Studio Mode** (user-drawn 2D/3D frames, boundaries, and DXF/STEP/STL import).
    3. **Structural Mechanics & Eurocode Teaching Studio Mode** (step-by-step educational analysis and EC2 concrete design).
  - Consolidated Mode 2 geometry modeling into `cap-geometry-meshing` as a structural rollup node (`state: foggy`, `reducer: min`), decomposing it into 3 sub-capabilities:
    - `cap-geometry-presets` (state: `implemented`): Parametric Benchmark Presets & Conformal Meshing Engine.
    - `cap-geometry-cad-import` (state: `foggy`): External CAD Geometry Ingestion (DXF, STEP, STL) & Polyline Healing.
    - `cap-geometry-canvas-sketcher` (state: `foggy`): Interactive 2D/3D Canvas Drafting & Custom Geometry Sketcher.
  - Created Mode 3 educational capabilities:
    - `cap-structural-mechanics-teaching` (state: `foggy`): Structural Mechanics Step-by-Step Educational & Analysis Engine (Isostatic: Ritter / Joints / Diagrams; Hyperstatic: Force Method, Clapeyron 3-Moment, Slope-Deflection, Cross, Kani; Energy: Virtual Work / Castigliano).
    - `cap-eurocode-rc-design` (state: `foggy`): Eurocode Reinforced Concrete Structural Design & Detailing Engine (EC2 beam flexure/shear, column interaction N-M, SLS crack/deflection, rebar detailing).
  - Updated `project.md`, `index.md`, and `docs/roadmap.md`. Validated OKF bundle conformance.
  - Cleared fog across the entire capabilities graph using `/fog-of-war-planning` with explicit user alignment on the educational step-by-step purpose:
    - Advanced `cap-structural-mechanics-teaching` from `foggy` to `specified`: detailed pedagogical step-by-step specifications for Isostatic systems (Joints, Ritter, M-Q-N diagrams), Hyperstatic systems (Force Method with Vereshchagin integration, Clapeyron, Cross Moment Distribution, Kani Method with rotation & sway balancing, Slope-Deflection), Energy principles (Virtual Work / Castigliano), exact rational fraction arithmetic, interactive step-walker UI, and 8 acceptance scenarios.
    - Advanced `cap-eurocode-rc-design` from `foggy` to `specified`: EC2 ULS beam flexure/shear, column N-M interaction curves, SLS crack/deflection, and 2D rebar detailing.
    - Advanced `cap-geometry-canvas-sketcher` from `foggy` to `specified`: interactive 2D/3D member and polygon sketcher with magnetic snapping, support/load assignment, and dual frame/continuum mesh handoff.
    - Advanced `cap-geometry-cad-import` from `foggy` to `specified`: 2D DXF/SVG parsing with polyline healing and 3D STL manifold mesh verification.
    - Materialized rollup state of `cap-geometry-meshing` to `specified`.
    - Entire capability map now has 0 foggy nodes.
