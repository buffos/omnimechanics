# OKF Knowledge Bundle Log

## 2026-11-02
- Closed issue 016: 2D Structural Model, Material/Section Catalogs & Pre-Solve Kinematic Diagnostics (`docs/agents/issues/done/20261101-016-frame-model-catalog-kinematic-diagnostics.md`).
- Closed issue 017: Linear Static DSM Solver, Analytical Member Diagrams, Envelopes & DSM Walkthrough (`docs/agents/issues/done/20261101-017-linear-static-dsm-analytical-diagrams-envelopes.md`). Implemented Euler-Bernoulli element stiffness with static condensation of end releases and rigid offsets, inclined support transformations, settlement and thermal load assembly, analytical internal force polynomials M(x), Q(x), N(x) under Greek sign conventions, pointwise envelopes across combinations (`SC-007`), and pedagogical DSM walkthrough generator (`SC-009`). 15 tests passing, limits check passing.
- Closed issue 018: Background Analysis Jobs, Unilateral Active-Set, 2nd-Order P-Delta & Linear Buckling (`docs/agents/issues/done/20261101-018-async-jobs-unilateral-pdelta-buckling.md`). Implemented asynchronous `AnalysisJob` coordinator with revision fencing, cooperative cancellation (`SC-008`), active-set iteration for unilateral supports and tension/compression-only braces (`SC-003`), 2nd-order $P\text{-}\Delta$ Newton solver with divergence detection and base reaction amplification (`SC-004`), and linear eigenvalue buckling solver with normalized mode shapes (`SC-004`). Registered IPC commands `frame_start_analysis_job`, `frame_cancel_analysis_job`, `frame_get_analysis_job_result`. 20 tests passing, limits check passing.

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
- Executed closeout for issues 010, 011, 012, 013 (Mini-Papers 2-5):
  - Archived issues 010, 011, 012, 013 to `docs/agents/issues/done/` and cleared them from the active registry `docs/agents/issues/issues.md`.
  - Transitioned `cap-parametric-benchmarking` to `implemented` with full issue traceability (008, 009, 010, 011, 012, 013).
  - Only Issue 014 (Mode 3 Structural Mechanics Teaching Studio Slice 1) remains in the active queue.
- Implemented Issue 014 (Slice 1 - Mode 3 Structural Mechanics Teaching Studio):
  - Created Rust analytical solvers: `fraction.rs` (ExactFraction p/q arithmetic with LaTeX formatting), `isostatic_beam.rs` (reactions, piecewise M-Q-N functions, Q roots, M_max), `truss.rs` (Method of Joints and Ritter cut section equilibrium), `cross_kani.rs` (Cross Moment Distribution and Kani iteration tables), and `presets.rs`.
  - Registered Tauri IPC command `solve_teaching_system`.
  - Created Angular Teaching Studio: Top Header Mode Switcher (Mode 1 Continuum vs Mode 3 Teaching), `TeachingSidebarComponent` with Angular 22 Signal Forms, `TeachingCanvasComponent` with sharp SVG structural elements, reactions, and M/Q/N diagrams, and `StepWalkerComponent` with timeline controls and pedagogical cards.
  - All tests passing (`cargo test --lib`, `npx ng build`, `npm run check:limits`), registered as `awaiting-human-review`.
    - Entire capability map now has 0 foggy nodes.
- Executed closeout for Issue 014 (Mode 3 Teaching Studio Slice 1):
  - Completed user visual and interaction review: added variable point load position slider $x_P$, rendered visible load arrows, ensured strictly vertical shear step boundaries, restored segmented-control styling, pinned preview SVG canvas with independent scrollable derivation pane and sticky toolbar, added full hyperstatic reactions and continuous M, Q, N diagrams to the Cross solver, and fixed LaTeX ampersand escaping.
  - Archived Issue 014 to `docs/agents/issues/done/20261030-014-structural-mechanics-teaching-slice1.md` and removed it from active registry `docs/agents/issues/issues.md`.
  - All automated checks verified (11/11 Rust teaching tests, 0 Angular build errors, all files <= 1000 lines).
- Created Issue 015 (`cap-geometry-canvas-sketcher`):
  - Formulated Vertical Slice 1 for Mode 2 Interactive 2D Canvas Drafting & Custom Geometry Sketcher.
  - Scope: 2D drawing canvas with magnetic snapping, orthogonal lock, support palette (Fixed, Pinned, Roller, Hinge), load palette (Point load, Distributed load, Moments), and seamless handoff to Mode 3 Teaching Studio.
  - Registered in active registry `docs/agents/issues/issues.md` as `ready`.

## 2026-10-31

### Issue 015 gate failure, fog clearing for 2D structural modeling & analysis

- `$process-reference-issue 015` stopped before implementation because of material gaps (no frame
  solver, divergent domain models, missing application synthesis, no frontend harness). Gaps are recorded
  in `docs/architecture/cap-geometry-canvas-sketcher/requirements-gap-analysis.md`.
- Fog-of-war planning (clear fog from code) ran 4 batches / 40 user decisions, recorded in
  `docs/architecture/cap-geometry-canvas-sketcher/discovery-notes.md`. Target: any 2D structure
  (beams, trusses, springs, catenary cables) with live DSM results, P-Δ/buckling, EN 1990/1998
  combinations and seismic analysis, and step-by-step when possible.
- Topology (user-confirmed, Q29/Q34):
  - New top-level `cap-frame-analysis-engine` (bounded). It owns the structural model.
  - New top-level shared `cap-eurocode-actions-combinations` (bounded), shared with RC design.
  - `cap-geometry-canvas-sketcher` converted to a roll-up with children `cap-sketcher-frame-modeler`
    (bounded) and `cap-sketcher-continuum-domain` (bounded).
- State changes:
  - `cap-geometry-canvas-sketcher`: `in-progress` (invalid) / index `specified` → roll-up `bounded`
    (derived: min of children).
  - `cap-geometry-meshing` (roll-up): `specified` → `bounded`, a derived transition caused by the
    sketcher child.
  - `cap-structural-mechanics-teaching`: invalid `in-progress` repaired to `specified`. The classifier
    and DSM walkthrough extension is noted, and a PRD refresh is pending.
  - New nodes: `cap-frame-analysis-engine`, `cap-eurocode-actions-combinations`,
    `cap-sketcher-frame-modeler`, `cap-sketcher-continuum-domain` (all `bounded`).
- Root verification block normalised to the profile schema (`mode: required`; surfaces backend-boundary
  required, frontend-integration required, end-to-end catalog-only).
- Issue 015 archived to `done/` as **superseded (not implemented)** and removed from the active
  registry.
- Totals (capabilities): foggy 0, bounded 8, specified 3, implemented 4.

## 2026-11-01

### Architecture specification pipeline completed across all bounded nodes

- Executed the `$bounded-capability-spec-orchestrator` architecture pipeline (`domain-glossary`, `prd`,
  `canonical-domain-model`, `canonical-use-cases`, `canonical-api-cli-contract`, `acceptance-scenarios`,
  `readiness-review`, `orchestration-status`) across all 6 own-state `bounded` capabilities:
  - `cap-frame-analysis-engine` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-frame-analysis-engine/`, `SC-001..SC-009`, 0 High/Medium blockers.
  - `cap-eurocode-actions-combinations` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-eurocode-actions-combinations/`, `SC-001..SC-009`, 0 High/Medium blockers.
  - `cap-sketcher-frame-modeler` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-sketcher-frame-modeler/`, `SC-001..SC-014`, 0 High/Medium blockers.
  - `cap-sketcher-continuum-domain` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-sketcher-continuum-domain/`, `SC-001..SC-009`, 0 High/Medium blockers.
  - `cap-interactive-canvas` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-interactive-canvas/`, `SC-001..SC-012`, 0 High/Medium blockers.
  - `cap-project-persistence` (`bounded` -> `specified`): 8 files in `docs/architecture/cap-project-persistence/`, `SC-001..SC-012`, 0 High/Medium blockers.
- Derived roll-up transitions (`state_policy.mode: rollup`, `reducer: min`):
  - `cap-geometry-canvas-sketcher`: `bounded` -> `specified` (children `cap-sketcher-frame-modeler` and `cap-sketcher-continuum-domain` both reached `specified`).
  - `cap-geometry-meshing`: `bounded` -> `specified` (children `cap-geometry-presets` [`implemented`], `cap-geometry-cad-import` [`specified`], and `cap-geometry-canvas-sketcher` [`specified`] are all >= `specified`).
- Root project `project.md`: `bounded` -> `specified` with synchronized application synthesis documents (`docs/prd.md` and `docs/architecture/application-architecture-summary.md`).
- Totals (capabilities): foggy 0, bounded 0, specified 11, implemented 4.

### Issue slicing for `cap-frame-analysis-engine`

- Sliced `cap-frame-analysis-engine` (`docs/architecture/cap-frame-analysis-engine/`) into 5 dependency-ordered vertical slices in `docs/agents/issues/pending/` and registered them in `docs/agents/issues/issues.md` (`# Current Max Issue ID` updated to `020`):
  - `016`: 2D Structural Model, Material/Section Catalogs & Pre-Solve Kinematic Diagnostics (`SC-002`) — unblocked (`ready-for-agent`).
  - `017`: Linear Static DSM Solver, Analytical Member Diagrams, Envelopes & DSM Walkthrough (`SC-001`, `SC-007`, `SC-009`) — blocked by `016`.
  - `018`: Background Analysis Jobs, Unilateral Active-Set, 2nd-Order P-Delta & Linear Buckling (`SC-003`, `SC-004`, `SC-008`) — blocked by `017`.
  - `019`: Exact Irvine Elastic Catenary Cable Solver & Slack Detection (`SC-005`) — blocked by `018`.
  - `020`: Modal Analysis, EN 1998 Response Spectrum (SRSS/CQC), Lateral Force & OpenSees Benchmarks (`SC-006`) — blocked by `018, 019`.
- Linked all 5 pending issue paths in `.okf/capabilities/cap-frame-analysis-engine.md`.

## 2026-11-02

### Delivery of Issues 016, 017, 018, 019, 020 for `cap-frame-analysis-engine`

- Closed Issue 016 (`frame-model-catalog-kinematic-diagnostics`):
  - Delivered 2D frame structural model data types, JSON serialization, section/material catalog, validation rules, and SVD/eigenvalue kinematic mechanism diagnostics (`SC-002`).
- Closed Issue 017 (`linear-static-dsm-analytical-diagrams-envelopes`):
  - Delivered live direct stiffness method linear solver (< 50 ms for 500 nodes / 1000 members), analytical Hermite member internal diagrams (M, Q, N, deformed shape), combination envelope extrema, and Mode 3 DSM educational walkthrough step generator (`SC-001`, `SC-007`, `SC-009`).
- Closed Issue 018 (`async-jobs-unilateral-pdelta-buckling`):
  - Delivered background async job worker with thread pools, revision fencing, cancellation tokens (`SC-008`), active-set iteration for tension-only/compression-only elements and unilateral supports (`SC-003`), second-order P-Delta solver with deformed base reactions, and linear eigenvalue buckling analysis with Euler critical load parity (`SC-004`).
- Closed Issue 019 (`irvine-elastic-catenary-cable-solver`):
  - Delivered exact 2D Irvine elastic catenary cable element solver (`catenary.rs`), supporting `UnstressedLength`, `SagRatio`, and `Pretension` conversions, analytical 2x2 flexibility Jacobian inversion to 4x4 tangent stiffness, outer global Newton-Raphson equilibrium iteration, slack cable detection ($H < 10^{-4} EA$), positive-definite regularization, and 21-point catenary curve geometry generation (`SC-005`).
  - Unit tests in `catenary_test.rs` pass with 100% compliance.
  - Archived to `docs/agents/issues/done/20261101-019-irvine-elastic-catenary-cable-solver.md` and unblocked Issue 020.
- Closed Issue 020 (`modal-rsa-lateral-force-opensees-benchmarks`):
  - Delivered modal eigenfrequency analysis (`dynamics.rs`), dynamic mass matrix assembly ($G + \psi_2 Q$ and `LumpedMass`), automatic mode expansion until cumulative effective mass $\ge 90\%$, EN 1998-1 Lateral Force solver with vertical story shear distribution, and Response Spectrum Analysis with SRSS and Der Kiureghian CQC modal combinations (`SC-006`).
  - Created offline reproducible benchmark generator `scripts/generate-frame-benchmarks.py` and committed 5 reference benchmark CSVs in `docs/benchmarks/frame/`.
  - Comprehensive integration tests in `dynamics_test.rs` and `benchmarks_test.rs` passing within $10^{-6}$ (linear) and $10^{-3}$ (nonlinear).
  - Archived to `docs/agents/issues/done/20261101-020-modal-rsa-lateral-force-opensees-benchmarks.md`.
- Capability `cap-frame-analysis-engine` transitioned to `state: implemented` (all 5 vertical slices complete).
- Totals (capabilities): foggy 0, bounded 0, specified 10, implemented 5.

## 2026-11-02

### Issue slicing for `cap-eurocode-actions-combinations`

- Sliced `cap-eurocode-actions-combinations` (`docs/architecture/cap-eurocode-actions-combinations/`) into 2 dependency-ordered vertical slices in `docs/agents/issues/pending/` and registered them in `docs/agents/issues/issues.md` (`# Current Max Issue ID` updated to `022`):
  - `021`: Greek National Annex Profile & EN 1990 Load Combination Engine (`SC-001`, `SC-002`, `SC-003`, `SC-004`, `SC-009`) — unblocked (`ready-for-agent`).
  - `022`: EN 1998-1 Seismic Spectra, Mass Participation, Lateral Force & CQC Correlation Engine (`SC-005`, `SC-006`, `SC-007`, `SC-008`, `SC-009`) — blocked by `021`.
- Linked `021` and `022` in `.okf/capabilities/cap-eurocode-actions-combinations.md`.

### Issue slicing for `cap-sketcher-frame-modeler`

- Sliced `cap-sketcher-frame-modeler` (`docs/architecture/cap-sketcher-frame-modeler/`) into 4 dependency-ordered vertical slices in `docs/agents/issues/pending/` and registered them in `docs/agents/issues/issues.md` (`# Current Max Issue ID` updated to `026`):
  - `023`: Mode 2 Workspace, 2D SVG Drafting Canvas, Priority Snapping, Auto-Splits & CAD Transforms (`SC-001`, `SC-002`, `SC-003`, `SC-004`) — unblocked (`ready-for-agent`).
  - `024`: Property Inspector, Internal Hinge Expansion, Support/Load Palettes, Live Linear DSM Diagrams & Mechanism Animation (`SC-005`, `SC-006`, `SC-007`, `SC-009`) — blocked by `023`.
  - `025`: Multi-LoadCase Management, EN 1990/1998 Combinations UI, Governing Envelopes & Background Nonlinear/Seismic Jobs (`SC-008`, `SC-010`) — blocked by `021, 022, 024`.
  - `026`: Mode 3 Method Classifier Handoff, Teaching Preset Import, Project Persistence & SVG/PNG Diagram Export (`SC-011`, `SC-012`, `SC-013`, `SC-014`) — blocked by `025`.
- Linked `023`–`026` in `.okf/capabilities/cap-sketcher-frame-modeler.md`.

### Closeout for Issue 021 (`cap-eurocode-actions-combinations`)

- Closed Issue 021 (`eurocode-na-profile-en1990-combination-engine`):
  - Implemented Greek National Annex (`GR`) default factors catalog ($\gamma_{G,\text{sup}}=1.35$, $\gamma_{G,\text{inf}}=1.00$, $\gamma_{Q,\text{sup}}=1.50$, $\gamma_{Q,\text{inf}}=0.00$, $\xi=0.85$, $\psi_0/\psi_1/\psi_2$ tables) and structural system $q$-factor guidance in `src-tauri/src/actions/na_profile.rs`.
  - Implemented EN 1990 load combination generator (`combinations.rs`) for `ULS_STR`/`ULS_GEO` (Eq. 6.10 & Eq. 6.10a/b), `SLS_CHAR` (Eq. 6.14b), `SLS_FREQ` (Eq. 6.15b), and `SLS_QP` (Eq. 6.16b), with automatic envelope grouping and provenance preservation (`Generated`, `UserEdited`, `Manual`).
  - Registered Tauri IPC commands: `actions_get_default_na_profile`, `actions_validate_na_profile`, `actions_generate_combinations`, `actions_save_custom_combination`, and `actions_delete_or_reset_combination`.
  - Unit tests in `actions_test.rs` pass with 100% compliance.
  - Archived to `docs/agents/issues/done/20261102-021-eurocode-na-profile-en1990-combination-engine.md`, removed from active registry `docs/agents/issues/issues.md`, and unblocked Issue 022.

### Closeout for Issue 022 (`cap-eurocode-actions-combinations`)

- Closed Issue 022 (`en1998-seismic-spectra-mass-lateral-cqc`):
  - Implemented EN 1998-1 Type 1 horizontal and vertical design spectrum engine ($S_e(T), S_d(T)$, damping correction $\eta$, plateau, lower-bound $\beta a_g$) in `src-tauri/src/actions/seismic.rs`.
  - Implemented seismic mass participation rule ($G + \psi_{E,i} Q$) and signed `SEISMIC` combinations ($\pm 1.00 E_x$ horizontal and 8 signed combinations for 30% bidirectional rule) in `combinations.rs` and `seismic.rs`.
  - Implemented equivalent static lateral force distribution ($F_b$, story forces $F_i$, $\lambda = 0.85$ correction).
  - Implemented Der Kiureghian CQC modal correlation matrix and SRSS diagonal matrix.
  - Registered Tauri IPC commands: `actions_compute_design_spectrum`, `actions_compute_seismic_mass_rule`, `actions_compute_lateral_forces`, and `actions_compute_cqc_matrix`.
  - Unit tests in `seismic_test.rs` pass with 100% compliance.
  - Archived to `docs/agents/issues/done/20261102-022-en1998-seismic-spectra-mass-lateral-cqc.md`, removed from active registry `docs/agents/issues/issues.md`, and unblocked Issue 025 dependency on 022.
- Capability `cap-eurocode-actions-combinations` transitioned to `state: implemented` (both vertical slices complete).
- Totals (capabilities): foggy 0, bounded 0, specified 9, implemented 6.

### Closeout for Issue 023 (`cap-sketcher-frame-modeler`)

- Closed Issue 023 (`mode2-drafting-canvas-snapping-splits-transforms`):
  - Bootstrapped Vitest frontend test harness (`vitest.config.ts`, `npm run test:unit`), verified 14 unit tests passing in pure TypeScript.
  - Implemented `DraftingViewport` affine screen $\leftrightarrow$ world transformations with pan, zoom, dynamic $y$-up grid lines, and selectable step ($0.1\text{ m}, 0.5\text{ m}, 1.0\text{ m}$).
  - Implemented `SnappingEngine` with priority-ordered snap evaluation (`NodeSnap` $< 15\text{ px}$, `IntersectionSnap`, `MidpointSnap`, `OrthoLock`, `AngularGuideSnap`, `GridSnap`), exact `(x, y)` and `(L, θ)` numeric keyboard entry, and rejection of zero-length elements (`ZERO_LENGTH_ELEMENT`).
  - Implemented `TopologicalSplitService` for automatic member splitting at crossings and intermediate nodes while partitioning distributed loads and preserving release/section/material continuity.
  - Implemented `CadTransformationService` supporting marquee box selection (enclosed vs. crossing), `Move`, `Copy`, `Mirror` across a 2-point axis, `LinearArray` with coincident node welding, and cascade `Delete`.
  - Implemented `UndoRedoStack` in `ModelerStore` with monotonic `StructuralModel.revision` increments on every action.
  - Integrated Mode 2 workspace in application header switcher («Μοντελοποίηση & Ανάλυση» / 'Structural Modeler') and created canvas & toolbar components.
  - Archived to `docs/agents/issues/done/20261102-023-mode2-drafting-canvas-snapping-splits-transforms.md`, removed from active registry `docs/agents/issues/issues.md`, and unblocked Issue 024.

### Closeout for Issue 024 (`cap-sketcher-frame-modeler`)

- Closed Issue 024 (`palettes-inspector-live-linear-dsm-diagrams-mechanisms`):
  - Delivered `PropertyInspectorComponent` editing node coordinates, sections, materials, end releases (`axial`, `shear`, `moment`), rigid offsets, truss axial modes, and cable prestress specs with boundary validations (`INVALID_CABLE_SPEC`).
  - Implemented `InternalHingeExpansionPolicy` expanding a joint shared by $k \ge 2$ members into $k - 1$ moment releases (keeping 1 continuous) and rejecting $k < 2$ with `INVALID_HINGE_TARGET`.
  - Delivered `ModelerPalettesComponent` for assigning/removing supports (`Fixed`, `Pinned`, inclined `Roller`, `Guided`, `Elastic`, unilateral) and loads (`NodalLoad`, `MemberPointLoad` at arbitrary interior distance without splitting, uniform/trapezoidal `MemberDistributedLoad`, `SupportSettlement`, `ThermalLoad`, and `self_weight`), rejecting out-of-bounds point loads with `LOAD_OUT_OF_MEMBER_BOUNDS`.
  - Delivered `ModelerResultsService` providing 150 ms debounced live linear DSM solving (`frame_solve_linear`), tension-side $M/Q/N$ diagram rendering with Greek tension-side convention ($M > 0$ on bottom), extrema labels, support reaction vectors, and `CrossSectionHoverProbe` (`UC-Q02`).
  - Delivered collapsible `EndForceTableComponent` (`UC-Q03`) tabulating member end forces ($N_i, Q_i, M_i, N_j, Q_j, M_j$).
  - Delivered kinematic mechanism detection & animated mode oscillation when structure is unstable (`SC-009`), suppressing force diagrams and showing bilingual diagnostic banner.
  - Unit tests in `inspector.spec.ts` pass 100%, Angular bundle builds with 0 errors.
  - Archived to `docs/agents/issues/done/20261102-024-palettes-inspector-live-linear-dsm-diagrams-mechanisms.md`, removed from active registry `docs/agents/issues/issues.md`, and unblocked Issue 025.

### Closeout for Issue 025 (`cap-sketcher-frame-modeler`)

- Closed Issue 025 (`mode2-loadcases-eurocode-combinations-envelopes-async-jobs`):
  - Implemented multi-loadcase CRUD, automatic Eurocode EN 1990 combination generator (`actions_generate_combinations`), seismic settings form, and analytical/IPC EN 1998-1 Type 1 design spectrum $S_d(T)$ preview in `CombinationManagerComponent`.
  - Enhanced `ModelerStore` with loadcase and combination actions (`addLoadCase`, `deleteLoadCase`, `setCombinations`, `updateCombination`, `setSeismicSettings`, `isCombinationModalOpen`).
  - Implemented `AnalysisKind` selector (`LinearStatic`, `SecondOrderPDelta`, `LinearBuckling`, `Modal`, `ResponseSpectrum`, `LateralForce`) and `ResultTarget` selector (`LoadCase`, `Combination`, `Envelope`) in `ModelerToolbarComponent`.
  - Implemented dual upper/lower shaded tension-side envelope diagram rendering on canvas, pointwise governing combination readout in `CrossSectionHoverProbe` and `EndForceTableComponent` (`SC-010`).
  - Integrated background nonlinear/dynamic job coordination (`frame_start_analysis_job`, `frame_cancel_analysis_job`, `analysis://progress`) with progress pill indicator and rapid mutation cancellation discarding stale revisions (`SC-008`).
  - Implemented curved catenary cable path rendering and non-superposition warning banner (`BR-4`) when nonlinear elements or $P\text{-}\Delta$ are active.
  - Verified 28 Vitest unit tests passing, `ng build` passing with 0 errors, and limits checks passing across all 188 files.
  - Archived to `docs/agents/issues/done/20261102-025-mode2-loadcases-eurocode-combinations-envelopes-async-jobs.md`, removed from active registry `docs/agents/issues/issues.md`, and unblocked Issue 026.

### Closeout for Issue 026 (`cap-sketcher-frame-modeler`)

- Closed Issue 026 (`mode3-classifier-handoff-preset-import-diagram-export`):
  - Implemented Rust structural method classifier (`teaching::classifier`) evaluating `MethodEligibility` across classical methods (`IsostaticBeamFrame`, `TrussJointsRitter`, `ContinuousBeamClapeyronCross`, `ForceMethod`) and unconditionally enforcing `BR-5` ineligibility with bilingual explanations when `Cable` or unilateral elements are present while keeping DSM Walkthrough available (`SC-011`, `SC-012`).
  - Implemented bidirectional Mode 2 <-> Mode 3 handoff (`teaching::handoff`):
    - `teaching_open_step_by_step`: routes eligible models to analytical step decks and falls back to `frame_generate_dsm_walkthrough` when classical methods are exceeded (`SC-011`, `SC-012`).
    - `teaching_preset_to_structural_model`: loads standard Mode 3 teaching presets (Simply Supported with q/P, Cantilever, Overhanging Beam, 2-Span Continuous Beam, Pratt Truss) into Mode 2 as editable `StructuralModel` (`revision: 1`) (`SC-013`).
  - Added "Edit in Mode 2 Sketcher" («Επεξεργασία στον Σχεδιαστή Mode 2») button in Mode 3's `TeachingSidebarComponent` (`UC-C09`, `SC-013`).
  - Implemented lossless `.bem` JSON project file persistence (`project_save_workspace`, `project_load_workspace`, `DiagramExportService`) preserving geometry, sections, releases, offsets, cables, load cases, combinations, and viewport camera (`SC-014`).
  - Implemented standalone publication-grade SVG and high-DPI PNG diagram export (`DiagramExportService`) with title block, scales, extrema tags, and bilingual Greek/English legend (`SC-014`).
  - Added live method classifier badge and popover in `ModelerToolbarComponent` with one-click transitions to Mode 3 step-by-step or DSM walkthrough.
  - Verified 34 Vitest unit tests passing, Rust `teaching::classifier_test` passing 100%, `ng build` passing with 0 errors, and limits checks passing across all 197 files.
  - Archived to `docs/agents/issues/done/20261102-026-mode3-classifier-handoff-preset-import-diagram-export.md`, removed from active registry `docs/agents/issues/issues.md`.
- Capability `cap-sketcher-frame-modeler` transitioned to `state: implemented` (all 4 vertical slices delivered and verified).
- Totals (capabilities): foggy 0, bounded 0, specified 8, implemented 7.

## 2026-11-03

### Issue slicing across all remaining specified capabilities (`027`–`039`)

- Sliced all 6 remaining own-state `specified` capabilities into 13 dependency-ordered vertical slices in `docs/agents/issues/pending/` and registered them in `docs/agents/issues/issues.md` (`# Current Max Issue ID` updated to `039`):
  - **`cap-sketcher-continuum-domain`** (`027`–`029`):
    - `027`: Continuum Domain Polygon, Cutout & Topological Clearance Validation Engine (`SC-002..SC-005`) — unblocked (`ready-for-agent`).
    - `028`: Conformal Constrained Delaunay (CDT) Mesher, Boundary Tagging & Inclusion Subdomains (`SC-001`, `SC-006..SC-009`) — blocked by `027`.
    - `029`: Mode 1 Interactive 2D Continuum Sketcher UI, Inspector & Solver Handoff (`SC-001..SC-009`) — blocked by `028`.
  - **`cap-interactive-canvas`** (`030`–`031`):
    - `030`: 3D/2D Viewport Orthographic Camera, Clipping Plane, Raycast Boundary Picking & Scientific Colormaps (`SC-005..SC-009`) — unblocked (`ready-for-agent`).
    - `031`: Shared 2D SVG Viewport Kernel, Dialog Guard, Toast Deduplication & Dock Synchronization (`SC-001..SC-004`, `SC-010..SC-012`) — blocked by `030`.
  - **`cap-project-persistence`** (`032`–`033`):
    - `032`: Canonical `.bem` v1.0 Schema, SHA-256 Digest, Migration Pipeline & Atomic File I/O (`SC-001..SC-008`) — unblocked (`ready-for-agent`).
    - `033`: Revision Dirty Tracking, Unsaved-Changes Guard, Recent Projects (MRU) & Header Project Menu (`SC-001..SC-002`, `SC-004..SC-012`) — blocked by `032`.
  - **`cap-structural-mechanics-teaching`** (`034`–`035`):
    - `034`: Mode 3 Clapeyron Three-Moment Equation & Force Method (Flexibility) Step-by-Step Solvers (`Scenario 4` + Force Method) — unblocked (`ready-for-agent`).
    - `035`: Principle of Virtual Work (ΑΔΕ) Deflections, Interactive Cross/Kani Spreadsheets & LuaLaTeX Handout Export (`Scenario 5..8`) — blocked by `034`.
  - **`cap-geometry-cad-import`** (`036`–`037`):
    - `036`: 2D DXF/SVG Vector CAD Parser, Layer Filtering, Polyline Healing & Mode 1/2/3 Handoff (`Scenario 1`, `Scenario 2`) — blocked by `028`.
    - `037`: 3D Binary/ASCII STL Manifold Ingestion, Normal Orientation & Conformal Tet4/BEM Meshing (`Scenario 3`) — blocked by `036`.
  - **`cap-eurocode-rc-design`** (`038`–`039`):
    - `038`: Eurocode 2 (EN 1992-1-1) ULS Beam Flexural & Variable-Angle Shear Design Engine (`Scenario 1`, `Scenario 2`) — unblocked (`ready-for-agent`).
    - `039`: EC2 Column N-M Interaction, SLS Crack/Deflection Checks & Interactive 2D Rebar Detailing Studio (`Scenario 1..4`) — blocked by `038`.
- Linked `027`–`039` in their respective `.okf/capabilities/*.md` nodes. Roll-up capabilities (`cap-geometry-canvas-sketcher`, `cap-geometry-meshing`) derive their completion from their structural children.
