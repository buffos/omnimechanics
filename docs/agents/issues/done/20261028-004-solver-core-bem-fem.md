## Issue Metadata

- Issue number: `004`
- Owning capability node: `/.okf/capabilities/cap-solver-core.md`
- Artifact root: `docs/architecture/cap-solver-core/`
- Issue file: `docs/agents/issues/done/20261028-004-solver-core-bem-fem.md`
- Category: `feature`
- Execution type: `AFK`
- Review gate: `visual-review`
- Suggested state: `done`

## Parent Artifacts

- `.okf/project.md`
- `.okf/capabilities/cap-solver-core.md`
- `.okf/capabilities/cap-geometry-meshing.md`
- `.okf/capabilities/cap-material-field.md`
- `.okf/capabilities/cap-interactive-canvas.md`

## What to build

Implement the computational multi-solver core in Rust (`src-tauri/src/solver/`):
1. **Variational Boundary Shape Matrix ($\mathbf{S}_{\text{shape}}$):**
   - Exact analytical / Gauss quadrature formulation for 2D (Line2) and 3D (Tri3) boundary elements.
   - Energy-consistent transformation between surface tractions $\mathbf{t}_\Gamma$ and nodal forces $\mathbf{F}_\Gamma$ ($\mathbf{F}_\Gamma = \mathbf{S}_{\text{shape}} \mathbf{t}_\Gamma$, $\mathbf{t}_\Gamma = \mathbf{S}_{\text{shape}}^{-1} \mathbf{F}_\Gamma$).
2. **Standalone Pure FEM Solver:**
   - Isoparametric element stiffness matrices ($\mathbf{K}_e = \int \mathbf{B}^T \mathbf{C}(\mathbf{x}) \mathbf{B} d\Omega$) for 2D (Tri3, Quad4) and 3D (Tet4, Hex8).
   - Global sparse assembly and direct solve via `faer`.
   - Non-linear Newton-Raphson loop for material perturbations and Vecchio-Collins / J2 plasticity.
3. **Standalone Pure BEM Solver:**
   - Kelvin fundamental solutions ($\mathbf{U}^*$ displacement and $\mathbf{T}^*$ traction tensors) for 2D plane strain/stress and 3D elastostatics.
   - Singular and regular boundary element integration.
   - $\mathbf{H}\mathbf{u} = \mathbf{G}\mathbf{t}$ assembly, boundary condition sorting, and dense LU solve.
4. **BEM-FEM Hybrid Solver (BEM Predictor -- FEM Corrector):**
   - BEM solves homogeneous reference response $\mathbf{E}_0$.
   - $\mathbf{S}_{\text{shape}}$ transforms interface tractions to nodal forces.
   - FEM evaluates volume residual forces $\mathbf{R}_{\text{inhom}} = \int \mathbf{B}^T \Delta \boldsymbol{\sigma} d\Omega$ at Gauss points without tangent matrix inversion.
   - Accelerated iterative convergence loop with residual monitoring.
5. **FEM-BEM Hybrid Solver:**
   - FEM predictor with BEM boundary reaction corrector.
6. **Frontend Integration & Three.js Deformation Rendering:**
   - `SolverService` managing simulation execution.
   - Viewport deformed mesh rendering with interactive deformation magnification slider ($1\times, 10\times, 50\times, 100\times$).
   - Colormap switching to von Mises stress $\sigma_{\text{vM}}$ and displacement magnitude $\|\mathbf{u}\|$.
   - Convergence HUD panel displaying iteration count and residual norm $\|\mathbf{R}\| / \|\mathbf{F}\|$.

## Acceptance criteria

- [x] Rust `boundary_shape` module computing $\mathbf{S}_{\text{shape}}^e$ and assembling global symmetric positive-definite consistency matrix.
- [x] Standalone Pure FEM solver with sparse direct solver (`faer`) and Newton-Raphson iteration for inhomogeneous material fields.
- [x] Standalone Pure BEM solver with Kelvin fundamental kernels and singular Gauss integration.
- [x] BEM-FEM and FEM-BEM hybrid iterative solvers with variational interface coupling via $\mathbf{S}_{\text{shape}}$.
- [x] Tauri IPC command `run_simulation` returning nodal displacements, stresses, deformed coordinates, and convergence logs.
- [x] Angular `SolverService` connecting sidebar controls to simulation execution.
- [x] Three.js `ViewportComponent` rendering deformed meshes with configurable displacement amplification and von Mises stress colormaps.
- [x] Convergence HUD displaying live iteration counts and residual norms.
- [x] Rust unit tests verifying boundary shape matrix symmetry, FEM benchmark deflection, and BEM fundamental solutions.

## Artifact sync required

- Application PRD: `none - capability foundation`
- Application architecture summary: `none - capability foundation`
- Owning capability node/artifacts: `required: .okf/capabilities/cap-solver-core.md`
- Issue registry: `required`; node `issues:` reference: `required`
- Reason/no-impact decision: `Grounding issue for the multi-solver computational engine.`

## Human review gate

- Visual review: User verifies in the desktop window that selecting each solver mode (Pure FEM, Pure BEM, BEM-FEM, FEM-BEM) and clicking "Run Selected Solver" executes the simulation, renders the physically deformed mesh with displacement/von Mises colormap, and displays convergence statistics. **Approved by user.**

## Blocked by

None - Issues 001, 002, 003 completed

## Artifact anchors

- Capability: `cap-solver-core`
- Capability: `cap-geometry-meshing`
- Capability: `cap-material-field`
- Capability: `cap-interactive-canvas`

## Verification obligations

- mode: `when-supported`
- surfaces:
  - backend-boundary: `when-supported` (Rust unit tests for solver modules and benchmark solutions)
  - frontend-integration: `when-supported` (Angular build and IPC integration)
  - end-to-end: `visual-review` (Simulation execution, deformed 3D mesh, and convergence display)
